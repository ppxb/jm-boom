use crate::application::CoverServiceError;
use crate::http_error::HttpError;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
pub async fn get_cover(
    State(app): State<AppState>,
    Path(comic_id): Path<String>,
) -> Result<Response, CoverError> {
    validate_comic_id(&comic_id)?;
    app.covers
        .get_cover(&comic_id)
        .await
        .map(cover_response)
        .map_err(CoverError::from)
}

fn validate_comic_id(comic_id: &str) -> Result<(), CoverError> {
    if comic_id.is_empty() || !comic_id.chars().all(|character| character.is_ascii_digit()) {
        return Err(CoverError::BadRequest("Comic id must be numeric".into()));
    }

    Ok(())
}

fn cover_response(body: Vec<u8>) -> Response {
    let content_type = match image::guess_format(&body) {
        Ok(image::ImageFormat::Gif) => "image/gif",
        Ok(image::ImageFormat::Png) => "image/png",
        Ok(image::ImageFormat::WebP) => "image/webp",
        _ => "image/jpeg",
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (
                header::CACHE_CONTROL,
                "public, max-age=604800, stale-while-revalidate=86400",
            ),
        ],
        body,
    )
        .into_response()
}

#[derive(Debug)]
pub enum CoverError {
    Jm(crate::jm::JmError),
    BadRequest(String),
    Cache(anyhow::Error),
    Internal(String),
}

impl From<CoverServiceError> for CoverError {
    fn from(error: CoverServiceError) -> Self {
        match error {
            CoverServiceError::Jm(error) => Self::Jm(error),
            CoverServiceError::Cache(error) => Self::Cache(error),
            CoverServiceError::Internal(message) => Self::Internal(message),
        }
    }
}

impl IntoResponse for CoverError {
    fn into_response(self) -> Response {
        let (status, message, retryable) = match self {
            Self::Jm(error) => {
                let retryable = error.is_retryable();
                (StatusCode::BAD_GATEWAY, error.to_string(), retryable)
            }
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message, false),
            Self::Cache(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string(), false),
            Self::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message, false),
        };

        HttpError::new(status, message, retryable).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::validate_comic_id;

    #[test]
    fn validates_numeric_comic_ids() {
        assert!(validate_comic_id("1444583").is_ok());
        assert!(validate_comic_id("").is_err());
        assert!(validate_comic_id("../1444583").is_err());
    }
}
