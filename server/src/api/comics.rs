use crate::jm::{Chapter, ComicDetail, JmClient, JmResult};
use axum::{extract::Path, Json};
use serde::Deserialize;

/// Get comic detail
pub async fn get_comic_detail(Path(comic_id): Path<String>) -> JmResult<Json<ComicDetail>> {
    let client = JmClient::new()?;
    let endpoint = "https://www.cdnhjk.net"; // TODO: 从配置获取

    let detail: ComicDetail = client
        .get(endpoint, &format!("album/{}", comic_id), &[])
        .await?;

    Ok(Json(detail))
}

/// Get comic chapters
pub async fn get_comic_chapters(Path(comic_id): Path<String>) -> JmResult<Json<Vec<Chapter>>> {
    let client = JmClient::new()?;
    let endpoint = "https://www.cdnhjk.net"; // TODO: 从配置获取

    #[derive(Deserialize)]
    struct ChapterResponse {
        #[serde(default)]
        list: Vec<Chapter>,
    }

    let response: ChapterResponse = client
        .get(endpoint, &format!("chapter/{}", comic_id), &[])
        .await?;

    Ok(Json(response.list))
}
