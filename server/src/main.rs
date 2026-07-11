mod api;
mod cache;
mod download;
mod endpoint;
mod image_work;
mod jm;
mod reader;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderName, HeaderValue, Method, StatusCode},
    response::IntoResponse,
    Json, Router,
};
use std::net::SocketAddr;
use std::{future::Future, pin::Pin};
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,jm_boom_server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("JM Boom 服务端正在启动");

    // 初始化应用状态
    let state = AppState::new().await?;

    let static_dir = std::env::var("JM_BOOM_STATIC_DIR").unwrap_or_else(|_| "./static".into());
    let index_file = std::path::Path::new(&static_dir).join("index.html");
    let static_service = ServeDir::new(&static_dir).not_found_service(ServeFile::new(index_file));

    // 构建路由
    let mut app = Router::new()
        // API 路由
        .nest("/api", api::routes().fallback(api_not_found))
        // 健康检查
        .route("/health", axum::routing::get(health_check))
        // React 静态资源与 SPA 路由回退
        .fallback_service(static_service)
        // 注入状态
        .with_state(state)
        // 中间件
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024)); // 50MB

    if let Some(cors) = cors_layer_from_env()? {
        app = app.layer(cors);
    }

    // 启动服务器
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn api_not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "API 路由不存在" })),
    )
}

fn cors_layer_from_env() -> anyhow::Result<Option<CorsLayer>> {
    let configured = match std::env::var("JM_BOOM_CORS_ORIGINS") {
        Ok(value) => value,
        Err(std::env::VarError::NotPresent) => return Ok(None),
        Err(error) => return Err(anyhow::anyhow!("invalid JM_BOOM_CORS_ORIGINS: {error}")),
    };
    let origins = parse_cors_origins(&configured)?;

    if origins.is_empty() {
        return Ok(None);
    }

    tracing::info!(origins = ?origins, "cross-origin access enabled");
    Ok(Some(
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([
                header::CONTENT_TYPE,
                HeaderName::from_static("x-jm-boom-image-priority"),
            ]),
    ))
}

fn parse_cors_origins(configured: &str) -> anyhow::Result<Vec<HeaderValue>> {
    let mut origins = Vec::new();

    for value in configured
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let url = reqwest::Url::parse(value)
            .map_err(|error| anyhow::anyhow!("invalid CORS origin {value:?}: {error}"))?;
        let is_http = matches!(url.scheme(), "http" | "https");
        let is_origin_only = url.path() == "/"
            && url.query().is_none()
            && url.fragment().is_none()
            && url.username().is_empty()
            && url.password().is_none();

        if !is_http || url.host_str().is_none() || !is_origin_only {
            return Err(anyhow::anyhow!(
                "CORS origin must be an http(s) origin without path, query, credentials, or fragment: {value:?}"
            ));
        }

        let origin = HeaderValue::from_str(&url.origin().ascii_serialization())?;
        if !origins.contains(&origin) {
            origins.push(origin);
        }
    }

    Ok(origins)
}

// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub cache: std::sync::Arc<cache::ImageCache>,
    pub image_work: image_work::ImageWorkBudget,
    pub endpoints: std::sync::Arc<endpoint::EndpointManager>,
    pub jm: std::sync::Arc<jm::JmClient>,
    pub downloads: std::sync::Arc<download::DownloadManager>,
    pub access_password: Option<std::sync::Arc<str>>,
}

impl AppState {
    async fn new() -> anyhow::Result<Self> {
        // 确保数据目录存在
        let data_dir = std::path::PathBuf::from("./data");
        std::fs::create_dir_all(&data_dir)?;
        tracing::info!("Data directory: {}", data_dir.display());

        // 初始化数据库
        let db_path = data_dir.join("app.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        tracing::info!("Database URL: {}", db_url);

        let db = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(10)
            .connect(&db_url)
            .await?;

        // 运行迁移
        sqlx::migrate!("./migrations").run(&db).await?;

        // 初始化缓存
        let cache_config = cache::CacheConfig::from_env()?;
        let cache = std::sync::Arc::new(cache::ImageCache::new(db.clone(), cache_config).await?);
        cache.start_maintenance();
        let image_work = image_work::ImageWorkBudget::new();

        let endpoints = std::sync::Arc::new(endpoint::EndpointManager::new(db.clone()).await?);
        let jm = std::sync::Arc::new(jm::JmClient::new()?);
        let downloads = std::sync::Arc::new(
            download::DownloadManager::new(
                db.clone(),
                jm.clone(),
                endpoints.clone(),
                cache.clone(),
                image_work.clone(),
            )
            .await?,
        );
        let access_password = std::env::var("JM_BOOM_ACCESS_PASSWORD")
            .ok()
            .filter(|password| !password.is_empty())
            .map(std::sync::Arc::<str>::from);
        tracing::info!(enabled = access_password.is_some(), "轻量访问门禁配置完成");

        let endpoint_manager = endpoints.clone();
        tokio::spawn(async move {
            let state = endpoint_manager.refresh().await;
            tracing::info!(endpoint = %state.current_endpoint, "endpoint probe completed");
        });
        let download_manager = downloads.clone();
        tokio::spawn(async move {
            download_manager.resume_pending().await;
        });

        Ok(Self {
            db,
            cache,
            image_work,
            endpoints,
            jm,
            downloads,
            access_password,
        })
    }

    pub async fn jm_request<T, F>(&self, operation: F) -> jm::JmResult<T>
    where
        F: for<'a> Fn(
            &'a jm::JmClient,
            &'a str,
        ) -> Pin<Box<dyn Future<Output = jm::JmResult<T>> + Send + 'a>>,
    {
        endpoint::request_with_failover(&self.jm, &self.endpoints, operation)
            .await
            .map(|(_, value)| value)
    }

    pub async fn img_host(&self) -> Option<String> {
        self.jm_request(|client, endpoint| Box::pin(client.get_img_host(endpoint)))
            .await
            .ok()
    }
}
