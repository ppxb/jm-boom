use crate::{
    application::AccountInput, cache::CacheStats, endpoint::EndpointState, http_error::HttpError,
    AppState,
};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct EndpointSelection {
    endpoint: Option<String>,
}

#[derive(Deserialize)]
pub struct AccountUpdate {
    username: String,
    password: Option<String>,
    #[serde(rename = "autoLogin")]
    auto_login: bool,
    #[serde(rename = "autoSignIn")]
    auto_sign_in: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    server_version: &'static str,
    cache: CacheStats,
}

pub async fn get_endpoints(State(app): State<AppState>) -> Json<EndpointState> {
    Json(app.settings.endpoints().await)
}

pub async fn get_account(State(app): State<AppState>) -> Json<crate::application::AccountState> {
    Json(app.account.state().await)
}

pub async fn update_account(
    State(app): State<AppState>,
    Json(payload): Json<AccountUpdate>,
) -> Result<Json<crate::application::AccountState>, crate::jm::JmError> {
    app.account
        .save_and_login(AccountInput {
            username: payload.username,
            password: payload.password,
            auto_login: payload.auto_login,
            auto_sign_in: payload.auto_sign_in,
        })
        .await
        .map(Json)
}

pub async fn clear_account(
    State(app): State<AppState>,
) -> Result<Json<crate::application::AccountState>, HttpError> {
    app.account.clear().await.map(Json).map_err(internal_error)
}

pub async fn probe_endpoints(State(app): State<AppState>) -> Json<EndpointState> {
    Json(app.settings.refresh_endpoints().await)
}

pub async fn set_endpoint(
    State(app): State<AppState>,
    Json(payload): Json<EndpointSelection>,
) -> Result<Json<EndpointState>, HttpError> {
    app.settings
        .set_endpoint(payload.endpoint)
        .await
        .map(Json)
        .map_err(|error| HttpError::new(StatusCode::BAD_REQUEST, error.to_string(), false))
}

pub async fn get_system_info(State(app): State<AppState>) -> Result<Json<SystemInfo>, HttpError> {
    system_info(&app).await.map(Json)
}

pub async fn clear_cache(State(app): State<AppState>) -> Result<Json<SystemInfo>, HttpError> {
    app.settings.clear_cache().await.map_err(internal_error)?;
    system_info(&app).await.map(Json)
}

async fn system_info(app: &AppState) -> Result<SystemInfo, HttpError> {
    let cache = app.settings.cache_stats().await.map_err(internal_error)?;

    Ok(SystemInfo {
        server_version: env!("CARGO_PKG_VERSION"),
        cache,
    })
}

fn internal_error(error: anyhow::Error) -> HttpError {
    HttpError::internal(error.to_string())
}
