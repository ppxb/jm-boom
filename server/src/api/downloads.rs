use crate::{
    download::{
        DownloadError, DownloadExport, DownloadTaskList, DownloadedChapterList, EnqueueDownload,
        ExportFormat,
    },
    AppState,
};
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderValue},
    response::{IntoResponse, Response},
    Json,
};

type ApiResult<T> = Result<Json<T>, DownloadError>;

#[derive(serde::Deserialize)]
pub struct ExportQuery {
    format: Option<String>,
}

pub async fn export(
    State(app): State<AppState>,
    Path(task_id): Path<String>,
    Query(query): Query<ExportQuery>,
) -> Result<Response, DownloadError> {
    let format = ExportFormat::parse(query.format.as_deref())?;
    let exported = app.downloads.export(&task_id, format).await?;
    export_response(exported)
}

struct ExportReader {
    file: tokio::fs::File,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl tokio::io::AsyncRead for ExportReader {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.get_mut().file).poll_read(cx, buf)
    }
}

fn export_response(exported: DownloadExport) -> Result<Response, DownloadError> {
    let reader = ExportReader {
        file: tokio::fs::File::from_std(exported.file),
        _permit: exported.permit,
    };
    let stream = tokio_util::io::ReaderStream::new(reader);
    let mut response = Body::from_stream(stream).into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(exported.format.content_type()),
    );
    headers.insert(header::CONTENT_LENGTH, HeaderValue::from(exported.size));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&attachment_filename(&exported.filename, exported.format))
            .map_err(anyhow::Error::from)?,
    );
    Ok(response)
}

fn attachment_filename(filename: &str, format: ExportFormat) -> String {
    let mut encoded = String::new();
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for byte in filename.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[(byte >> 4) as usize]));
            encoded.push(char::from(HEX[(byte & 15) as usize]));
        }
    }
    format!(
        "attachment; filename=\"download.{}\"; filename*=UTF-8''{encoded}",
        format.extension()
    )
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnqueueResponse {
    task_id: String,
    #[serde(flatten)]
    list: DownloadTaskList,
}

pub async fn enqueue(
    State(app): State<AppState>,
    Json(payload): Json<EnqueueDownload>,
) -> ApiResult<EnqueueResponse> {
    let (task_id, list) = app.downloads.enqueue(payload).await?;
    Ok(Json(EnqueueResponse { task_id, list }))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Seek, Write},
        sync::Arc,
    };
    use tokio::sync::Semaphore;

    #[test]
    fn enqueue_response_identifies_the_created_task_and_keeps_the_task_list_shape() {
        let response = EnqueueResponse {
            task_id: "new-task".into(),
            list: DownloadTaskList { tasks: Vec::new() },
        };
        assert_eq!(
            serde_json::to_value(response).unwrap(),
            serde_json::json!({ "taskId": "new-task", "tasks": [] })
        );
    }

    fn response(permits: &Arc<Semaphore>) -> Response {
        let mut file = tempfile::tempfile().unwrap();
        file.write_all(b"export").unwrap();
        file.rewind().unwrap();
        export_response(DownloadExport {
            file,
            permit: permits.clone().try_acquire_owned().unwrap(),
            filename: "JM-100.cbz".into(),
            format: ExportFormat::Cbz,
            size: 6,
        })
        .unwrap()
    }

    #[tokio::test]
    async fn export_headers_and_body_hold_the_permit_until_transfer_finishes() {
        let permits = Arc::new(Semaphore::new(1));
        let response = response(&permits);
        assert_eq!(permits.available_permits(), 0);
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "application/vnd.comicbook+zip"
        );
        assert_eq!(response.headers()[header::CONTENT_LENGTH], "6");
        assert_eq!(
            response.headers()[header::CONTENT_DISPOSITION],
            "attachment; filename=\"download.cbz\"; filename*=UTF-8''JM-100.cbz"
        );
        let body = axum::body::to_bytes(response.into_body(), 100)
            .await
            .unwrap();
        assert_eq!(body.as_ref(), b"export");
        assert_eq!(permits.available_permits(), 1);
    }

    #[tokio::test]
    async fn dropping_an_export_response_releases_its_permit() {
        let permits = Arc::new(Semaphore::new(1));
        let response = response(&permits);
        assert_eq!(permits.available_permits(), 0);
        drop(response);
        assert_eq!(permits.available_permits(), 1);
    }
}
