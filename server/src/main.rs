mod api;
mod cache;
mod config;
mod jm;
mod reader;
mod storage;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, Method},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    cors::CorsLayer,
    compression::CompressionLayer,
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
    pub preloader: std::sync::Arc<reader::ChapterPreloader>,
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

        // 初始化预加载器
        let preloader = std::sync::Arc::new(reader::ChapterPreloader::new(cache.clone()));

        Ok(Self {
            db,
            cache,
            preloader,
        })
    }
}
