use crate::jm::{HomeSection, JmClient, JmResult};
use axum::Json;

/// Get home feed sections
pub async fn get_home_feed() -> JmResult<Json<Vec<HomeSection>>> {
    let client = JmClient::new()?;
    let endpoint = crate::config::get_endpoint();

    let sections = client.get_home_feed(&endpoint).await?;
    Ok(Json(sections))
}
