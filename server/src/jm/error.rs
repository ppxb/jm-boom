use crate::http_error::HttpError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

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

    #[error("Upstream image is too large ({actual_bytes} bytes, limit {limit_bytes} bytes)")]
    ImageTooLarge { actual_bytes: u64, limit_bytes: u64 },

    #[error("{0}")]
    Other(String),
}

impl JmError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Network(_) | Self::Http(_) | Self::Empty)
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Network(_) | Self::Http(_) | Self::ImageTooLarge { .. } => {
                StatusCode::BAD_GATEWAY
            }
            Self::Api(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for JmError {
    fn into_response(self) -> Response {
        let retryable = self.is_retryable();
        HttpError::new(self.status_code(), self.to_string(), retryable).into_response()
    }
}
