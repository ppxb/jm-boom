use crate::{
    download::{DownloadError, DownloadTaskList, DownloadedChapterList, EnqueueDownload},
    AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};

type ApiResult<T> = Result<Json<T>, DownloadError>;

pub async fn enqueue(
    State(app): State<AppState>,
    Json(payload): Json<EnqueueDownload>,
) -> ApiResult<DownloadTaskList> {
    app.downloads.enqueue(payload).await.map(Json)
}

pub async fn list(State(app): State<AppState>) -> Json<DownloadTaskList> {
    Json(app.downloads.list().await)
}

pub async fn downloaded_chapters(State(app): State<AppState>) -> ApiResult<DownloadedChapterList> {
    app.downloads.downloaded_chapters().await.map(Json)
}

pub async fn pause(
    State(app): State<AppState>,
    Path(task_id): Path<String>,
) -> ApiResult<DownloadTaskList> {
    app.downloads.pause(&task_id).await.map(Json)
}

pub async fn resume(
    State(app): State<AppState>,
    Path(task_id): Path<String>,
) -> ApiResult<DownloadTaskList> {
    app.downloads.resume(&task_id).await.map(Json)
}

pub async fn cancel(
    State(app): State<AppState>,
    Path(task_id): Path<String>,
) -> ApiResult<DownloadTaskList> {
    app.downloads.cancel(&task_id).await.map(Json)
}

pub async fn remove(
    State(app): State<AppState>,
    Path(task_id): Path<String>,
) -> ApiResult<DownloadTaskList> {
    app.downloads.remove(&task_id).await.map(Json)
}
