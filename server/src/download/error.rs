use crate::{http_error::HttpError, jm::JmError};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub type DownloadResult<T> = Result<T, DownloadError>;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("{0}")]
    InvalidRequest(String),
    #[error("Download task not found")]
    NotFound,
    #[error("Cannot {operation} a download task in {status} state")]
    InvalidState {
        operation: &'static str,
        status: String,
    },
    #[error(transparent)]
    Upstream(#[from] JmError),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for DownloadError {
    fn into_response(self) -> Response {
        let retryable = matches!(&self, Self::Upstream(error) if error.is_retryable());
        let status = match &self {
            Self::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InvalidState { .. } => StatusCode::CONFLICT,
            Self::Upstream(_) => StatusCode::BAD_GATEWAY,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        HttpError::new(status, self.to_string(), retryable).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::DownloadError;
    use axum::{body::to_bytes, http::StatusCode, response::IntoResponse};

    #[tokio::test]
    async fn serializes_not_found_error_contract() {
        let response = DownloadError::NotFound.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read download error response body");
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body)
                .expect("parse download error response"),
            serde_json::json!({
                "error": "Download task not found",
                "retryable": false
            })
        );
    }
}
