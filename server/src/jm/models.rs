use super::serde_ext::{
    lossy_string_vec_from_array_or_scalar, optional_string_from_any, string_from_any,
    string_from_any_or_default, u32_from_any,
};
use serde::{Deserialize, Serialize};

// ============ API Response Models ============

/// Comic basic info (used in lists, search results)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comic {
    pub id: String,
    pub name: String,
    pub author: String,
    #[serde(default)]
    pub description: String,
    pub image: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub likes: u32,
    #[serde(default)]
    pub views: u32,
}

/// Search result
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub total: u32,
    pub content: Vec<Comic>,
    #[serde(default, deserialize_with = "optional_string_from_any")]
    pub redirect_aid: Option<String>,
}

/// Comic detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComicDetail {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub author: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub actors: Vec<String>,
    #[serde(default)]
    pub works: Vec<String>,
    pub total_views: u32,
    pub likes: u32,
    pub comment_total: u32,
    #[serde(default)]
    pub related_list: Vec<RelatedComic>,
    #[serde(default)]
    pub series: Vec<Chapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedComic {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub image: String,
}

/// Chapter info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub sort: String,
    #[serde(default)]
    pub images: Vec<String>,
}

/// Home feed section
#[derive(Debug, Serialize, Deserialize)]
pub struct HomeSection {
    pub id: String,
    pub title: String,
    pub slug: String,
    #[serde(rename = "type")]
    pub section_type: String,
    pub filter_val: String,
    pub content: Vec<Comic>,
}

// ============ Internal Payload Models ============

#[derive(Debug, Deserialize)]
pub(crate) struct SearchPayload {
    #[serde(default, deserialize_with = "u32_from_any")]
    pub total: u32,
    #[serde(default, deserialize_with = "optional_string_from_any")]
    pub redirect_aid: Option<String>,
    #[serde(default)]
    pub content: Vec<ComicPayload>,
}

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
    #[serde(default, deserialize_with = "u32_from_any")]
    pub likes: u32,
    #[serde(default, deserialize_with = "u32_from_any", rename = "total_views")]
    pub views: u32,
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
            likes: p.likes,
            views: p.views,
        }
    }
}

impl From<RelatedComicPayload> for RelatedComic {
    fn from(p: RelatedComicPayload) -> Self {
        Self {
            id: p.id,
            name: p.name,
            author: p.author,
            image: p.image,
        }
    }
}

impl From<ChapterPayload> for Chapter {
    fn from(p: ChapterPayload) -> Self {
        Self {
            id: p.id,
            name: p.name,
            sort: p.sort,
            images: Vec::new(),
        }
    }
}
