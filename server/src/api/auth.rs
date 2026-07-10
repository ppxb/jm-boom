use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
}

pub async fn login(Json(_payload): Json<LoginRequest>) -> Json<LoginResponse> {
    // TODO: 实现登录
    Json(LoginResponse {
        token: "test_token".to_string(),
    })
}

pub async fn get_session() -> Json<Option<String>> {
    // TODO: 实现会话获取
    Json(None)
}
