use super::serde_ext::{lossy_string_vec_from_array_or_scalar, string_from_any, u32_from_any};
use serde::Deserialize;

// ============ Internal Payload Models ============

#[derive(Debug, Deserialize)]
pub(crate) struct ComicDetailPayload {
    #[serde(deserialize_with = "string_from_any")]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub image: String,
    #[serde(default, deserialize_with = "lossy_string_vec_from_array_or_scalar")]
    pub author: Vec<String>,
    #[serde(default, deserialize_with = "lossy_string_vec_from_array_or_scalar")]
    pub tags: Vec<String>,
    #[serde(default, deserialize_with = "lossy_string_vec_from_array_or_scalar")]
    pub actors: Vec<String>,
    #[serde(default, deserialize_with = "lossy_string_vec_from_array_or_scalar")]
    pub works: Vec<String>,
    #[serde(default, deserialize_with = "u32_from_any")]
    pub total_views: u32,
    #[serde(default, deserialize_with = "u32_from_any")]
    pub likes: u32,
    #[serde(default, deserialize_with = "u32_from_any")]
    pub comment_total: u32,
    #[serde(default)]
    pub related_list: Vec<RelatedComicPayload>,
    #[serde(default)]
    pub series: Vec<ChapterPayload>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RelatedComicPayload {
    #[serde(deserialize_with = "string_from_any")]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub image: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChapterPayload {
    #[serde(deserialize_with = "string_from_any")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub sort: String,
}

#[cfg(test)]
mod tests {
    use super::ComicDetailPayload;

    #[test]
    fn deserializes_mixed_scalar_and_array_fields_from_upstream_samples() {
        let detail: ComicDetailPayload = serde_json::from_value(serde_json::json!({
            "id": 54321,
            "name": "detail",
            "author": "single-author",
            "tags": 7,
            "actors": ["actor-a", 8, false],
            "works": null,
            "total_views": "1001",
            "likes": true,
            "comment_total": null,
            "related_list": [{"id": 9, "name": "related"}],
            "series": [{"id": 10, "name": "chapter", "sort": "1"}]
        }))
        .expect("decode mixed comic detail payload");
        assert_eq!(detail.id, "54321");
        assert_eq!(detail.author, vec!["single-author"]);
        assert_eq!(detail.tags, vec!["7"]);
        assert_eq!(detail.actors, vec!["actor-a", "8", "false"]);
        assert!(detail.works.is_empty());
        assert_eq!(detail.total_views, 1001);
        assert_eq!(detail.likes, 1);
        assert_eq!(detail.comment_total, 0);
        assert_eq!(detail.related_list[0].id, "9");
        assert_eq!(detail.series[0].id, "10");
    }
}
