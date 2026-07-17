use crate::{
    application::{
        AccessGateService, ComicService, CoverService, DownloadService, FavoriteService,
        ReaderService, SettingsService,
    },
    bootstrap::config::AppConfig,
    cache, endpoint, image_work, jm, page_materializer,
};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) access_gate: Arc<AccessGateService>,
    pub(crate) comics: Arc<ComicService>,
    pub(crate) covers: Arc<CoverService>,
    pub(crate) reader: Arc<ReaderService>,
    pub(crate) downloads: Arc<DownloadService>,
    pub(crate) favorites: Arc<FavoriteService>,
    pub(crate) settings: Arc<SettingsService>,
}

impl AppState {
    pub(crate) async fn new(config: &AppConfig) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&config.data_dir)?;
        tracing::info!(data_dir = %config.data_dir.display(), "Data directory ready");

        let db_path = config.data_dir.join("app.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        tracing::info!("Database URL: {}", db_url);

        let db = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(10)
            .connect(&db_url)
            .await?;
        sqlx::migrate!("./migrations").run(&db).await?;

        let cache_config = cache::CacheConfig::from_env()?;
        let cache = Arc::new(cache::ImageCache::new(db.clone(), cache_config).await?);
        cache.start_maintenance();

        let endpoints = Arc::new(endpoint::EndpointManager::new(db.clone()).await?);
        let jm = Arc::new(jm::JmClient::new()?);
        let comics = Arc::new(ComicService::new(jm.clone(), endpoints.clone()));
        let page_materializer = Arc::new(page_materializer::PageMaterializer::new(
            jm.clone(),
            endpoints.clone(),
            cache.clone(),
            image_work::ImageWorkBudget::new(),
        ));
        let favorites = Arc::new(FavoriteService::new(db.clone()));
        let downloads = Arc::new(
            DownloadService::new(db, jm.clone(), endpoints.clone(), page_materializer.clone())
                .await?,
        );

        let covers = Arc::new(CoverService::new(
            cache.clone(),
            jm.clone(),
            endpoints.clone(),
        ));
        let reader = Arc::new(ReaderService::new(
            comics.clone(),
            page_materializer,
            downloads.clone(),
        ));
        let settings = Arc::new(SettingsService::new(endpoints.clone(), cache));
        let access_gate = Arc::new(AccessGateService::from_env());

        endpoints.start_maintenance();
        let download_service = downloads.clone();
        tokio::spawn(async move {
            download_service.resume_pending().await;
        });

        Ok(Self {
            access_gate,
            comics,
            covers,
            reader,
            downloads,
            favorites,
            settings,
        })
    }
}
