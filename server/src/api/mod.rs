use crate::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;

const COLLECTION_PAGE_SIZE: u32 = 20;

#[derive(Debug, Deserialize)]
struct CollectionListQuery {
    #[serde(default = "default_collection_page")]
    page: u32,
}

fn default_collection_page() -> u32 {
    1
}

mod auth;
mod comic_dto;
mod comic_summary;
mod comics;
mod covers;
mod downloads;
mod favorites;
mod history;
mod home;
mod media;
mod reader;
mod search;
mod settings;

pub fn routes() -> Router<AppState> {
    Router::new()
        // 本地部署轻量访问门禁
        .route("/auth/config", get(auth::config))
        .route("/auth/login", post(auth::login))
        // 搜索
        .route("/search", get(search::search_comics))
        // 漫画详情
        .route("/comics/:id", get(comics::get_comic_detail))
        .route("/comics/:id/state", get(comics::get_comic_state))
        .route("/comics/:id/comments", get(comics::get_comments))
        .route("/covers/:id", get(covers::get_cover))
        // 单实例共享收藏
        .route("/favorites", get(favorites::list).delete(favorites::clear))
        .route(
            "/favorites/:id",
            put(favorites::upsert).delete(favorites::remove),
        )
        // 单实例共享阅读历史
        .route("/history", get(history::list).delete(history::clear))
        .route("/history/remove", post(history::remove_many))
        .route("/history/:id", put(history::upsert).delete(history::remove))
        // 服务端离线缓存任务
        .route("/downloads", get(downloads::list))
        .route("/downloads", post(downloads::enqueue))
        .route("/downloads/chapters", get(downloads::downloaded_chapters))
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
        // 服务端上游端点管理
        .route("/settings/endpoints", get(settings::get_endpoints))
        .route("/settings/endpoints", put(settings::set_endpoint))
        .route("/settings/endpoints/probe", post(settings::probe_endpoints))
        .route("/settings/system", get(settings::get_system_info))
        .route("/settings/cache", delete(settings::clear_cache))
        .route(
            "/settings/account",
            get(settings::get_account)
                .put(settings::update_account)
                .delete(settings::clear_account),
        )
}
