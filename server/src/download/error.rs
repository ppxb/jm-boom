use crate::jm::JmError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
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

        (
            status,
            Json(serde_json::json!({
                "error": self.to_string(),
                "retryable": retryable,
            })),
        )
            .into_response()
    }
}
