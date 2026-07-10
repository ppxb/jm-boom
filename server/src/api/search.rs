use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct SearchResult {
    items: Vec<String>,
}

pub async fn search_comics() -> Json<SearchResult> {
    // TODO: 实现搜索
    Json(SearchResult { items: vec![] })
}
