use crate::{api::media::cover_url, application::FavoriteInput, http_error::HttpError, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteListResponse {
    items: Vec<FavoriteResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FavoriteResponse {
    id: String,
    title: String,
    author: String,
    description: String,
    image: String,
    tags: Vec<String>,
    favorited_at: i64,
}

pub async fn list(State(app): State<AppState>) -> Result<Json<FavoriteListResponse>, HttpError> {
    favorite_list(&app).await.map(Json)
}

pub async fn upsert(
    State(app): State<AppState>,
    Path(comic_id): Path<String>,
    Json(input): Json<FavoriteInput>,
) -> Result<Json<FavoriteResponse>, HttpError> {
    validate_comic_id(&comic_id)?;
    app.favorites
        .upsert(&comic_id, input)
        .await
        .map(FavoriteResponse::from)
        .map(Json)
        .map_err(internal_error)
}

pub async fn remove(
    State(app): State<AppState>,
    Path(comic_id): Path<String>,
) -> Result<StatusCode, HttpError> {
    validate_comic_id(&comic_id)?;
    app.favorites
        .remove(&comic_id)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn clear(State(app): State<AppState>) -> Result<StatusCode, HttpError> {
    app.favorites.clear().await.map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn favorite_list(app: &AppState) -> Result<FavoriteListResponse, HttpError> {
    let items = app
        .favorites
        .list()
        .await
        .map_err(internal_error)?
        .into_iter()
        .map(FavoriteResponse::from)
        .collect();
    Ok(FavoriteListResponse { items })
}

impl From<crate::application::FavoriteItem> for FavoriteResponse {
    fn from(item: crate::application::FavoriteItem) -> Self {
        Self {
            image: cover_url(&item.comic_id, &item.image),
            id: item.comic_id,
            title: item.title,
            author: item.author,
            description: item.description,
            tags: item.tags,
            favorited_at: item.favorited_at,
        }
    }
}

fn validate_comic_id(comic_id: &str) -> Result<(), HttpError> {
    if comic_id.is_empty() || !comic_id.chars().all(|character| character.is_ascii_digit()) {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "收藏漫画 ID 必须为数字",
            false,
        ));
    }
    Ok(())
}

fn internal_error(error: anyhow::Error) -> HttpError {
    tracing::error!(%error, "收藏存储操作失败");
    HttpError::internal("收藏存储操作失败")
}
