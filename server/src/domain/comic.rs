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
pub struct ComicSummary {
    pub id: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub image: String,
    pub tags: Vec<String>,
}

impl ComicSummary {
    pub fn new(
        id: String,
        title: String,
        author: String,
        description: String,
        image: String,
        tags: Vec<String>,
    ) -> Self {
        Self {
            id,
            title,
            author,
            description,
            image,
            tags,
        }
    }

    pub fn from_detail(detail: ComicDetail) -> Self {
        Self::new(
            detail.id,
            detail.title,
            detail.authors.join(" / "),
            detail.description,
            detail.image,
            detail.tags,
        )
    }
}

#[derive(Debug, Clone)]
pub struct ComicComment {
    pub id: String,
    pub comic_id: Option<String>,
    pub user_id: String,
    pub username: String,
    pub nickname: String,
    pub content: String,
    pub like_count: u32,
    pub time: String,
    pub updated_at: String,
    pub avatar: String,
    pub parent_id: String,
    pub spoiler: bool,
    pub replies: Vec<ComicComment>,
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
