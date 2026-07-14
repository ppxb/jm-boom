#[derive(Debug, Clone)]
pub struct ComicDetail {
    pub id: String,
    pub title: String,
    pub description: String,
    pub image: String,
    pub authors: Vec<String>,
    pub tags: Vec<String>,
    pub actors: Vec<String>,
    pub works: Vec<String>,
    pub total_views: u32,
    pub likes: u32,
    pub comment_count: u32,
    pub related_comics: Vec<RelatedComic>,
    pub chapters: Vec<ComicChapter>,
}

#[derive(Debug, Clone)]
pub struct RelatedComic {
    pub id: String,
    pub title: String,
    pub author: String,
    pub image: String,
}

#[derive(Debug, Clone)]
pub struct ComicChapter {
    pub id: String,
    pub title: String,
    pub sort: String,
}
