use crate::{http_error::HttpError, AppState};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct AccessGateConfig {
    enabled: bool,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    success: bool,
}

pub async fn config(State(app): State<AppState>) -> Json<AccessGateConfig> {
    Json(AccessGateConfig {
        enabled: app.access_gate.enabled(),
    })
}

pub async fn login(
    State(app): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, HttpError> {
    let valid = app.access_gate.verify(payload.password.as_bytes());

    if valid {
        return Ok(Json(LoginResponse { success: true }));
    }

    Err(HttpError::new(
        StatusCode::UNAUTHORIZED,
        "访问密码错误",
        false,
    ))
}
