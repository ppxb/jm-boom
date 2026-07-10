use crate::jm::{JmClient, JmResult, SearchResult};
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

pub async fn search_comics(Query(query): Query<SearchQuery>) -> JmResult<Json<SearchResult>> {
    let client = JmClient::new()?;
    let endpoint = "https://www.cdnhjk.net"; // TODO: 从配置获取

    let result: SearchResult = client
        .get(
            endpoint,
            "search",
            &[
                ("search_query", query.keyword),
                ("page", query.page.to_string()),
            ],
        )
        .await?;

    Ok(Json(result))
}
