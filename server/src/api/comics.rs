use crate::jm::{ComicDetail, JmClient, JmResult};
use axum::{extract::Path, Json};

/// Get comic detail
pub async fn get_comic_detail(Path(comic_id): Path<String>) -> JmResult<Json<ComicDetail>> {
    let client = JmClient::new()?;
    let endpoint = crate::config::get_endpoint();

    let detail = client.get_comic_detail(&endpoint, &comic_id).await?;
    Ok(Json(detail))
}

/// Get comic chapters (same as series in detail)
pub async fn get_comic_chapters(
    Path(comic_id): Path<String>,
) -> JmResult<Json<Vec<crate::jm::Chapter>>> {
    let client = JmClient::new()?;
    let endpoint = crate::config::get_endpoint();

    let detail = client.get_comic_detail(&endpoint, &comic_id).await?;
    Ok(Json(detail.series))
}
