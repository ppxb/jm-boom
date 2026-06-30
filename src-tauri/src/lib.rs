mod api;
mod diagnostics;
mod download;
mod plugin_codec;
mod reader;
mod storage;
mod updater;

use api::{
    ApiEndpointProbe, ComicCommentsResult, ComicDetailResult, FavoriteListResult,
    FavoriteToggleResult, HomeFeedResult, HomeSectionListResult, LoginResult, RemoteSettingResult,
    SearchResultContract, SignInDataResult, SignInResult, WeekFiltersResult, WeekItemsResult,
};
use reader::{ComicReadManifestResult, ComicReadPageResult, ReaderCacheStatsResult};
use std::collections::HashMap;

#[tauri::command]
async fn get_remote_setting(endpoint: Option<String>) -> Result<RemoteSettingResult, String> {
    api::get_remote_setting(endpoint)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn discover_api_endpoints() -> Result<Vec<ApiEndpointProbe>, String> {
    api::discover_api_endpoints()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn search_comics(
    keyword: String,
    page: Option<u32>,
    extern_payload: Option<HashMap<String, serde_json::Value>>,
    endpoint: Option<String>,
) -> Result<SearchResultContract, String> {
    api::search_comics(keyword, page, extern_payload, endpoint)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_home_feed(endpoint: Option<String>) -> Result<HomeFeedResult, String> {
    api::get_home_feed(endpoint)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn get_home_section_list(
    mode: String,
    page: Option<u32>,
    section_id: Option<String>,
    section_title: Option<String>,
    slug: Option<String>,
    section_type: Option<String>,
    filter_value: Option<String>,
    category: Option<String>,
    week: Option<String>,
    order: Option<String>,
    endpoint: Option<String>,
) -> Result<HomeSectionListResult, String> {
    api::get_home_section_list(
        mode,
        page,
        section_id,
        section_title,
        slug,
        section_type,
        filter_value,
        category,
        week,
        order,
        endpoint,
    )
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_week_filters(endpoint: Option<String>) -> Result<WeekFiltersResult, String> {
    api::get_week_filters(endpoint)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_week_items(
    page: Option<u32>,
    category_id: String,
    type_id: String,
    endpoint: Option<String>,
) -> Result<WeekItemsResult, String> {
    api::get_week_items(page, category_id, type_id, endpoint)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_comic_detail(
    comic_id: String,
    endpoint: Option<String>,
) -> Result<ComicDetailResult, String> {
    api::get_comic_detail(comic_id, endpoint)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn toggle_comic_favorite(
    comic_id: String,
    current_favorite: bool,
    endpoint: Option<String>,
) -> Result<FavoriteToggleResult, String> {
    api::toggle_comic_favorite(comic_id, current_favorite, endpoint)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_favorite_comics(
    page: Option<u32>,
    folder_id: Option<String>,
    order: Option<String>,
    endpoint: Option<String>,
) -> Result<FavoriteListResult, String> {
    api::get_favorite_comics(page, folder_id, order, endpoint)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_comic_comments(
    comic_id: String,
    page: Option<u32>,
    endpoint: Option<String>,
) -> Result<ComicCommentsResult, String> {
    api::get_comic_comments(comic_id, page, endpoint)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn login(
    username: String,
    password: String,
    endpoint: Option<String>,
) -> Result<LoginResult, String> {
    api::login(username, password, endpoint)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_sign_in_data(
    user_id: u32,
    endpoint: Option<String>,
) -> Result<SignInDataResult, String> {
    api::get_sign_in_data(user_id, endpoint)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn sign_in(
    user_id: u32,
    daily_id: u32,
    endpoint: Option<String>,
) -> Result<SignInResult, String> {
    api::sign_in(user_id, daily_id, endpoint)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn clear_session() {
    api::clear_session();
}

#[tauri::command]
fn configure_network_proxy(
    mode: String,
    host: Option<String>,
    port: Option<u16>,
) -> Result<(), String> {
    api::configure_network_proxy(mode, host, port).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_reader_cache_stats(
    app: tauri::AppHandle,
    cache_limit_bytes: Option<u64>,
) -> Result<ReaderCacheStatsResult, String> {
    reader::get_reader_cache_stats(&app, cache_limit_bytes).map_err(|error| error.to_string())
}

#[tauri::command]
fn clear_reader_cache(
    app: tauri::AppHandle,
    cache_limit_bytes: Option<u64>,
) -> Result<ReaderCacheStatsResult, String> {
    reader::clear_reader_cache(&app, cache_limit_bytes).map_err(|error| error.to_string())
}

#[tauri::command]
fn open_reader_cache_dir(app: tauri::AppHandle) -> Result<(), String> {
    reader::open_reader_cache_dir(&app).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_diagnostics_info(app: tauri::AppHandle) -> Result<diagnostics::DiagnosticsInfo, String> {
    diagnostics::get_info(&app)
}

#[tauri::command]
fn open_diagnostics_log_dir(app: tauri::AppHandle) -> Result<(), String> {
    diagnostics::open_log_dir(&app)
}

#[tauri::command]
fn set_diagnostics_debug_logging(
    app: tauri::AppHandle,
    enabled: bool,
) -> Result<diagnostics::DiagnosticsInfo, String> {
    diagnostics::set_debug_logging_enabled(&app, enabled)
}

#[tauri::command]
async fn get_comic_read_manifest(
    read_id: String,
    endpoint: Option<String>,
) -> Result<ComicReadManifestResult, String> {
    reader::get_comic_read_manifest(read_id, endpoint)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_comic_read_page(
    app: tauri::AppHandle,
    read_id: String,
    index: u32,
    endpoint: Option<String>,
    request_origin: Option<String>,
    cache_limit_bytes: Option<u64>,
) -> Result<ComicReadPageResult, String> {
    reader::get_comic_read_page(
        &app,
        read_id,
        index,
        endpoint,
        request_origin,
        cache_limit_bytes,
    )
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
async fn enqueue_comic_download(
    app: tauri::AppHandle,
    request: download::EnqueueDownloadRequest,
) -> Result<download::DownloadTaskListResult, String> {
    download::enqueue_comic_download(app, request).await
}

#[tauri::command]
async fn list_download_tasks(
    app: tauri::AppHandle,
) -> Result<download::DownloadTaskListResult, String> {
    download::list_download_tasks(app).await
}

#[tauri::command]
async fn cancel_download_task(
    app: tauri::AppHandle,
    task_id: String,
) -> Result<download::DownloadTaskListResult, String> {
    download::cancel_download_task(app, task_id).await
}

#[tauri::command]
async fn pause_download_task(
    app: tauri::AppHandle,
    task_id: String,
) -> Result<download::DownloadTaskListResult, String> {
    download::pause_download_task(app, task_id).await
}

#[tauri::command]
async fn resume_download_task(
    app: tauri::AppHandle,
    task_id: String,
) -> Result<download::DownloadTaskListResult, String> {
    download::resume_download_task(app, task_id).await
}

#[tauri::command]
async fn remove_download_task(
    app: tauri::AppHandle,
    task_id: String,
) -> Result<download::DownloadTaskListResult, String> {
    download::remove_download_task(app, task_id).await
}

#[tauri::command]
async fn open_download_task_dir(app: tauri::AppHandle, task_id: String) -> Result<(), String> {
    download::open_download_task_dir(app, task_id).await
}

#[tauri::command]
fn open_download_root_dir(app: tauri::AppHandle) -> Result<(), String> {
    download::open_download_root_dir(app)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            let _ = diagnostics::init(&handle);
            if let Err(error) = tauri::async_runtime::block_on(storage::init(&handle)) {
                diagnostics::error(format!("Failed to initialize storage: {error}"));
                return Err(std::io::Error::other(error).into());
            }
            diagnostics::info("JM Boom started");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_remote_setting,
            discover_api_endpoints,
            search_comics,
            get_home_feed,
            get_home_section_list,
            get_week_filters,
            get_week_items,
            get_comic_detail,
            toggle_comic_favorite,
            get_favorite_comics,
            get_comic_comments,
            login,
            get_sign_in_data,
            sign_in,
            clear_session,
            configure_network_proxy,
            get_reader_cache_stats,
            clear_reader_cache,
            open_reader_cache_dir,
            get_diagnostics_info,
            open_diagnostics_log_dir,
            set_diagnostics_debug_logging,
            get_comic_read_manifest,
            get_comic_read_page,
            enqueue_comic_download,
            list_download_tasks,
            cancel_download_task,
            pause_download_task,
            resume_download_task,
            remove_download_task,
            open_download_task_dir,
            open_download_root_dir,
            updater::check_app_update,
            updater::install_app_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
