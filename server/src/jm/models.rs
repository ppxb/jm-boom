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
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub liked: bool,
}

/// Search result
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub total: u32,
    pub content: Vec<Comic>,
    #[serde(default)]
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
    pub is_favorite: bool,
    pub liked: bool,
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
    #[serde(default)]
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
    #[serde(default, deserialize_with = "string_vec_from_any")]
    pub tags: Vec<String>,
    #[serde(default, deserialize_with = "u32_from_any")]
    pub likes: u32,
    #[serde(default, deserialize_with = "u32_from_any", rename = "total_views")]
    pub views: u32,
    #[serde(default, deserialize_with = "bool_from_any")]
    pub is_favorite: bool,
    #[serde(default, deserialize_with = "bool_from_any")]
    pub liked: bool,
}

// Helper to deserialize with default fallback
fn string_from_any_or_default<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(string_from_value(value))
}

fn string_vec_from_any<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let values = match value {
        serde_json::Value::Null => Vec::new(),
        serde_json::Value::Array(values) => values,
        value => vec![value],
    };
    Ok(values
        .into_iter()
        .map(string_from_value)
        .filter(|value| !value.is_empty())
        .collect())
}

fn bool_from_any<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Bool(value) => value,
        serde_json::Value::Number(value) => value.as_i64().unwrap_or_default() != 0,
        serde_json::Value::String(value) => matches!(value.as_str(), "1" | "true" | "yes"),
        _ => false,
    })
}

fn string_from_value(value: serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(value) => value,
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        _ => String::new(),
    }
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

// Helper to deserialize any type to String
fn string_from_any<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct StringVisitor;

    impl<'de> Visitor<'de> for StringVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string, number, or boolean")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }
    }

    deserializer.deserialize_any(StringVisitor)
}

// Helper to deserialize any type to u32
fn u32_from_any<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(value
        .as_u64()
        .map(|value| value as u32)
        .or_else(|| value.as_str()?.parse().ok())
        .unwrap_or_default())
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
    #[serde(default)]
    pub author: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub actors: Vec<String>,
    #[serde(default)]
    pub works: Vec<String>,
    #[serde(default, deserialize_with = "u32_from_any")]
    pub total_views: u32,
    #[serde(default, deserialize_with = "u32_from_any")]
    pub likes: u32,
    #[serde(default, deserialize_with = "u32_from_any")]
    pub comment_total: u32,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub liked: bool,
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
            is_favorite: p.is_favorite,
            liked: p.liked,
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
