use super::{media::cover_url, CollectionListQuery, COLLECTION_PAGE_SIZE};
use crate::{application::ReadingHistoryInput, http_error::HttpError, AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingHistoryListResponse {
    items: Vec<ReadingHistoryResponse>,
    total: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReadingHistoryResponse {
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
    Query(query): Query<CollectionListQuery>,
) -> Result<Json<ReadingHistoryListResponse>, HttpError> {
    validate_page(query.page)?;
    history_list(&app, query.page).await.map(Json)
}

pub async fn upsert(
    State(app): State<AppState>,
    Path(comic_id): Path<String>,
    Json(input): Json<ReadingHistoryInput>,
) -> Result<StatusCode, HttpError> {
    validate_item(&comic_id, &input)?;
    app.history
        .upsert(&comic_id, input)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove(
    State(app): State<AppState>,
    Path(comic_id): Path<String>,
) -> Result<StatusCode, HttpError> {
    validate_comic_id(&comic_id)?;
    app.history
        .remove_many(&[comic_id])
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_many(
    State(app): State<AppState>,
    Json(payload): Json<RemoveHistoryRequest>,
) -> Result<StatusCode, HttpError> {
    for comic_id in &payload.comic_ids {
        validate_comic_id(comic_id)?;
    }
    app.history
        .remove_many(&payload.comic_ids)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn clear(State(app): State<AppState>) -> Result<StatusCode, HttpError> {
    app.history.clear().await.map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn history_list(app: &AppState, page: u32) -> Result<ReadingHistoryListResponse, HttpError> {
    let (items, total) = app
        .history
        .list(page, COLLECTION_PAGE_SIZE)
        .await
        .map_err(internal_error)?;
    let items = items
        .into_iter()
        .map(ReadingHistoryResponse::from)
        .collect();
    Ok(ReadingHistoryListResponse { items, total })
}

impl From<crate::application::ReadingHistoryItem> for ReadingHistoryResponse {
    fn from(item: crate::application::ReadingHistoryItem) -> Self {
        Self {
            image: cover_url(&item.comic_id, &item.image),
            id: item.comic_id,
            title: item.title,
            author: item.author,
            chapter_id: item.chapter_id,
            chapter_title: item.chapter_title,
            page_index: item.page_index,
            page_count: item.page_count,
            last_read_at: item.last_read_at,
        }
    }
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

fn validate_page(page: u32) -> Result<(), HttpError> {
    if page == 0 {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "页码必须大于 0",
            false,
        ));
    }
    Ok(())
}

fn internal_error(error: anyhow::Error) -> HttpError {
    tracing::error!(%error, "阅读历史存储操作失败");
    HttpError::internal("阅读历史存储操作失败")
}
