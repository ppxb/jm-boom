use crate::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub(crate) mod auth;
mod comics;
mod downloads;
mod home;
mod reader;
mod search;
mod settings;

pub fn routes() -> Router<AppState> {
    Router::new()
        // 搜索
        .route("/search", get(search::search_comics))
        // 漫画详情
        .route("/comics/:id", get(comics::get_comic_detail))
        .route("/comics/:id/chapters", get(comics::get_comic_chapters))
        .route("/comics/:id/comments", get(comics::get_comments))
        .route("/comics/:id/favorite", post(comics::toggle_favorite))
        .route("/favorites", get(comics::get_favorites))
        // 服务端离线缓存任务
        .route("/downloads", get(downloads::list))
        .route("/downloads", post(downloads::enqueue))
        .route("/downloads/:id/pause", post(downloads::pause))
        .route("/downloads/:id/resume", post(downloads::resume))
        .route("/downloads/:id/cancel", post(downloads::cancel))
        .route("/downloads/:id", delete(downloads::remove))
        // 主页
        .route("/home/feed", get(home::get_home_feed))
        .route("/home/sections", get(home::get_home_section_list))
        .route("/home/weekly/filters", get(home::get_week_filters))
        .route("/home/weekly/items", get(home::get_week_items))
        // 阅读器
        .route("/reader/:chapter_id/manifest", get(reader::get_manifest))
        .route("/reader/:chapter_id/pages/:page", get(reader::get_page))
        // 认证
        .route("/auth/login", post(auth::login))
        .route("/auth/session", get(auth::get_session))
        .route("/auth/session", delete(auth::clear_session))
        .route("/auth/sign-in", get(auth::get_sign_in_data))
        .route("/auth/sign-in", post(auth::sign_in))
        // 服务端上游端点管理
        .route("/settings/endpoints", get(settings::get_endpoints))
        .route("/settings/endpoints", put(settings::set_endpoint))
        .route("/settings/endpoints/probe", post(settings::probe_endpoints))
        .route("/settings/system", get(settings::get_system_info))
        .route("/settings/cache", delete(settings::clear_cache))
}
