use crate::{cache::CacheStats, endpoint::EndpointState, AppState};
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
    Json(app.endpoints.state().await)
}

pub async fn probe_endpoints(State(app): State<AppState>) -> Json<EndpointState> {
    Json(app.endpoints.refresh().await)
}

pub async fn set_endpoint(
    State(app): State<AppState>,
    Json(payload): Json<EndpointSelection>,
) -> Result<Json<EndpointState>, (StatusCode, String)> {
    app.endpoints
        .set_selected(payload.endpoint)
        .await
        .map(Json)
        .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))
}

pub async fn get_system_info(
    State(app): State<AppState>,
) -> Result<Json<SystemInfo>, (StatusCode, String)> {
    system_info(&app).await.map(Json)
}

pub async fn clear_cache(
    State(app): State<AppState>,
) -> Result<Json<SystemInfo>, (StatusCode, String)> {
    app.cache.clear().await.map_err(internal_error)?;
    system_info(&app).await.map(Json)
}

async fn system_info(app: &AppState) -> Result<SystemInfo, (StatusCode, String)> {
    let cache = app.cache.stats().await.map_err(internal_error)?;

    Ok(SystemInfo {
        server_version: env!("CARGO_PKG_VERSION"),
        cache,
    })
}

fn internal_error(error: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
