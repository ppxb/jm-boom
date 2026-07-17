use crate::{
    api::comic_dto::{map_comic_detail, ComicDetailResponse},
    api::history::ReadingHistoryResponse,
    application::ComicComments,
    domain::comic::ComicComment as DomainComicComment,
    http_error::HttpError,
    jm::JmResult,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

pub async fn get_comic_detail(
    State(app): State<AppState>,
    Path(comic_id): Path<String>,
) -> JmResult<Json<ComicDetailResponse>> {
    app.comics
        .get_comic_detail(comic_id)
        .await
        .map(map_comic_detail)
        .map(Json)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicStateResponse {
    is_favorite: bool,
    history: Option<ReadingHistoryResponse>,
}

pub async fn get_comic_state(
    State(app): State<AppState>,
    Path(comic_id): Path<String>,
) -> Result<Json<ComicStateResponse>, HttpError> {
    validate_comic_id(&comic_id)?;
    let (is_favorite, history) = tokio::try_join!(
        app.favorites.contains(&comic_id),
        app.history.get(&comic_id)
    )
    .map_err(state_error)?;

    Ok(Json(ComicStateResponse {
        is_favorite,
        history: history.map(ReadingHistoryResponse::from),
    }))
}

#[derive(Deserialize)]
pub struct CommentsQuery {
    #[serde(default = "default_page")]
    page: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicCommentsResult {
    page: u32,
    total: u32,
    comments: Vec<ComicComment>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicComment {
    id: String,
    comic_id: Option<String>,
    user_id: String,
    username: String,
    nickname: String,
    content: String,
    like_count: u32,
    time: String,
    updated_at: String,
    avatar: String,
    parent_id: String,
    spoiler: bool,
    replies: Vec<ComicComment>,
}

pub async fn get_comments(
    State(app): State<AppState>,
    Path(comic_id): Path<String>,
    Query(query): Query<CommentsQuery>,
) -> JmResult<Json<ComicCommentsResult>> {
    app.comics
        .get_comments(comic_id, query.page)
        .await
        .map(ComicCommentsResult::from)
        .map(Json)
}

impl From<ComicComments> for ComicCommentsResult {
    fn from(result: ComicComments) -> Self {
        Self {
            page: result.page,
            total: result.total,
            comments: result.comments.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<DomainComicComment> for ComicComment {
    fn from(comment: DomainComicComment) -> Self {
        Self {
            id: comment.id,
            comic_id: comment.comic_id,
            user_id: comment.user_id,
            username: comment.username,
            nickname: comment.nickname,
            content: comment.content,
            like_count: comment.like_count,
            time: comment.time,
            updated_at: comment.updated_at,
            avatar: comment.avatar,
            parent_id: comment.parent_id,
            spoiler: comment.spoiler,
            replies: comment.replies.into_iter().map(Into::into).collect(),
        }
    }
}

fn default_page() -> u32 {
    1
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

fn state_error(error: anyhow::Error) -> HttpError {
    tracing::error!(%error, "漫画状态查询失败");
    HttpError::internal("漫画状态查询失败")
}
