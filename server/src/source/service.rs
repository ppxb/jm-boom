use std::{collections::HashMap, sync::Arc};

use thiserror::Error;
use tokio::sync::{Mutex, RwLock, Semaphore};

use super::{
    Chapter, CompiledSource, FilterValue, Listing, Manga, MangaPageResult, Page, SourceInstance,
    SourceRegistry, SourceRuntimeError,
};

const DEFAULT_MAX_CONCURRENT_EXECUTIONS: usize = 8;

type CacheKey = (String, [u8; 16]);

pub struct SourceService {
    registry: Arc<SourceRegistry>,
    compiled: RwLock<HashMap<CacheKey, Arc<CompiledSource>>>,
    compile_lock: Mutex<()>,
    execution_limit: Semaphore,
}

#[derive(Debug, Error)]
pub enum SourceServiceError {
    #[error("source is not installed: {0}")]
    NotInstalled(String),
    #[error(transparent)]
    Runtime(#[from] SourceRuntimeError),
    #[error("source worker failed: {0}")]
    Worker(String),
    #[error("source service is shutting down")]
    ShuttingDown,
}

impl SourceService {
    pub fn new(registry: Arc<SourceRegistry>) -> Self {
        Self::with_concurrency(registry, DEFAULT_MAX_CONCURRENT_EXECUTIONS)
    }

    pub fn with_concurrency(registry: Arc<SourceRegistry>, max_concurrent: usize) -> Self {
        Self {
            registry,
            compiled: RwLock::new(HashMap::new()),
            compile_lock: Mutex::new(()),
            execution_limit: Semaphore::new(max_concurrent.max(1)),
        }
    }

    pub async fn search(
        &self,
        source_id: &str,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult, SourceServiceError> {
        self.execute(source_id, move |source| source.search(query, page, filters))
            .await
    }

    pub async fn update_manga(
        &self,
        source_id: &str,
        manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga, SourceServiceError> {
        self.execute(source_id, move |source| {
            source.update_manga(manga, needs_details, needs_chapters)
        })
        .await
    }

    pub async fn get_pages(
        &self,
        source_id: &str,
        manga: Manga,
        chapter: Chapter,
    ) -> Result<Vec<Page>, SourceServiceError> {
        self.execute(source_id, move |source| source.get_pages(manga, chapter))
            .await
    }

    pub async fn get_listing(
        &self,
        source_id: &str,
        listing: Listing,
        page: i32,
    ) -> Result<MangaPageResult, SourceServiceError> {
        self.execute(source_id, move |source| source.get_listing(listing, page))
            .await
    }

    async fn execute<T, F>(&self, source_id: &str, operation: F) -> Result<T, SourceServiceError>
    where
        T: Send + 'static,
        F: FnOnce(&mut SourceInstance) -> Result<T, SourceRuntimeError> + Send + 'static,
    {
        let _permit = self
            .execution_limit
            .acquire()
            .await
            .map_err(|_| SourceServiceError::ShuttingDown)?;
        let package = self
            .registry
            .get(source_id)
            .await
            .ok_or_else(|| SourceServiceError::NotInstalled(source_id.into()))?;
        let compiled = self.compiled_source(package).await?;
        tokio::task::spawn_blocking(move || {
            let mut source = compiled.instantiate()?;
            operation(&mut source)
        })
        .await
        .map_err(|error| SourceServiceError::Worker(error.to_string()))?
        .map_err(SourceServiceError::from)
    }

    async fn compiled_source(
        &self,
        package: Arc<super::SourcePackage>,
    ) -> Result<Arc<CompiledSource>, SourceServiceError> {
        let source_id = package.manifest.info.id.clone();
        let key = (source_id.clone(), package.wasm_fingerprint());
        if let Some(compiled) = self.compiled.read().await.get(&key).cloned() {
            return Ok(compiled);
        }
        let _compile = self.compile_lock.lock().await;
        if let Some(compiled) = self.compiled.read().await.get(&key).cloned() {
            return Ok(compiled);
        }

        let compiled = Arc::new(
            tokio::task::spawn_blocking(move || CompiledSource::compile(&package))
                .await
                .map_err(|error| SourceServiceError::Worker(error.to_string()))??,
        );
        let mut cache = self.compiled.write().await;
        cache.retain(|(installed_id, _), _| installed_id != &source_id);
        Ok(cache.entry(key).or_insert(compiled).clone())
    }
}
