use crate::cache::READER_CACHE_VERSION;
use crate::http_error::HttpError;
use crate::image_work::ImageWorkPriority;
use crate::page_materializer::PageMaterializeError;
use crate::reader::page_name_from_image;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::{sync::Arc, time::Instant};
use tokio::sync::Semaphore;

const MANIFEST_PREWARM_COUNT: u32 = 5;
const PAGE_PREWARM_COUNT: u32 = 3;
const PAGE_PREWARM_CONCURRENCY: usize = 2;

static PAGE_PREWARM_SEMAPHORE: std::sync::LazyLock<Arc<Semaphore>> =
    std::sync::LazyLock::new(|| Arc::new(Semaphore::new(PAGE_PREWARM_CONCURRENCY)));

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
    let chapter = app.reader.chapter(&chapter_id).await;

    let offline_manifest = if chapter.is_err() {
        app.reader
            .offline_manifest(&chapter_id)
            .await
            .map_err(|error| ApiError::Cache(anyhow::anyhow!(error.to_string())))?
            .map(|manifest| (manifest.id, manifest.images))
    } else {
        None
    };
    if offline_manifest.is_some() {
        tracing::debug!(chapter_id, "上游章节不可用，使用离线 manifest");
    }
    let (manifest_chapter_id, images) = resolve_manifest_data(chapter, offline_manifest)?;

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
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    validate_chapter_id(&chapter_id)?;
    let priority = request_priority(&headers);
    let image = materialize_page(&app, &chapter_id, page, priority).await?;
    if should_expand_prewarm_window(priority) {
        prewarm_pages(app, chapter_id, page.saturating_add(1), PAGE_PREWARM_COUNT);
    }
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
    priority: ImageWorkPriority,
) -> Result<MaterializedImage, ApiError> {
    let materialize_started = Instant::now();
    let comic_id = validate_chapter_id(chapter_id)?;

    if let Some(cached) = app
        .reader
        .cached_page(chapter_id, page as usize)
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

    if let Some(offline) = app
        .reader
        .offline_page(chapter_id, page as usize)
        .await
        .map_err(|error| ApiError::Cache(anyhow::anyhow!(error.to_string())))?
    {
        tracing::debug!(
            chapter_id,
            page,
            elapsed_ms = materialize_started.elapsed().as_millis(),
            bytes = offline.data.len(),
            "阅读页命中离线下载"
        );
        return Ok(MaterializedImage {
            content_type: offline.format.content_type(),
            data: offline.data,
        });
    }

    let materialized = app
        .reader
        .materialize(chapter_id, page as usize, comic_id, priority)
        .await
        .map_err(ApiError::from)?;
    tracing::debug!(
        chapter_id,
        page,
        elapsed_ms = materialize_started.elapsed().as_millis(),
        bytes = materialized.data.len(),
        "阅读页物化完成"
    );

    Ok(MaterializedImage {
        content_type: materialized.format.content_type(),
        data: materialized.data,
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

            if let Err(error) =
                materialize_page(&app, &chapter_id, page, ImageWorkPriority::Prefetch).await
            {
                if !matches!(error, ApiError::NotFound(_)) {
                    tracing::debug!(chapter_id, page, error = ?error, "reader page prewarm failed");
                }
            }
        });
    }
}

fn request_priority(headers: &HeaderMap) -> ImageWorkPriority {
    let is_prefetch = headers
        .get("x-jm-boom-image-priority")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("prefetch"));

    if is_prefetch {
        ImageWorkPriority::Prefetch
    } else {
        ImageWorkPriority::Foreground
    }
}

fn should_expand_prewarm_window(priority: ImageWorkPriority) -> bool {
    priority == ImageWorkPriority::Foreground
}

fn resolve_manifest_data(
    chapter: crate::jm::JmResult<crate::domain::reader::ChapterManifest>,
    offline_manifest: Option<(String, Vec<String>)>,
) -> Result<(String, Vec<String>), ApiError> {
    match chapter {
        Ok(chapter) => Ok((chapter.id, chapter.images)),
        Err(error) => offline_manifest.ok_or(ApiError::Jm(error)),
    }
}

fn validate_chapter_id(chapter_id: &str) -> Result<u32, ApiError> {
    chapter_id
        .parse::<u32>()
        .map_err(|_| ApiError::BadRequest("Chapter id must be numeric".into()))
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
    Cache(anyhow::Error),
    NotFound(String),
}

impl From<crate::jm::JmError> for ApiError {
    fn from(err: crate::jm::JmError) -> Self {
        Self::Jm(err)
    }
}

impl From<PageMaterializeError> for ApiError {
    fn from(error: PageMaterializeError) -> Self {
        match error {
            PageMaterializeError::Upstream(error) => Self::Jm(error),
            PageMaterializeError::PageNotFound => Self::NotFound("Page index out of range".into()),
            PageMaterializeError::Cancelled => {
                Self::Cache(anyhow::anyhow!("Page materialization was cancelled"))
            }
            PageMaterializeError::Internal(error) => Self::Cache(error),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message, retryable) = match self {
            ApiError::Jm(error) => {
                let status = error.status_code();
                let retryable = error.is_retryable();
                (status, error.to_string(), retryable)
            }
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message, false),
            ApiError::Cache(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string(), false),
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, message, false),
        };

        HttpError::new(status, message, retryable).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_manifest_data, should_expand_prewarm_window, ApiError};
    use crate::{domain::reader::ChapterManifest, image_work::ImageWorkPriority, jm::JmError};

    #[test]
    fn prefetch_requests_do_not_expand_the_server_prewarm_window() {
        assert!(should_expand_prewarm_window(ImageWorkPriority::Foreground));
        assert!(!should_expand_prewarm_window(ImageWorkPriority::Prefetch));
    }

    #[test]
    fn falls_back_to_offline_manifest_only_when_upstream_fails() {
        let offline = Some(("1001".into(), vec!["offline-001.jpg".into()]));
        let fallback = resolve_manifest_data(
            Err(JmError::Network("upstream unavailable".into())),
            offline.clone(),
        )
        .expect("offline manifest should be used");
        assert_eq!(fallback, offline.expect("offline manifest tuple"));

        let upstream = resolve_manifest_data(
            Ok(ChapterManifest {
                id: "1001".into(),
                images: vec!["upstream-001.jpg".into()],
            }),
            Some(("1001".into(), vec!["offline-001.jpg".into()])),
        )
        .expect("upstream manifest should be preferred");
        assert_eq!(upstream.1, vec!["upstream-001.jpg"]);

        assert!(matches!(
            resolve_manifest_data(Err(JmError::Empty), None),
            Err(ApiError::Jm(JmError::Empty))
        ));
    }

    #[tokio::test]
    async fn serializes_reader_not_found_error_contract() {
        use axum::{body::to_bytes, http::StatusCode, response::IntoResponse};

        let response = ApiError::NotFound("Page index out of range".into()).into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read reader error response body");
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body)
                .expect("parse reader error response"),
            serde_json::json!({
                "error": "Page index out of range",
                "retryable": false
            })
        );
    }
}
