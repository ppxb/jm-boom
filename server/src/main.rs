mod api;
mod cache;
mod download;
mod endpoint;
mod jm;
mod reader;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, Method},
    Router,
};
use std::net::SocketAddr;
use std::{future::Future, pin::Pin};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
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

    tracing::info!("JM Boom Server starting...");

    // 初始化应用状态
    let state = AppState::new().await?;

    // 构建路由
    let app = Router::new()
        // API 路由
        .nest("/api", api::routes())
        // 健康检查
        .route("/health", axum::routing::get(health_check))
        // 注入状态
        .with_state(state)
        // 中间件
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
        )
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024)); // 50MB

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

// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub cache: std::sync::Arc<cache::ImageCache>,
    pub endpoints: std::sync::Arc<endpoint::EndpointManager>,
    pub jm: std::sync::Arc<jm::JmClient>,
    pub session: std::sync::Arc<tokio::sync::RwLock<Option<api::auth::UserProfile>>>,
    pub downloads: std::sync::Arc<download::DownloadManager>,
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
        let cache = std::sync::Arc::new(cache::ImageCache::new(db.clone()).await?);

        let endpoints = std::sync::Arc::new(endpoint::EndpointManager::new(db.clone()).await?);
        let jm = std::sync::Arc::new(jm::JmClient::new()?);
        let session = std::sync::Arc::new(tokio::sync::RwLock::new(None));
        let downloads = std::sync::Arc::new(
            download::DownloadManager::new(
                db.clone(),
                jm.clone(),
                endpoints.clone(),
                cache.clone(),
            )
            .await?,
        );

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
            endpoints,
            jm,
            session,
            downloads,
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
