use crate::cache::{page_image_id, reader_page_cache_key, READER_CACHE_VERSION};
use crate::http_error::HttpError;
use crate::image_work::ImageWorkPriority;
use crate::page_materializer::PageMaterializeError;
use crate::reader::page_name_from_image;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Instant};

mod prewarm;
use prewarm::PagePrewarmQueue;

const MANIFEST_PREWARM_COUNT: u32 = 5;
const PAGE_PREWARM_COUNT: u32 = 3;
static PAGE_PREWARM_QUEUE: std::sync::LazyLock<Arc<PagePrewarmQueue>> =
    std::sync::LazyLock::new(|| Arc::new(PagePrewarmQueue::default()));

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}

#[derive(Deserialize)]
pub struct PageQuery {
    v: u8,
    image: String,
}

fn validate_page_query(query: &PageQuery) -> Result<(), ApiError> {
    if query.v != READER_CACHE_VERSION {
        return Err(ApiError::BadRequest(
            "Reader cache version changed; reload the chapter".into(),
        ));
    }
    if query.image.len() != 32
        || !query
            .image
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ApiError::BadRequest("Invalid reader image identity".into()));
    }
    Ok(())
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
    let dimensions = app
        .reader
        .page_dimensions(&chapter_id)
        .await
        .map_err(ApiError::Cache)?;

    let pages = manifest_pages(&chapter_id, &images, &dimensions);
    let prewarm_count = MANIFEST_PREWARM_COUNT.min(pages.len() as u32);

    prewarm_pages(app, chapter_id.clone(), 0, prewarm_count);

    Ok(Json(ManifestResponse {
        chapter_id: manifest_chapter_id,
        pages,
    }))
}

fn manifest_pages(
    chapter_id: &str,
    images: &[String],
    dimensions: &std::collections::HashMap<String, crate::reader::ImageDimensions>,
) -> Vec<PageInfo> {
    images
        .iter()
        .enumerate()
        .map(|(index, image)| {
            let name = page_name_from_image(image);
            let image_id = page_image_id(image);
            let size = dimensions.get(&reader_page_cache_key(chapter_id, &image_id));
            PageInfo {
                index: index as u32,
                name,
                url: format!("/api/reader/{chapter_id}/pages/{index}?v={READER_CACHE_VERSION}&image={image_id}"),
                width: size.map(|size| size.width),
                height: size.map(|size| size.height),
            }
        })
        .collect()
}

/// GET /api/reader/:chapter_id/pages/:page
pub async fn get_page(
    State(app): State<AppState>,
    Path((chapter_id, page)): Path<(String, u32)>,
    Query(query): Query<PageQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    validate_page_query(&query)?;
    let priority = request_priority(&headers);
    let image = materialize_page(&app, &chapter_id, page, &query.image, priority).await?;
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
    image_id: &str,
    priority: ImageWorkPriority,
) -> Result<MaterializedImage, ApiError> {
    let materialize_started = Instant::now();
    // HTTP requests and background prewarm jobs both pass this check before reading data.
    let comic_id = validate_chapter_id(chapter_id)?;

    if let Some(cached) = app
        .reader
        .cached_page(chapter_id, image_id)
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
        .offline_page(chapter_id, image_id)
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
        .materialize(chapter_id, image_id, comic_id, priority)
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
        PAGE_PREWARM_QUEUE.enqueue(chapter_id.clone(), page, async move {
            let result = async {
                let chapter = app.reader.chapter(&chapter_id).await?;
                let image_path = chapter
                    .images
                    .get(page as usize)
                    .ok_or_else(|| ApiError::NotFound("Page index out of range".into()))?;
                materialize_page(
                    &app,
                    &chapter_id,
                    page,
                    &page_image_id(image_path),
                    ImageWorkPriority::Prefetch,
                )
                .await
            }
            .await;
            if let Err(error) = result {
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
    use super::{
        resolve_manifest_data, should_expand_prewarm_window, validate_chapter_id, ApiError,
    };
    use crate::api::tests::assert_bad_request;
    use crate::{domain::reader::ChapterManifest, image_work::ImageWorkPriority, jm::JmError};

    #[test]
    fn manifest_urls_and_sizes_follow_the_image_instead_of_its_position() {
        use crate::{
            cache::{page_image_id, reader_page_cache_key},
            reader::ImageDimensions,
        };
        let sizes = std::collections::HashMap::from([(
            reader_page_cache_key("1001", &page_image_id("A.png")),
            ImageDimensions {
                width: 10,
                height: 20,
            },
        )]);
        let old = super::manifest_pages("1001", &["A.png".into(), "B.png".into()], &sizes);
        let new = super::manifest_pages(
            "1001",
            &["X.png".into(), "A.png".into(), "B.png".into()],
            &sizes,
        );
        assert_ne!(old[0].url, new[0].url);
        assert_eq!(
            old[0].url.split("&image=").last(),
            new[1].url.split("&image=").last()
        );
        assert_eq!(new[0].width, None);
        assert_eq!((new[1].width, new[1].height), (Some(10), Some(20)));
        assert!(new[0].url.contains("?v=4&image="));
    }

    #[test]
    fn rejects_old_or_invalid_page_identities() {
        let image = crate::cache::page_image_id("A.png");
        assert!(super::validate_page_query(&super::PageQuery {
            v: 4,
            image: image.clone()
        })
        .is_ok());
        assert!(super::validate_page_query(&super::PageQuery { v: 3, image }).is_err());
        assert!(super::validate_page_query(&super::PageQuery {
            v: 4,
            image: "../0".into()
        })
        .is_err());
    }

    #[test]
    fn prefetch_requests_do_not_expand_the_server_prewarm_window() {
        assert!(should_expand_prewarm_window(ImageWorkPriority::Foreground));
        assert!(!should_expand_prewarm_window(ImageWorkPriority::Prefetch));
    }

    #[tokio::test]
    async fn preserves_the_readers_bounded_integer_parser() {
        for (id, expected) in [
            ("0", 0),
            ("000123", 123),
            ("+123", 123),
            ("4294967295", u32::MAX),
        ] {
            assert_eq!(validate_chapter_id(id).unwrap(), expected);
        }
        for id in ["", "-1", "4294967296", "1.0", "１２", " 1", "1 "] {
            assert_bad_request(
                validate_chapter_id(id).unwrap_err(),
                "Chapter id must be numeric",
            )
            .await;
        }
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
