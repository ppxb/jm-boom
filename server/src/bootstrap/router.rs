use crate::{
    api,
    bootstrap::{config::AppConfig, state::AppState},
    http_error::HttpError,
};
use axum::{
    extract::DefaultBodyLimit, http::StatusCode, response::IntoResponse, routing::get, Router,
};
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

pub(crate) fn build(config: &AppConfig, state: AppState) -> anyhow::Result<Router> {
    let index_file = config.static_dir.join("index.html");
    let static_service =
        ServeDir::new(&config.static_dir).not_found_service(ServeFile::new(index_file));

    let mut app = Router::new()
        .nest("/api", api::routes().fallback(api_not_found))
        .route("/health", get(health_check))
        .fallback_service(static_service)
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024));

    if let Some(cors) = config.cors_layer()? {
        app = app.layer(cors);
    }

    Ok(app)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn api_not_found() -> impl IntoResponse {
    HttpError::new(StatusCode::NOT_FOUND, "API 路由不存在", false)
}
