use axum::{Router, routing::{get, post}};
use crate::AppState;

mod auth;
mod comics;
mod home;
mod reader;
mod search;

pub fn routes() -> Router<AppState> {
    Router::new()
        // 搜索
        .route("/search", get(search::search_comics))

        // 漫画详情
        .route("/comics/:id", get(comics::get_comic_detail))
        .route("/comics/:id/chapters", get(comics::get_comic_chapters))

        // 主页
        .route("/home/feed", get(home::get_home_feed))

        // 阅读器
        .route("/reader/:chapter_id/manifest", get(reader::get_manifest))
        .route("/reader/:chapter_id/pages/:page", get(reader::get_page))

        // 认证
        .route("/auth/login", post(auth::login))
        .route("/auth/session", get(auth::get_session))
}
