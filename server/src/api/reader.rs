use crate::cache::{reader_page_cache_key, READER_CACHE_VERSION};
use crate::endpoint::request_with_failover;
use crate::jm::invalidate_img_host;
use crate::reader::{page_name_from_image, prepare_page_image};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::{collections::HashMap, sync::Arc, time::Instant};
use tokio::sync::{Mutex, Semaphore};

const MANIFEST_PREWARM_COUNT: u32 = 5;
const PAGE_PREWARM_COUNT: u32 = 3;
const PAGE_PREWARM_CONCURRENCY: usize = 2;

static PAGE_MATERIALIZE_LOCKS: Lazy<Mutex<HashMap<String, Arc<Mutex<()>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static PAGE_PREWARM_SEMAPHORE: Lazy<Arc<Semaphore>> =
    Lazy::new(|| Arc::new(Semaphore::new(PAGE_PREWARM_CONCURRENCY)));

#[derive(Serialize)]
pub struct ManifestResponse {
    pub chapter_id: String,
    pub pages: Vec<PageInfo>,
}

#[derive(Serialize)]
pub struct PageInfo {
    pub index: u32,
    pub name: String,
    pub url: String,
}

/// GET /api/reader/:chapter_id/manifest
pub async fn get_manifest(
    State(app): State<AppState>,
    Path(chapter_id): Path<String>,
) -> Result<Json<ManifestResponse>, ApiError> {
    validate_chapter_id(&chapter_id)?;
    let request_chapter_id = chapter_id.to_string();
    let chapter = app
        .jm_request(move |client, endpoint| {
            let chapter_id = request_chapter_id.clone();
            Box::pin(async move { client.get_chapter(endpoint, &chapter_id).await })
        })
        .await;

    let (manifest_chapter_id, images) = match chapter {
        Ok(chapter) => (chapter.id, chapter.images),
        Err(upstream_error) => match app
            .downloads
            .offline_manifest(&chapter_id)
            .await
            .map_err(ApiError::Cache)?
        {
            Some(manifest) => {
                tracing::debug!(chapter_id, "上游章节不可用，使用离线 manifest");
                (manifest.chapter_id, manifest.images)
            }
            None => return Err(ApiError::Jm(upstream_error)),
        },
    };

    let pages: Vec<_> = images
        .iter()
        .enumerate()
        .map(|(index, image)| {
            let name = page_name_from_image(image);
            PageInfo {
                index: index as u32,
                name,
                url: format!("/api/reader/{chapter_id}/pages/{index}?v={READER_CACHE_VERSION}"),
            }
        })
        .collect();
    let prewarm_count = MANIFEST_PREWARM_COUNT.min(pages.len() as u32);

    prewarm_pages(app, chapter_id.clone(), 0, prewarm_count);

    Ok(Json(ManifestResponse {
        chapter_id: manifest_chapter_id,
        pages,
    }))
}

/// GET /api/reader/:chapter_id/pages/:page
pub async fn get_page(
    State(app): State<AppState>,
    Path((chapter_id, page)): Path<(String, u32)>,
) -> Result<Response, ApiError> {
    validate_chapter_id(&chapter_id)?;
    let image = materialize_page(&app, &chapter_id, page).await?;
    prewarm_pages(app, chapter_id, page.saturating_add(1), PAGE_PREWARM_COUNT);
    Ok(image_response(image.content_type, image.data))
}

struct MaterializedImage {
    content_type: &'static str,
    data: Vec<u8>,
}

async fn materialize_page(
    app: &AppState,
    chapter_id: &str,
    page: u32,
) -> Result<MaterializedImage, ApiError> {
    let materialize_started = Instant::now();
    let comic_id = validate_chapter_id(chapter_id)?;
    let cache_key = reader_page_cache_key(chapter_id, page as usize);

    if let Some(cached) = app
        .cache
        .get_reader_page(&cache_key)
        .await
        .map_err(ApiError::Cache)?
    {
        tracing::debug!(
            chapter_id,
            page,
            elapsed_ms = materialize_started.elapsed().as_millis(),
            bytes = cached.data.len(),
            "阅读页命中缓存"
        );
        return Ok(MaterializedImage {
            content_type: cached.format.content_type(),
            data: cached.data,
        });
    }

    let materialize_lock = page_materialize_lock(&cache_key).await;
    let lock_started = Instant::now();
    let _guard = materialize_lock.lock().await;
    let lock_wait_ms = lock_started.elapsed().as_millis();

    if let Some(cached) = app
        .cache
        .get_reader_page(&cache_key)
        .await
        .map_err(ApiError::Cache)?
    {
        tracing::debug!(
            chapter_id,
            page,
            lock_wait_ms,
            elapsed_ms = materialize_started.elapsed().as_millis(),
            bytes = cached.data.len(),
            "阅读页等待并复用物化结果"
        );
        return Ok(MaterializedImage {
            content_type: cached.format.content_type(),
            data: cached.data,
        });
    }

    if let Some(offline) = app
        .downloads
        .offline_page(chapter_id, page as usize)
        .await
        .map_err(ApiError::Cache)?
    {
        tracing::debug!(
            chapter_id,
            page,
            lock_wait_ms,
            elapsed_ms = materialize_started.elapsed().as_millis(),
            bytes = offline.data.len(),
            "阅读页命中离线下载"
        );
        return Ok(MaterializedImage {
            content_type: offline.format.content_type(),
            data: offline.data,
        });
    }

    let request_chapter_id = chapter_id.to_string();
    let chapter_started = Instant::now();
    let chapter = app
        .jm_request(move |client, endpoint| {
            let chapter_id = request_chapter_id.clone();
            Box::pin(async move { client.get_chapter(endpoint, &chapter_id).await })
        })
        .await?;
    let chapter_fetch_ms = chapter_started.elapsed().as_millis();

    let image_path = chapter
        .images
        .get(page as usize)
        .ok_or_else(|| ApiError::NotFound("Page index out of range".into()))?;

    let page_name = page_name_from_image(image_path);

    let download_started = Instant::now();
    let image_data = download_page_image(app, chapter_id, image_path).await?;
    let download_ms = download_started.elapsed().as_millis();

    let image_process_started = Instant::now();
    let prepared = prepare_page_image(image_data, comic_id, page_name)
        .await
        .map_err(|error| ApiError::ImageProcessing(error.to_string()))?;
    let image_process_ms = image_process_started.elapsed().as_millis();
    let cache_write_started = Instant::now();
    app.cache
        .put_reader_page(&cache_key, prepared.format, &prepared.data)
        .await
        .map_err(ApiError::Cache)?;
    tracing::debug!(
        chapter_id,
        page,
        lock_wait_ms,
        chapter_fetch_ms,
        download_ms,
        image_process_ms,
        cache_write_ms = cache_write_started.elapsed().as_millis(),
        elapsed_ms = materialize_started.elapsed().as_millis(),
        bytes = prepared.data.len(),
        decoded = prepared.decoded,
        "阅读页物化完成"
    );

    Ok(MaterializedImage {
        content_type: prepared.format.content_type(),
        data: prepared.data,
    })
}

fn prewarm_pages(app: AppState, chapter_id: String, start_page: u32, count: u32) {
    for page in start_page..start_page.saturating_add(count) {
        let app = app.clone();
        let chapter_id = chapter_id.clone();
        tokio::spawn(async move {
            let Ok(_permit) = PAGE_PREWARM_SEMAPHORE.clone().acquire_owned().await else {
                return;
            };

            if let Err(error) = materialize_page(&app, &chapter_id, page).await {
                if !matches!(error, ApiError::NotFound(_)) {
                    tracing::debug!(chapter_id, page, error = ?error, "reader page prewarm failed");
                }
            }
        });
    }
}

async fn page_materialize_lock(cache_key: &str) -> Arc<Mutex<()>> {
    let mut locks = PAGE_MATERIALIZE_LOCKS.lock().await;
    locks
        .entry(cache_key.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

fn validate_chapter_id(chapter_id: &str) -> Result<u32, ApiError> {
    chapter_id
        .parse::<u32>()
        .map_err(|_| ApiError::BadRequest("Chapter id must be numeric".into()))
}

async fn download_page_image(
    app: &AppState,
    chapter_id: &str,
    image_path: &str,
) -> crate::jm::JmResult<Vec<u8>> {
    let (endpoint, img_host) =
        request_with_failover(&app.jm, &app.endpoints, |client, endpoint| {
            Box::pin(client.get_img_host(endpoint))
        })
        .await?;
    let image_url = format!("{img_host}/media/photos/{chapter_id}/{image_path}");

    match app.jm.download_image(&image_url).await {
        Ok(data) => Ok(data),
        Err(_) => {
            invalidate_img_host(&endpoint).await;
            let (_, refreshed_host) =
                request_with_failover(&app.jm, &app.endpoints, |client, endpoint| {
                    Box::pin(client.get_img_host(endpoint))
                })
                .await?;
            let refreshed_url = format!("{refreshed_host}/media/photos/{chapter_id}/{image_path}");
            app.jm.download_image(&refreshed_url).await
        }
    }
}

fn image_response(content_type: &'static str, body: Vec<u8>) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        body,
    )
        .into_response()
}

// Error handling
#[derive(Debug)]
pub enum ApiError {
    Jm(crate::jm::JmError),
    BadRequest(String),
    ImageProcessing(String),
    Cache(anyhow::Error),
    NotFound(String),
}

impl From<crate::jm::JmError> for ApiError {
    fn from(err: crate::jm::JmError) -> Self {
        Self::Jm(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Jm(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::ImageProcessing(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Cache(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
