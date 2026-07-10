use serde::{Deserialize, Serialize};

/// Comic basic info (used in lists, search results)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comic {
    pub id: String,
    pub name: String,
    pub author: String,
    #[serde(default)]
    pub description: Option<String>,
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

/// Comic detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComicDetail {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
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
    pub series: Vec<SeriesItem>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesItem {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub sort: String,
}

/// Chapter info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub sort: String,
}

/// Search result
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub total: u32,
    pub content: Vec<Comic>,
    #[serde(default)]
    pub redirect_aid: Option<String>,
}

/// Home feed section
#[derive(Debug, Serialize, Deserialize)]
pub struct HomeSection {
    pub id: String,
    pub title: String,
    pub slug: String,
    #[serde(rename = "type")]
    pub section_type: String,
    pub content: Vec<Comic>,
}

/// User info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub email: Option<String>,
    pub avatar: Option<String>,
}

/// Favorite folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteFolder {
    pub id: String,
    pub name: String,
}

/// Reading history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingHistory {
    pub comic_id: String,
    pub chapter_id: String,
    pub page: u32,
    pub updated_at: i64,
}
