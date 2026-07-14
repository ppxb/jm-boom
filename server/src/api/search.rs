use crate::{
    api::comic_summary::ComicSummaryResponse, domain::comic::ComicDetail, jm::JmResult, AppState,
};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

const SEARCH_PAGE_SIZE: u32 = 80;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub keyword: String,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_sort_by", rename = "sortBy")]
    pub sort_by: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    paging: SearchPaging,
    items: Vec<ComicSummaryResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchPaging {
    page: u32,
    pages: u32,
    total: u32,
    has_reached_max: bool,
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
) -> JmResult<Json<SearchResponse>> {
    let keyword = query.keyword.trim().to_string();
    let page = query.page.max(1);
    if keyword.is_empty() {
        return Ok(Json(search_response(page, 0, Vec::new())));
    }

    if page == 1 && is_comic_id(&keyword) {
        let comic_id = keyword.clone();
        if let Ok(detail) = app
            .comics
            .request(move |client, endpoint| {
                let comic_id = comic_id.clone();
                Box::pin(async move { client.get_comic_detail(endpoint, &comic_id).await })
            })
            .await
        {
            return Ok(Json(search_response(
                page,
                1,
                vec![search_comic_from_detail(detail)],
            )));
        }
    }

    let order = match query.sort_by {
        2 => "mv",
        3 => "mp",
        4 => "tf",
        _ => "mr",
    }
    .to_string();
    let result = app
        .comics
        .request(move |client, endpoint| {
            let keyword = keyword.clone();
            let order = order.clone();
            Box::pin(async move { client.search(endpoint, &keyword, page, &order).await })
        })
        .await?;

    if result.content.is_empty() {
        if let Some(redirect_id) = result.redirect_aid.filter(|id| is_comic_id(id)) {
            let detail = app
                .comics
                .request(move |client, endpoint| {
                    let comic_id = redirect_id.clone();
                    Box::pin(async move { client.get_comic_detail(endpoint, &comic_id).await })
                })
                .await?;
            return Ok(Json(search_response(
                page,
                1,
                vec![search_comic_from_detail(detail)],
            )));
        }
    }

    let total = result.total;
    let items = result
        .content
        .into_iter()
        .map(|comic| {
            ComicSummaryResponse::new(
                comic.id,
                comic.name,
                comic.author,
                comic.description,
                comic.image,
                comic.tags,
            )
        })
        .collect();
    Ok(Json(search_response(page, total, items)))
}

fn search_comic_from_detail(detail: ComicDetail) -> ComicSummaryResponse {
    ComicSummaryResponse::new(
        detail.id,
        detail.title,
        detail.authors.join(" / "),
        detail.description,
        detail.image,
        detail.tags,
    )
}

fn search_response(page: u32, total: u32, items: Vec<ComicSummaryResponse>) -> SearchResponse {
    let pages = total.div_ceil(SEARCH_PAGE_SIZE);
    SearchResponse {
        paging: SearchPaging {
            page,
            pages,
            total,
            has_reached_max: page >= pages,
        },
        items,
    }
}

fn is_comic_id(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|character| character.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::search_response;

    #[test]
    fn serializes_empty_search_response_contract() {
        let body = serde_json::to_value(search_response(1, 0, Vec::new()))
            .expect("serialize empty search response");
        assert_eq!(
            body,
            serde_json::json!({
                "paging": {
                    "page": 1,
                    "pages": 0,
                    "total": 0,
                    "hasReachedMax": true
                },
                "items": []
            })
        );
    }
}
