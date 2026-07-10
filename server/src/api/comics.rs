use axum::{extract::Path, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct ComicDetail {
    id: String,
    title: String,
    // TODO: 添加更多字段
}

pub async fn get_detail(Path(id): Path<String>) -> Json<ComicDetail> {
    // TODO: 从 JM API 获取详情
    Json(ComicDetail {
        id,
        title: "Test Comic".to_string(),
    })
}

pub async fn get_comments(Path(_id): Path<String>) -> Json<Vec<String>> {
    // TODO: 实现评论获取
    Json(vec![])
}
