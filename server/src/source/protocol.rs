use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type PageContext = HashMap<String, String>;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MangaStatus {
    #[default]
    Unknown,
    Ongoing,
    Completed,
    Cancelled,
    Hiatus,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContentRating {
    #[default]
    Unknown,
    Safe,
    Suggestive,
    Nsfw,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Viewer {
    #[default]
    Unknown,
    LeftToRight,
    RightToLeft,
    Vertical,
    Webtoon,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateStrategy {
    #[default]
    Always,
    Never,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manga {
    pub key: String,
    pub title: String,
    pub cover: Option<String>,
    pub artists: Option<Vec<String>>,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: MangaStatus,
    pub content_rating: ContentRating,
    pub viewer: Viewer,
    pub update_strategy: UpdateStrategy,
    pub next_update_time: Option<i64>,
    pub chapters: Option<Vec<Chapter>>,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaPageResult {
    pub entries: Vec<Manga>,
    pub has_next_page: bool,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub key: String,
    pub title: Option<String>,
    pub chapter_number: Option<f32>,
    pub volume_number: Option<f32>,
    pub date_uploaded: Option<i64>,
    pub scanlators: Option<Vec<String>>,
    pub url: Option<String>,
    pub language: Option<String>,
    pub thumbnail: Option<String>,
    pub locked: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PageContent {
    Url(String, Option<PageContext>),
    Text(String),
    Image(ImageRef),
    Zip(String, String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub content: PageContent,
    pub thumbnail: Option<String>,
    pub has_description: bool,
    pub description: Option<String>,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ListingKind {
    #[default]
    Default,
    List,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Listing {
    pub id: String,
    pub name: String,
    pub kind: ListingKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FilterValue {
    Text {
        id: String,
        value: String,
    },
    Sort {
        id: String,
        index: i32,
        ascending: bool,
    },
    Check {
        id: String,
        value: i32,
    },
    Select {
        id: String,
        value: String,
    },
    MultiSelect {
        id: String,
        included: Vec<String>,
        excluded: Vec<String>,
    },
    Range {
        id: String,
        from: Option<f32>,
        to: Option<f32>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ImageRef(pub i32);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageRequest {
    pub url: Option<String>,
    pub headers: HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageResponse {
    pub code: u16,
    pub headers: HashMap<String, String>,
    pub request: ImageRequest,
    pub image: ImageRef,
}

#[cfg(test)]
mod tests {
    use super::{ContentRating, Manga, MangaPageResult};

    #[test]
    fn protocol_models_round_trip_through_postcard() {
        let value = MangaPageResult {
            entries: vec![Manga {
                key: "1".into(),
                title: "漫画".into(),
                content_rating: ContentRating::Nsfw,
                ..Manga::default()
            }],
            has_next_page: true,
        };
        let bytes = postcard::to_allocvec(&value).expect("encode protocol value");
        let decoded: MangaPageResult = postcard::from_bytes(&bytes).expect("decode protocol value");
        assert_eq!(decoded, value);
    }
}
