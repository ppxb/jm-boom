use crate::{
    api::comic_dto::{map_comic_detail, ComicDetailResponse},
    application::ComicComments,
    domain::comic::ComicComment as DomainComicComment,
    jm::JmResult,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
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
