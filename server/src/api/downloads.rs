use crate::{
    download::{DownloadTaskList, EnqueueDownload},
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

type ApiResult = Result<Json<DownloadTaskList>, (StatusCode, String)>;

pub async fn enqueue(
    State(app): State<AppState>,
    Json(payload): Json<EnqueueDownload>,
) -> ApiResult {
    app.downloads
        .enqueue(payload)
        .await
        .map(Json)
        .map_err(internal_error)
}

pub async fn list(State(app): State<AppState>) -> Json<DownloadTaskList> {
    Json(app.downloads.list().await)
}

pub async fn pause(State(app): State<AppState>, Path(task_id): Path<String>) -> ApiResult {
    app.downloads
        .pause(&task_id)
        .await
        .map(Json)
        .map_err(internal_error)
}

pub async fn resume(State(app): State<AppState>, Path(task_id): Path<String>) -> ApiResult {
    app.downloads
        .resume(&task_id)
        .await
        .map(Json)
        .map_err(internal_error)
}

pub async fn cancel(State(app): State<AppState>, Path(task_id): Path<String>) -> ApiResult {
    app.downloads
        .cancel(&task_id)
        .await
        .map(Json)
        .map_err(internal_error)
}

pub async fn remove(State(app): State<AppState>, Path(task_id): Path<String>) -> ApiResult {
    app.downloads
        .remove(&task_id)
        .await
        .map(Json)
        .map_err(internal_error)
}

fn internal_error(error: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
