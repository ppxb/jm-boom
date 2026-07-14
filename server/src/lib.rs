mod api;
mod application;
mod bootstrap;
mod cache;
mod domain;
mod download;
mod endpoint;
mod expiring_cache;
mod http_error;
mod image_work;
mod jm;
mod keyed_lock;
mod page_materializer;
mod reader;

pub(crate) use bootstrap::state::AppState;

pub async fn build_app() -> anyhow::Result<axum::Router> {
    let config = bootstrap::config::AppConfig::from_env();
    let state = AppState::new(&config).await?;
    bootstrap::router::build(&config, state)
}
