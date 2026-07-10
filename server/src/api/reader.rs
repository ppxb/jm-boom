use crate::cache::reader_page_cache_key;
use crate::endpoint::request_with_failover;
use crate::jm::invalidate_img_host;
use crate::reader::{decode_scrambled_image, encode_webp, needs_decoding, page_name_from_image};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

const PAGE_PREWARM_COUNT: u32 = 2;

static PAGE_MATERIALIZE_LOCKS: Lazy<Mutex<HashMap<String, Arc<Mutex<()>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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
        .await?;

    let pages = chapter
        .images
        .iter()
        .enumerate()
        .map(|(index, image)| {
            let name = page_name_from_image(image);
            PageInfo {
                index: index as u32,
                name,
                url: format!("/api/reader/{chapter_id}/pages/{index}"),
            }
        })
        .collect();

    Ok(Json(ManifestResponse {
        chapter_id: chapter.id,
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
    prewarm_pages(app, chapter_id, page);
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
    let comic_id = validate_chapter_id(chapter_id)?;
    let cache_key = reader_page_cache_key(chapter_id, page as usize);

    if let Some(cached) = app.cache.get(&cache_key).await.map_err(ApiError::Cache)? {
        return Ok(MaterializedImage {
            content_type: "image/webp",
            data: cached,
        });
    }

    let materialize_lock = page_materialize_lock(&cache_key).await;
    let _guard = materialize_lock.lock().await;

    if let Some(cached) = app.cache.get(&cache_key).await.map_err(ApiError::Cache)? {
        return Ok(MaterializedImage {
            content_type: "image/webp",
            data: cached,
        });
    }

    let request_chapter_id = chapter_id.to_string();
    let chapter = app
        .jm_request(move |client, endpoint| {
            let chapter_id = request_chapter_id.clone();
            Box::pin(async move { client.get_chapter(endpoint, &chapter_id).await })
        })
        .await?;

    let image_path = chapter
        .images
        .get(page as usize)
        .ok_or_else(|| ApiError::NotFound("Page index out of range".into()))?;

    let file_name = file_name(image_path);
    let page_name = page_name_from_image(image_path);
    let is_gif = file_name.to_ascii_lowercase().ends_with(".gif");

    if is_gif {
        if let Some(cached) = app
            .cache
            .get_gif(&cache_key)
            .await
            .map_err(ApiError::Cache)?
        {
            return Ok(MaterializedImage {
                content_type: "image/gif",
                data: cached,
            });
        }
    }

    let image_data = download_page_image(&app, &chapter_id, image_path).await?;

    if is_gif {
        app.cache
            .put_gif(&cache_key, &image_data)
            .await
            .map_err(ApiError::Cache)?;
        return Ok(MaterializedImage {
            content_type: "image/gif",
            data: image_data,
        });
    }

    let webp_data = tokio::task::spawn_blocking(move || {
        let original = image::load_from_memory(&image_data).map_err(|error| error.to_string())?;
        let rgb = if needs_decoding(comic_id, &page_name, false) {
            decode_scrambled_image(original, comic_id, &page_name)
        } else {
            original.to_rgb8()
        };
        Ok::<_, String>(encode_webp(&rgb))
    })
    .await
    .map_err(|error| ApiError::ImageProcessing(error.to_string()))?
    .map_err(ApiError::ImageProcessing)?;
    app.cache
        .put(&cache_key, &webp_data)
        .await
        .map_err(ApiError::Cache)?;

    Ok(MaterializedImage {
        content_type: "image/webp",
        data: webp_data,
    })
}

fn prewarm_pages(app: AppState, chapter_id: String, current_page: u32) {
    for page in current_page + 1..=current_page.saturating_add(PAGE_PREWARM_COUNT) {
        let app = app.clone();
        let chapter_id = chapter_id.clone();
        tokio::spawn(async move {
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

fn file_name(url: &str) -> String {
    url.split('/')
        .last()
        .and_then(|s| s.split('?').next())
        .unwrap_or("page")
        .to_string()
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
