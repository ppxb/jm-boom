use crate::{endpoint::EndpointState, AppState};
use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct EndpointSelection {
    endpoint: Option<String>,
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
