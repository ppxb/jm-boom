use crate::{jm::JmResult, AppState};
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
            let img_host = app.img_host().await;
            return Ok(Json(crate::jm::SearchResult {
                total: 1,
                content: vec![crate::jm::Comic {
                    image: cover_url(img_host.as_deref(), &detail.id, &detail.image),
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
    let img_host = app.img_host().await;
    let result = crate::jm::SearchResult {
        content: result
            .content
            .into_iter()
            .map(|mut comic| {
                comic.image = cover_url(img_host.as_deref(), &comic.id, &comic.image);
                comic
            })
            .collect(),
        ..result
    };
    Ok(Json(result))
}

fn cover_url(img_host: Option<&str>, comic_id: &str, fallback: &str) -> String {
    img_host
        .filter(|host| !host.is_empty() && !comic_id.is_empty())
        .map(|host| {
            format!(
                "{}/media/albums/{}_3x4.jpg",
                host.trim_end_matches('/'),
                comic_id
            )
        })
        .unwrap_or_else(|| fallback.to_string())
}
