use crate::api::media::cover_url;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ComicSummaryResponse {
    id: String,
    title: String,
    author: String,
    description: String,
    image: String,
    tags: Vec<String>,
}

impl ComicSummaryResponse {
    pub(super) fn new(
        id: String,
        title: String,
        author: String,
        description: String,
        image: String,
        tags: Vec<String>,
    ) -> Self {
        Self {
            image: cover_url(&id, &image),
            id,
            title,
            author,
            description,
            tags,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ComicSummaryResponse;

    #[test]
    fn serializes_the_shared_comic_summary_contract() {
        let value = serde_json::to_value(ComicSummaryResponse::new(
            "123".into(),
            "Title".into(),
            "Author".into(),
            "Description".into(),
            "fallback.jpg".into(),
            vec!["Tag".into()],
        ))
        .expect("serialize comic summary response");

        assert_eq!(
            value,
            serde_json::json!({
                "id": "123",
                "title": "Title",
                "author": "Author",
                "description": "Description",
                "image": "/api/covers/123",
                "tags": ["Tag"]
            })
        );
    }
}
