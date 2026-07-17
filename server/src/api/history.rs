use crate::{
    api::media::cover_url, application::ReadingHistoryInput, http_error::HttpError, AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingHistoryListResponse {
    items: Vec<ReadingHistoryResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadingHistoryResponse {
    id: String,
    title: String,
    author: String,
    image: String,
    chapter_id: String,
    chapter_title: String,
    page_index: i64,
    page_count: i64,
    last_read_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportHistoryRequest {
    items: Vec<ImportHistoryItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportHistoryItem {
    id: String,
    title: String,
    author: String,
    image: String,
    chapter_id: String,
    chapter_title: String,
    page_index: i64,
    page_count: i64,
    last_read_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveHistoryRequest {
    comic_ids: Vec<String>,
}

pub async fn list(
    State(app): State<AppState>,
) -> Result<Json<ReadingHistoryListResponse>, HttpError> {
    history_list(&app).await.map(Json)
}

pub async fn upsert(
    State(app): State<AppState>,
    Path(comic_id): Path<String>,
    Json(input): Json<ReadingHistoryInput>,
) -> Result<Json<ReadingHistoryListResponse>, HttpError> {
    validate_item(&comic_id, &input)?;
    app.history
        .upsert(&comic_id, input)
        .await
        .map_err(internal_error)?;
    history_list(&app).await.map(Json)
}

pub async fn import(
    State(app): State<AppState>,
    Json(payload): Json<ImportHistoryRequest>,
) -> Result<Json<ReadingHistoryListResponse>, HttpError> {
    let mut items = Vec::with_capacity(payload.items.len());
    for item in payload.items {
        let input = ReadingHistoryInput {
            title: item.title,
            author: item.author,
            image: item.image,
            chapter_id: item.chapter_id,
            chapter_title: item.chapter_title,
            page_index: item.page_index,
            page_count: item.page_count,
            last_read_at: Some(item.last_read_at),
        };
        validate_item(&item.id, &input)?;
        items.push((item.id, input));
    }
    app.history.import(items).await.map_err(internal_error)?;
    history_list(&app).await.map(Json)
}

pub async fn remove(
    State(app): State<AppState>,
    Path(comic_id): Path<String>,
) -> Result<Json<ReadingHistoryListResponse>, HttpError> {
    validate_comic_id(&comic_id)?;
    app.history
        .remove_many(&[comic_id])
        .await
        .map_err(internal_error)?;
    history_list(&app).await.map(Json)
}

pub async fn remove_many(
    State(app): State<AppState>,
    Json(payload): Json<RemoveHistoryRequest>,
) -> Result<Json<ReadingHistoryListResponse>, HttpError> {
    for comic_id in &payload.comic_ids {
        validate_comic_id(comic_id)?;
    }
    app.history
        .remove_many(&payload.comic_ids)
        .await
        .map_err(internal_error)?;
    history_list(&app).await.map(Json)
}

pub async fn clear(
    State(app): State<AppState>,
) -> Result<Json<ReadingHistoryListResponse>, HttpError> {
    app.history.clear().await.map_err(internal_error)?;
    history_list(&app).await.map(Json)
}

async fn history_list(app: &AppState) -> Result<ReadingHistoryListResponse, HttpError> {
    let items = app
        .history
        .list()
        .await
        .map_err(internal_error)?
        .into_iter()
        .map(
            |item: crate::application::ReadingHistoryItem| ReadingHistoryResponse {
                image: cover_url(&item.comic_id, &item.image),
                id: item.comic_id,
                title: item.title,
                author: item.author,
                chapter_id: item.chapter_id,
                chapter_title: item.chapter_title,
                page_index: item.page_index,
                page_count: item.page_count,
                last_read_at: item.last_read_at,
            },
        )
        .collect();
    Ok(ReadingHistoryListResponse { items })
}

fn validate_item(comic_id: &str, input: &ReadingHistoryInput) -> Result<(), HttpError> {
    validate_comic_id(comic_id)?;
    validate_comic_id(&input.chapter_id)?;
    if input.page_index < 0 || input.page_count <= 0 || input.page_index >= input.page_count {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "阅读进度页码无效",
            false,
        ));
    }
    Ok(())
}

fn validate_comic_id(comic_id: &str) -> Result<(), HttpError> {
    if comic_id.is_empty() || !comic_id.chars().all(|character| character.is_ascii_digit()) {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "漫画 ID 必须为数字",
            false,
        ));
    }
    Ok(())
}

fn internal_error(error: anyhow::Error) -> HttpError {
    tracing::error!(%error, "阅读历史存储操作失败");
    HttpError::internal("阅读历史存储操作失败")
}
