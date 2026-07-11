use crate::AppState;
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
        enabled: app.access_password.is_some(),
    })
}

pub async fn login(
    State(app): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<serde_json::Value>)> {
    let valid = app.access_password.as_deref().is_none_or(|expected| {
        constant_time_equals(expected.as_bytes(), payload.password.as_bytes())
    });

    if valid {
        return Ok(Json(LoginResponse { success: true }));
    }

    Err((
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": "访问密码错误" })),
    ))
}

fn constant_time_equals(expected: &[u8], actual: &[u8]) -> bool {
    let mut difference = expected.len() ^ actual.len();
    let length = expected.len().max(actual.len());
    for index in 0..length {
        let left = expected.get(index).copied().unwrap_or_default();
        let right = actual.get(index).copied().unwrap_or_default();
        difference |= usize::from(left ^ right);
    }
    difference == 0
}
