use crate::{
    download::{DownloadTaskList, DownloadedChapterList, EnqueueDownload},
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

type ApiResult<T> = Result<Json<T>, (StatusCode, String)>;

pub async fn enqueue(
    State(app): State<AppState>,
    Json(payload): Json<EnqueueDownload>,
) -> ApiResult<DownloadTaskList> {
    app.downloads
        .enqueue(payload)
        .await
        .map(Json)
        .map_err(internal_error)
}

pub async fn list(State(app): State<AppState>) -> Json<DownloadTaskList> {
    Json(app.downloads.list().await)
}

pub async fn downloaded_chapters(State(app): State<AppState>) -> ApiResult<DownloadedChapterList> {
    app.downloads
        .downloaded_chapters()
        .await
        .map(Json)
        .map_err(internal_error)
}

pub async fn pause(
    State(app): State<AppState>,
    Path(task_id): Path<String>,
) -> ApiResult<DownloadTaskList> {
    app.downloads
        .pause(&task_id)
        .await
        .map(Json)
        .map_err(internal_error)
}

pub async fn resume(
    State(app): State<AppState>,
    Path(task_id): Path<String>,
) -> ApiResult<DownloadTaskList> {
    app.downloads
        .resume(&task_id)
        .await
        .map(Json)
        .map_err(internal_error)
}

pub async fn cancel(
    State(app): State<AppState>,
    Path(task_id): Path<String>,
) -> ApiResult<DownloadTaskList> {
    app.downloads
        .cancel(&task_id)
        .await
        .map(Json)
        .map_err(internal_error)
}

pub async fn remove(
    State(app): State<AppState>,
    Path(task_id): Path<String>,
) -> ApiResult<DownloadTaskList> {
    app.downloads
        .remove(&task_id)
        .await
        .map(Json)
        .map_err(internal_error)
}

fn internal_error(error: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
