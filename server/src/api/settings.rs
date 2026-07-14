use crate::{cache::CacheStats, endpoint::EndpointState, http_error::HttpError, AppState};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct EndpointSelection {
    endpoint: Option<String>,
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
