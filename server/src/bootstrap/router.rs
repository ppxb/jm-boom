use crate::{
    api,
    bootstrap::{config::AppConfig, state::AppState},
    http_error::HttpError,
};
use axum::{
    body::Body,
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

const STATIC_DIR: &str = "./static";
const STATIC_INDEX_FILE: &str = "./static/index.html";

pub(crate) fn build(config: &AppConfig, state: AppState) -> anyhow::Result<Router> {
    let static_service =
        ServeDir::new(STATIC_DIR).not_found_service(ServeFile::new(STATIC_INDEX_FILE));

    let mut app = Router::new()
        .nest("/api", api::routes().fallback(api_not_found))
        .route("/health", get(health_check))
        .fallback_service(static_service)
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .layer(middleware::from_fn(static_cache_control));

    if let Some(cors) = config.cors_layer()? {
        app = app.layer(cors);
    }

    Ok(app)
}

const ASSET_CACHE_CONTROL: &str = "public, max-age=31536000, immutable";
const DOCUMENT_CACHE_CONTROL: &str = "no-cache";

/// 带内容哈希的构建产物长缓存，入口 HTML 与 SPA 回退走协商缓存。
async fn static_cache_control(request: Request<Body>, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;

    if response.headers().contains_key(header::CACHE_CONTROL) {
        return response;
    }

    let is_document = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/html"));

    let value = if path.starts_with("/api/") || path == "/health" {
        None
    } else if is_document {
        Some(DOCUMENT_CACHE_CONTROL)
    } else if path.starts_with("/assets/") {
        Some(ASSET_CACHE_CONTROL)
    } else {
        Some(DOCUMENT_CACHE_CONTROL)
    };

    if let Some(value) = value {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
    }

    response
}

async fn health_check() -> &'static str {
    "OK"
}

async fn api_not_found() -> impl IntoResponse {
    HttpError::new(StatusCode::NOT_FOUND, "API 路由不存在", false)
}
