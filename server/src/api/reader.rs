use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
pub struct ManifestResponse {
    chapter_id: String,
    pages: Vec<PageInfo>,
}

#[derive(Serialize)]
pub struct PageInfo {
    page: u32,
    url: String,
}

/// 获取章节清单
pub async fn get_manifest(
    Path(chapter_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ManifestResponse>, AppError> {
    tracing::info!("Getting manifest for chapter: {}", chapter_id);

    // TODO: 从 JM API 获取章节信息
    // 临时返回测试数据
    let pages = (1..=10)
        .map(|i| PageInfo {
            page: i,
            url: format!("/api/reader/{}/pages/{}", chapter_id, i),
        })
        .collect();

    // 触发预加载（异步，不阻塞响应）
    let preloader = state.preloader.clone();
    let chapter_id_clone = chapter_id.clone();
    tokio::spawn(async move {
        if let Err(e) = preloader.preload_chapter(&chapter_id_clone, 10).await {
            tracing::error!("Preload failed: {}", e);
        }
    });

    Ok(Json(ManifestResponse {
        chapter_id,
        pages,
    }))
}

/// 获取图片
pub async fn get_page(
    Path((chapter_id, page)): Path<(String, u32)>,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    tracing::debug!("Getting page: chapter={}, page={}", chapter_id, page);

    let cache_key = format!("{}:{}", chapter_id, page);

    // 尝试从缓存获取
    if let Some(cached_data) = state.cache.get(&cache_key).await? {
        return Ok((
            StatusCode::OK,
            [("Content-Type", "image/webp"), ("Cache-Control", "public, max-age=31536000")],
            cached_data,
        )
            .into_response());
    }

    // TODO: 缓存未命中，从 JM 下载并处理
    // 临时返回 404
    Err(AppError::NotFound("Image not in cache".to_string()))
}

// 错误处理
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Internal(anyhow::Error),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Internal(err) => {
                tracing::error!("Internal error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        (status, message).into_response()
    }
}
