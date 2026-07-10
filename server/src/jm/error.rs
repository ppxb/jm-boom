use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

pub type JmResult<T> = Result<T, JmError>;

#[derive(Debug, thiserror::Error)]
pub enum JmError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Decryption failed: {0}")]
    Decrypt(String),

    #[error("Decoding failed: {0}")]
    Decode(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Invalid payload: {0}")]
    Payload(String),

    #[error("Empty response")]
    Empty,

    #[error("Missing data in response")]
    MissingData,

    #[error("{0}")]
    Other(String),
}

impl JmError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Network(_) | Self::Http(_) | Self::Empty)
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    retryable: bool,
}

impl IntoResponse for JmError {
    fn into_response(self) -> Response {
        let retryable = self.is_retryable();
        let status = match &self {
            Self::Network(_) | Self::Http(_) => StatusCode::BAD_GATEWAY,
            Self::Api(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (
            status,
            Json(ErrorResponse {
                error: self.to_string(),
                retryable,
            }),
        )
            .into_response()
    }
}
