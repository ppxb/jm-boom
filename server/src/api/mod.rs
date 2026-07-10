use axum::{Router, routing::{get, post}};
use crate::AppState;

mod comics;
mod reader;
mod search;
mod auth;

pub fn routes() -> Router<AppState> {
    Router::new()
        // 搜索
        .route("/search", get(search::search_comics))

        // 漫画详情
        .route("/comics/:id", get(comics::get_detail))
        .route("/comics/:id/comments", get(comics::get_comments))

        // 阅读器
        .route("/reader/:chapter_id/manifest", get(reader::get_manifest))
        .route("/reader/:chapter_id/pages/:page", get(reader::get_page))

        // 认证
        .route("/auth/login", post(auth::login))
        .route("/auth/session", get(auth::get_session))
}
