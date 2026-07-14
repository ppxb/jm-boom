use super::ComicService;
use crate::{
    domain::comic::ComicComment,
    jm::{
        serde_ext::{optional_string_from_any, string_from_any, u32_from_any},
        JmResult,
    },
};
use serde::{Deserialize, Deserializer};

pub(crate) struct ComicComments {
    pub(crate) page: u32,
    pub(crate) total: u32,
    pub(crate) comments: Vec<ComicComment>,
}

impl ComicService {
    pub async fn get_comments(&self, comic_id: String, page: u32) -> JmResult<ComicComments> {
        let page = page.max(1);
        let payload: CommentListPayload = self
            .with_failover(move |client, endpoint| {
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
        let image_host = self.image_host().await;

        Ok(ComicComments {
            page,
            total: payload.total,
            comments: payload
                .list
                .into_iter()
                .map(|comment| map_comment(comment, image_host.as_deref()))
                .collect(),
        })
    }
}

#[derive(Deserialize)]
struct CommentListPayload {
    #[serde(default, deserialize_with = "u32_from_any")]
    total: u32,
    #[serde(default)]
    list: Vec<CommentPayload>,
}

#[derive(Deserialize)]
struct CommentPayload {
    #[serde(default, rename = "AID", deserialize_with = "optional_string_from_any")]
    comic_id: Option<String>,
    #[serde(default, rename = "CID", deserialize_with = "string_from_any")]
    id: String,
    #[serde(default, rename = "UID", deserialize_with = "string_from_any")]
    user_id: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    nickname: String,
    #[serde(default, rename = "likes", deserialize_with = "u32_from_any")]
    like_count: u32,
    #[serde(default, rename = "update_at")]
    updated_at: String,
    #[serde(default, rename = "addtime")]
    time: String,
    #[serde(default, rename = "parent_CID", deserialize_with = "string_from_any")]
    parent_id: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    photo: String,
    #[serde(default, deserialize_with = "bool_from_any")]
    spoiler: bool,
    #[serde(default, rename = "replys")]
    replies: Vec<CommentPayload>,
}

fn map_comment(payload: CommentPayload, image_host: Option<&str>) -> ComicComment {
    let avatar = if payload.photo.starts_with("http") {
        payload.photo
    } else {
        image_host
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
        id: payload.id,
        comic_id: payload.comic_id,
        user_id: payload.user_id,
        username: payload.username,
        nickname: payload.nickname,
        content: payload.content,
        like_count: payload.like_count,
        time: payload.time,
        updated_at: payload.updated_at,
        avatar,
        parent_id: payload.parent_id,
        spoiler: payload.spoiler,
        replies: payload
            .replies
            .into_iter()
            .map(|reply| map_comment(reply, image_host))
            .collect(),
    }
}

fn bool_from_any<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Bool(value) => value,
        serde_json::Value::Number(value) => value.as_i64().unwrap_or_default() != 0,
        serde_json::Value::String(value) => matches!(value.as_str(), "1" | "true"),
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::{map_comment, CommentListPayload};

    #[test]
    fn preserves_upstream_comment_field_mappings() {
        let payload: CommentListPayload = serde_json::from_value(serde_json::json!({
            "total": "1",
            "list": [{
                "AID": 123,
                "CID": 456,
                "UID": 789,
                "username": "user",
                "nickname": "nickname",
                "content": "comment",
                "likes": "12",
                "addtime": "created",
                "update_at": "updated",
                "parent_CID": 0,
                "photo": "avatar.jpg",
                "spoiler": "1",
                "replys": []
            }]
        }))
        .expect("deserialize upstream comment payload");

        let comment = map_comment(
            payload.list.into_iter().next().expect("comment"),
            Some("https://cdn.example"),
        );
        assert_eq!(payload.total, 1);
        assert_eq!(comment.id, "456");
        assert_eq!(comment.comic_id.as_deref(), Some("123"));
        assert_eq!(comment.user_id, "789");
        assert_eq!(comment.like_count, 12);
        assert_eq!(comment.time, "created");
        assert_eq!(comment.updated_at, "updated");
        assert_eq!(comment.parent_id, "0");
        assert_eq!(comment.avatar, "https://cdn.example/media/users/avatar.jpg");
        assert!(comment.spoiler);
    }
}
