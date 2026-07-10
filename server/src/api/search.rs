use crate::jm::{JmClient, JmResult};
use axum::{extract::Query, Json};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub keyword: String,
    #[serde(default = "default_page")]
    pub page: u32,
}

fn default_page() -> u32 {
    1
}

pub async fn search_comics(
    Query(query): Query<SearchQuery>,
) -> JmResult<Json<crate::jm::SearchResult>> {
    let client = JmClient::new()?;
    let endpoint = crate::config::get_endpoint();

    let keyword = query.keyword.trim();
    if keyword.is_empty() {
        return Ok(Json(crate::jm::SearchResult {
            total: 0,
            content: vec![],
            redirect_aid: None,
        }));
    }

    let result = client.search(&endpoint, keyword, query.page).await?;
    Ok(Json(result))
}
