use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug)]
pub struct HttpError {
    status: StatusCode,
    message: String,
    retryable: bool,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    retryable: bool,
}

impl HttpError {
    pub fn new(status: StatusCode, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            status,
            message: message.into(),
            retryable,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message, false)
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.message,
                retryable: self.retryable,
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::HttpError;
    use axum::{body::to_bytes, http::StatusCode, response::IntoResponse};

    #[tokio::test]
    async fn serializes_the_shared_json_error_contract() {
        let response =
            HttpError::new(StatusCode::BAD_GATEWAY, "upstream failed", true).into_response();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read error response body");
        let value: serde_json::Value =
            serde_json::from_slice(&body).expect("parse error response body");
        assert_eq!(
            value,
            serde_json::json!({
                "error": "upstream failed",
                "retryable": true,
            })
        );
    }
}
