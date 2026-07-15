use crate::{source::InstalledSource, AppState};
use axum::{extract::State, Json};

pub async fn list(State(app): State<AppState>) -> Json<Vec<InstalledSource>> {
    Json(app.sources.list().await)
}
