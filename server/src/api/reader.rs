use crate::AppState;
use crate::jm::JmClient;
use crate::reader::{descramble_image, encode_webp};
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ManifestQuery {
    #[serde(default)]
    endpoint: Option<String>,
}

#[derive(Deserialize)]
pub struct PageQuery {
    #[serde(default)]
    endpoint: Option<String>,
}

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

const DEFAULT_ENDPOINT: &str = "https://www.cdnhjk.net";

/// GET /api/reader/:chapter_id/manifest
pub async fn get_manifest(
    State(_app): State<AppState>,
    Path(chapter_id): Path<String>,
    Query(params): Query<ManifestQuery>,
) -> Result<Json<ManifestResponse>, ApiError> {
    let client = JmClient::new()?;
    let endpoint = params.endpoint.as_deref().unwrap_or(DEFAULT_ENDPOINT);
    let chapter = client.get_chapter(endpoint, &chapter_id).await?;

    let pages = chapter
        .images
        .iter()
        .enumerate()
        .map(|(index, image)| {
            let name = extract_filename(image);
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
    State(_app): State<AppState>,
    Path((chapter_id, page)): Path<(String, u32)>,
    Query(params): Query<PageQuery>,
) -> Result<Response, ApiError> {
    let client = JmClient::new()?;
    let endpoint = params.endpoint.as_deref().unwrap_or(DEFAULT_ENDPOINT);

    // Get chapter manifest
    let chapter = client.get_chapter(endpoint, &chapter_id).await?;

    let image_path = chapter
        .images
        .get(page as usize)
        .ok_or_else(|| ApiError::NotFound("Page index out of range".into()))?;

    // Get img_host
    let img_host = client.get_img_host(endpoint).await?;

    // Build full image URL
    let image_url = format!("{img_host}/media/photos/{chapter_id}/{image_path}");

    // Download image
    let image_data = client.download_image(&image_url).await?;

    // Descramble image
    let descrambled = descramble_image(&image_data, &chapter_id)
        .map_err(|e| ApiError::ImageProcessing(e.to_string()))?;

    // Encode as WebP
    let webp_data =
        encode_webp(&descrambled).map_err(|e| ApiError::ImageProcessing(e.to_string()))?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/webp")],
        webp_data,
    )
        .into_response())
}

fn extract_filename(url: &str) -> String {
    url.split('/')
        .last()
        .and_then(|s| s.split('?').next())
        .and_then(|s| s.rsplit_once('.').map(|(name, _)| name))
        .unwrap_or("page")
        .to_string()
}

// Error handling
#[derive(Debug)]
pub enum ApiError {
    Jm(crate::jm::JmError),
    ImageProcessing(String),
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
            ApiError::ImageProcessing(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
