use crate::{
    api::comic_summary::ComicSummaryResponse,
    application::{ComicSearch, ComicSearchRequest},
    jm::JmResult,
    AppState,
};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

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

pub async fn search_comics(
    State(app): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> JmResult<Json<SearchResponse>> {
    app.comics
        .search_comics(ComicSearchRequest {
            keyword: query.keyword,
            page: query.page,
            sort_by: query.sort_by,
        })
        .await
        .map(SearchResponse::from)
        .map(Json)
}

impl From<ComicSearch> for SearchResponse {
    fn from(result: ComicSearch) -> Self {
        Self {
            paging: SearchPaging {
                page: result.page,
                pages: result.pages,
                total: result.total,
                has_reached_max: result.has_reached_max,
            },
            items: result.items.into_iter().map(Into::into).collect(),
        }
    }
}

fn default_page() -> u32 {
    1
}

fn default_sort_by() -> u8 {
    1
}

#[cfg(test)]
mod tests {
    use super::SearchResponse;
    use crate::application::ComicSearch;

    #[test]
    fn serializes_empty_search_response_contract() {
        let body = serde_json::to_value(SearchResponse::from(ComicSearch {
            page: 1,
            pages: 0,
            total: 0,
            has_reached_max: true,
            items: Vec::new(),
        }))
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
