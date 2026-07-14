use crate::{
    api::media::cover_url,
    domain::comic::{ComicChapter, ComicDetail, RelatedComic},
};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicDetailResponse {
    id: String,
    title: String,
    description: String,
    image: String,
    authors: Vec<String>,
    tags: Vec<String>,
    actors: Vec<String>,
    works: Vec<String>,
    total_views: u32,
    likes: u32,
    comment_count: u32,
    related_comics: Vec<RelatedComicResponse>,
    chapters: Vec<ComicChapterResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RelatedComicResponse {
    id: String,
    title: String,
    author: String,
    image: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ComicChapterResponse {
    id: String,
    title: String,
    sort: String,
}

pub fn map_comic_detail(detail: ComicDetail) -> ComicDetailResponse {
    ComicDetailResponse {
        image: cover_url(&detail.id, &detail.image),
        id: detail.id,
        title: detail.title,
        description: detail.description,
        authors: normalize_text_list(detail.authors),
        tags: normalize_text_list(detail.tags),
        actors: normalize_text_list(detail.actors),
        works: normalize_text_list(detail.works),
        total_views: detail.total_views,
        likes: detail.likes,
        comment_count: detail.comment_count,
        related_comics: detail
            .related_comics
            .into_iter()
            .map(map_related_comic)
            .collect(),
        chapters: detail.chapters.into_iter().map(map_chapter).collect(),
    }
}

fn map_related_comic(related: RelatedComic) -> RelatedComicResponse {
    RelatedComicResponse {
        image: cover_url(&related.id, &related.image),
        id: related.id,
        title: related.title,
        author: related.author.trim().to_string(),
    }
}

fn map_chapter(chapter: ComicChapter) -> ComicChapterResponse {
    ComicChapterResponse {
        id: chapter.id,
        title: chapter.title,
        sort: chapter.sort,
    }
}

fn normalize_text_list(items: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::with_capacity(items.len());
    let mut normalized = Vec::with_capacity(items.len());
    for item in items {
        let item = item.trim().to_string();
        if !item.is_empty() && seen.insert(item.clone()) {
            normalized.push(item);
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::map_comic_detail;
    use crate::domain::comic::{ComicChapter, ComicDetail, RelatedComic};

    #[test]
    fn serializes_the_comic_detail_api_contract() {
        let response = map_comic_detail(ComicDetail {
            id: "123".into(),
            title: "Example".into(),
            description: "Description".into(),
            image: "upstream.jpg".into(),
            authors: vec![" Author ".into(), "Author".into(), String::new()],
            tags: vec!["Tag".into(), " Tag ".into()],
            actors: Vec::new(),
            works: vec!["Work".into()],
            total_views: 100,
            likes: 20,
            comment_count: 3,
            related_comics: vec![RelatedComic {
                id: "456".into(),
                title: "Related".into(),
                author: " Related Author ".into(),
                image: "related.jpg".into(),
            }],
            chapters: vec![ComicChapter {
                id: "789".into(),
                title: "Chapter 1".into(),
                sort: "1".into(),
            }],
        });

        let value = serde_json::to_value(response).expect("serialize comic detail response");
        assert_eq!(
            value,
            serde_json::json!({
                "id": "123",
                "title": "Example",
                "description": "Description",
                "image": "/api/covers/123",
                "authors": ["Author"],
                "tags": ["Tag"],
                "actors": [],
                "works": ["Work"],
                "totalViews": 100,
                "likes": 20,
                "commentCount": 3,
                "relatedComics": [{
                    "id": "456",
                    "title": "Related",
                    "author": "Related Author",
                    "image": "/api/covers/456"
                }],
                "chapters": [{
                    "id": "789",
                    "title": "Chapter 1",
                    "sort": "1"
                }]
            })
        );
    }
}
