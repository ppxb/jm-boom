use super::serde_ext::{
    lossy_string_vec_from_array_or_scalar, string_from_any, string_from_any_or_default,
    u32_from_any,
};
use serde::Deserialize;

// ============ Normalized JM Models ============

/// Comic basic info (used in lists, search results)
#[derive(Debug, Clone)]
pub struct Comic {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub image: String,
    pub tags: Vec<String>,
}

/// Home feed section
#[derive(Debug)]
pub struct HomeSection {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub section_type: String,
    pub filter_val: String,
    pub content: Vec<Comic>,
}

// ============ Internal Payload Models ============

#[derive(Debug, Deserialize)]
pub(crate) struct ComicPayload {
    #[serde(default, deserialize_with = "string_from_any_or_default")]
    pub id: String,
    #[serde(default, deserialize_with = "string_from_any_or_default")]
    pub name: String,
    #[serde(default, deserialize_with = "string_from_any_or_default")]
    pub author: String,
    #[serde(default, deserialize_with = "string_from_any_or_default")]
    pub description: String,
    #[serde(default, deserialize_with = "string_from_any_or_default")]
    pub image: String,
    #[serde(default, deserialize_with = "lossy_string_vec_from_array_or_scalar")]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct HomeSectionPayload {
    #[serde(deserialize_with = "string_from_any")]
    pub id: String,
    pub title: String,
    pub slug: String,
    #[serde(rename = "type")]
    pub section_type: String,
    #[serde(deserialize_with = "string_from_any")]
    pub filter_val: String,
    pub content: Vec<ComicPayload>,
}

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

// ============ Conversion Helpers ============

impl From<ComicPayload> for Comic {
    fn from(p: ComicPayload) -> Self {
        Self {
            id: p.id,
            name: p.name,
            author: p.author,
            description: p.description,
            image: p.image,
            tags: p.tags,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ComicDetailPayload, ComicPayload};

    #[test]
    fn deserializes_mixed_scalar_and_array_fields_from_upstream_samples() {
        let comic: ComicPayload = serde_json::from_value(serde_json::json!({
            "id": 12345,
            "name": true,
            "author": 678,
            "description": null,
            "image": false,
            "tags": ["tag-a", 2, true, null, ""],
            "likes": "42",
            "total_views": 99
        }))
        .expect("decode mixed comic payload");
        assert_eq!(comic.id, "12345");
        assert_eq!(comic.name, "true");
        assert_eq!(comic.author, "678");
        assert_eq!(comic.description, "");
        assert_eq!(comic.image, "false");
        assert_eq!(comic.tags, vec!["tag-a", "2", "true"]);

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
