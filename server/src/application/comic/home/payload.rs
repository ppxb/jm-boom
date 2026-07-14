use crate::jm::serde_ext::{string_from_any, u32_from_any};
use serde::Deserialize;

pub(super) struct SectionPayload {
    pub(super) page_size: u32,
    pub(super) total: u32,
    pub(super) has_more: bool,
    pub(super) items: Vec<ComicListPayload>,
}

#[derive(Deserialize)]
pub(super) struct PromotePayload {
    #[serde(default, deserialize_with = "u32_from_any")]
    pub(super) total: u32,
    #[serde(default)]
    pub(super) list: Vec<ComicListPayload>,
}

#[derive(Deserialize)]
pub(super) struct WeeklyPayload {
    #[serde(default)]
    pub(super) list: Vec<ComicListPayload>,
}

#[derive(Deserialize)]
pub(super) struct CategoryPayload {
    #[serde(default, deserialize_with = "u32_from_any")]
    pub(super) total: u32,
    #[serde(default)]
    pub(super) content: Vec<ComicListPayload>,
}

#[derive(Deserialize)]
pub(super) struct WeekPayload {
    #[serde(default)]
    pub(super) categories: Vec<WeekCategoryPayload>,
    #[serde(default, rename = "type")]
    pub(super) types: Vec<WeekTypePayload>,
}

#[derive(Deserialize)]
pub(super) struct WeekCategoryPayload {
    #[serde(default, deserialize_with = "string_from_any")]
    pub(super) id: String,
    #[serde(default, deserialize_with = "string_from_any")]
    pub(super) time: String,
    #[serde(default, deserialize_with = "string_from_any")]
    pub(super) title: String,
}

#[derive(Deserialize)]
pub(super) struct WeekTypePayload {
    #[serde(default, deserialize_with = "string_from_any")]
    pub(super) id: String,
    #[serde(default, deserialize_with = "string_from_any")]
    pub(super) title: String,
}

#[derive(Deserialize)]
pub(super) struct WeekComicsPayload {
    #[serde(default, deserialize_with = "u32_from_any")]
    pub(super) total: u32,
    #[serde(default)]
    pub(super) list: Vec<ComicListPayload>,
}

#[derive(Deserialize)]
pub(super) struct ComicListPayload {
    #[serde(default, deserialize_with = "string_from_any")]
    pub(super) id: String,
    #[serde(default, deserialize_with = "string_from_any")]
    pub(super) author: String,
    #[serde(default, deserialize_with = "string_from_any")]
    pub(super) description: String,
    #[serde(default, deserialize_with = "string_from_any")]
    pub(super) name: String,
    #[serde(default, deserialize_with = "string_from_any")]
    pub(super) image: String,
    #[serde(default)]
    pub(super) category: Option<CategoryTag>,
    #[serde(default, rename = "category_sub")]
    pub(super) category_sub: Option<CategoryTag>,
}

#[derive(Deserialize)]
pub(super) struct CategoryTag {
    #[serde(default, deserialize_with = "string_from_any")]
    pub(super) title: String,
}
