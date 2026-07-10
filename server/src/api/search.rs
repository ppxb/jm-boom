use crate::{api::home::cover_url, jm::JmResult, AppState};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub keyword: String,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_sort_by", rename = "sortBy")]
    pub sort_by: u8,
}

fn default_page() -> u32 {
    1
}

fn default_sort_by() -> u8 {
    1
}

pub async fn search_comics(
    State(app): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> JmResult<Json<crate::jm::SearchResult>> {
    let keyword = query.keyword.trim().to_string();
    if keyword.is_empty() {
        return Ok(Json(crate::jm::SearchResult {
            total: 0,
            content: vec![],
            redirect_aid: None,
        }));
    }

    if query.page == 1 && keyword.chars().all(|ch| ch.is_ascii_digit()) {
        let comic_id = keyword.clone();
        if let Ok(detail) = app
            .jm_request(move |client, endpoint| {
                let comic_id = comic_id.clone();
                Box::pin(async move { client.get_comic_detail(endpoint, &comic_id).await })
            })
            .await
        {
            return Ok(Json(crate::jm::SearchResult {
                total: 1,
                content: vec![crate::jm::Comic {
                    image: cover_url(&detail.id, &detail.image),
                    id: detail.id,
                    name: detail.name,
                    author: detail.author.join(" / "),
                    description: detail.description,
                    tags: detail.tags,
                    likes: detail.likes,
                    views: detail.total_views,
                }],
                redirect_aid: None,
            }));
        }
    }

    let page = query.page;
    let order = match query.sort_by {
        2 => "mv",
        3 => "mp",
        4 => "tf",
        _ => "mr",
    }
    .to_string();
    let result = app
        .jm_request(move |client, endpoint| {
            let keyword = keyword.clone();
            let order = order.clone();
            Box::pin(async move { client.search(endpoint, &keyword, page, &order).await })
        })
        .await?;
    let result = crate::jm::SearchResult {
        content: result
            .content
            .into_iter()
            .map(|mut comic| {
                comic.image = cover_url(&comic.id, &comic.image);
                comic
            })
            .collect(),
        ..result
    };
    Ok(Json(result))
}
