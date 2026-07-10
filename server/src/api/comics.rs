use crate::{
    api::home::cover_url,
    jm::{ComicDetail, JmResult},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Deserializer, Serialize};

pub async fn get_comic_detail(
    State(app): State<AppState>,
    Path(comic_id): Path<String>,
) -> JmResult<Json<ComicDetail>> {
    let mut detail = app
        .jm_request(move |client, endpoint| {
            let comic_id = comic_id.clone();
            Box::pin(async move { client.get_comic_detail(endpoint, &comic_id).await })
        })
        .await?;
    let img_host = app.img_host().await;
    detail.image = cover_url(img_host.as_deref(), &detail.id, &detail.image);
    for related in &mut detail.related_list {
        related.image = cover_url(img_host.as_deref(), &related.id, &related.image);
    }
    Ok(Json(detail))
}

pub async fn get_comic_chapters(
    State(app): State<AppState>,
    Path(comic_id): Path<String>,
) -> JmResult<Json<Vec<crate::jm::Chapter>>> {
    let detail = app
        .jm_request(move |client, endpoint| {
            let comic_id = comic_id.clone();
            Box::pin(async move { client.get_comic_detail(endpoint, &comic_id).await })
        })
        .await?;
    Ok(Json(detail.series))
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
    let page = query.page.max(1);
    let payload: CommentListPayload = app
        .jm_request(move |client, endpoint| {
            let comic_id = comic_id.clone();
            Box::pin(async move {
                client
                    .get(
                        endpoint,
                        "forum",
                        &[
                            ("page", page.to_string()),
                            ("aid", comic_id),
                            ("mode", "manhua".to_string()),
                        ],
                    )
                    .await
            })
        })
        .await?;
    let img_host = app.img_host().await;
    Ok(Json(ComicCommentsResult {
        page,
        total: payload.total,
        comments: payload
            .list
            .into_iter()
            .map(|comment| map_comment(comment, img_host.as_deref()))
            .collect(),
    }))
}

#[derive(Deserialize)]
struct CommentListPayload {
    #[serde(default, deserialize_with = "u32_from_value")]
    total: u32,
    #[serde(default)]
    list: Vec<CommentPayload>,
}

#[derive(Deserialize)]
struct CommentPayload {
    #[serde(
        default,
        rename = "AID",
        deserialize_with = "optional_string_from_value"
    )]
    aid: Option<String>,
    #[serde(default, rename = "CID", deserialize_with = "string_from_value")]
    cid: String,
    #[serde(default, rename = "UID", deserialize_with = "string_from_value")]
    uid: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    nickname: String,
    #[serde(default, deserialize_with = "u32_from_value")]
    likes: u32,
    #[serde(default)]
    update_at: String,
    #[serde(default)]
    addtime: String,
    #[serde(default, rename = "parent_CID", deserialize_with = "string_from_value")]
    parent_cid: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    photo: String,
    #[serde(default, deserialize_with = "bool_from_value")]
    spoiler: bool,
    #[serde(default)]
    replys: Vec<CommentPayload>,
}

fn map_comment(payload: CommentPayload, img_host: Option<&str>) -> ComicComment {
    let avatar = if payload.photo.starts_with("http") {
        payload.photo.clone()
    } else {
        img_host
            .filter(|_| !payload.photo.is_empty())
            .map(|host| {
                format!(
                    "{}/media/users/{}",
                    host.trim_end_matches('/'),
                    payload.photo.trim_start_matches('/')
                )
            })
            .unwrap_or_default()
    };
    ComicComment {
        id: payload.cid,
        comic_id: payload.aid,
        user_id: payload.uid,
        username: payload.username,
        nickname: payload.nickname,
        content: payload.content,
        like_count: payload.likes,
        time: payload.addtime,
        updated_at: payload.update_at,
        avatar,
        parent_id: payload.parent_cid,
        spoiler: payload.spoiler,
        replies: payload
            .replys
            .into_iter()
            .map(|reply| map_comment(reply, img_host))
            .collect(),
    }
}

fn default_page() -> u32 {
    1
}

fn value_from<'de, D>(deserializer: D) -> Result<serde_json::Value, D::Error>
where
    D: Deserializer<'de>,
{
    serde_json::Value::deserialize(deserializer)
}

fn string_from_value<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = value_from(deserializer)?;
    Ok(match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(value) => value,
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        value => value.to_string(),
    })
}

fn optional_string_from_value<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = string_from_value(deserializer)?;
    Ok((!value.is_empty()).then_some(value))
}

fn u32_from_value<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = value_from(deserializer)?;
    Ok(value
        .as_u64()
        .map(|value| value as u32)
        .or_else(|| value.as_str()?.parse().ok())
        .unwrap_or_default())
}

fn bool_from_value<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = value_from(deserializer)?;
    Ok(match value {
        serde_json::Value::Bool(value) => value,
        serde_json::Value::Number(value) => value.as_i64().unwrap_or_default() != 0,
        serde_json::Value::String(value) => matches!(value.as_str(), "1" | "true"),
        _ => false,
    })
}
