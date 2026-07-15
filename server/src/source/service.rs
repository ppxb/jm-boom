use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Duration,
};

use thiserror::Error;
use tokio::sync::{Mutex, RwLock, Semaphore};

use super::{
    Chapter, CompiledSource, FilterValue, Listing, Manga, MangaPageResult, Page, PageContent,
    PageContext, SourceInstance, SourceRegistry, SourceRuntimeError,
};
use crate::{
    cache::{CachedReaderPage, ImageCache},
    expiring_cache::ExpiringCache,
    image_work::{ImageWorkBudget, ImageWorkPriority},
    keyed_lock::KeyedLock,
    reader::PageImageFormat,
};
use once_cell::sync::Lazy;

const DEFAULT_MAX_CONCURRENT_EXECUTIONS: usize = 8;
const PAGE_ASSET_TTL: Duration = Duration::from_secs(30 * 60);
const MAX_PAGE_ASSETS: usize = 8_192;
const SOURCE_PAGE_CACHE_VERSION: u8 = 1;

static SOURCE_PAGE_LOCKS: Lazy<KeyedLock> = Lazy::new(KeyedLock::new);

type CacheKey = (String, [u8; 16]);

pub struct SourceService {
    registry: Arc<SourceRegistry>,
    compiled: RwLock<HashMap<CacheKey, Arc<CompiledSource>>>,
    compile_lock: Mutex<()>,
    execution_limit: Semaphore,
    page_assets: ExpiringCache<SourcePageAsset>,
    cache: Arc<ImageCache>,
    image_work: ImageWorkBudget,
}

#[derive(Clone)]
struct SourcePageAsset {
    url: String,
    context: Option<PageContext>,
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
    #[error("source returned an in-memory page that is not supported by the HTTP bridge")]
    UnsupportedInMemoryPage,
    #[error("source page asset was not found or expired")]
    PageAssetNotFound,
    #[error("source page image is invalid: {0}")]
    InvalidImage(String),
    #[error("source page cache failed: {0}")]
    Cache(#[source] anyhow::Error),
}

impl SourceService {
    pub fn new(
        registry: Arc<SourceRegistry>,
        cache: Arc<ImageCache>,
        image_work: ImageWorkBudget,
    ) -> Self {
        Self::with_concurrency(
            registry,
            cache,
            image_work,
            DEFAULT_MAX_CONCURRENT_EXECUTIONS,
        )
    }

    pub fn with_concurrency(
        registry: Arc<SourceRegistry>,
        cache: Arc<ImageCache>,
        image_work: ImageWorkBudget,
        max_concurrent: usize,
    ) -> Self {
        Self {
            registry,
            compiled: RwLock::new(HashMap::new()),
            compile_lock: Mutex::new(()),
            execution_limit: Semaphore::new(max_concurrent.max(1)),
            page_assets: ExpiringCache::new(PAGE_ASSET_TTL, MAX_PAGE_ASSETS),
            cache,
            image_work,
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
        let pages = self
            .execute(source_id, move |source| source.get_pages(manga, chapter))
            .await?;
        if pages
            .iter()
            .any(|page| matches!(&page.content, PageContent::Image(_)))
        {
            return Err(SourceServiceError::UnsupportedInMemoryPage);
        }
        Ok(pages)
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

    pub async fn register_page_asset(
        &self,
        source_id: &str,
        url: String,
        context: Option<PageContext>,
    ) -> Result<String, SourceServiceError> {
        let package = self
            .registry
            .get(source_id)
            .await
            .ok_or_else(|| SourceServiceError::NotInstalled(source_id.into()))?;
        let token = page_asset_token(
            source_id,
            package.wasm_fingerprint(),
            &url,
            context.as_ref(),
        );
        self.page_assets
            .insert(
                page_asset_key(source_id, &token),
                SourcePageAsset { url, context },
            )
            .await;
        Ok(token)
    }

    pub async fn materialize_page_asset(
        &self,
        source_id: &str,
        token: &str,
        priority: ImageWorkPriority,
    ) -> Result<CachedReaderPage, SourceServiceError> {
        let cache_key = source_page_cache_key(source_id, token);
        if let Some(cached) = self
            .cache
            .get_reader_page(&cache_key)
            .await
            .map_err(SourceServiceError::Cache)?
        {
            return Ok(cached);
        }
        let asset_key = page_asset_key(source_id, token);
        let asset = self
            .page_assets
            .get(&asset_key)
            .await
            .ok_or(SourceServiceError::PageAssetNotFound)?;

        let _guard = SOURCE_PAGE_LOCKS.lock(&cache_key).await;
        if let Some(cached) = self
            .cache
            .get_reader_page(&cache_key)
            .await
            .map_err(SourceServiceError::Cache)?
        {
            return Ok(cached);
        }

        let package = self
            .registry
            .get(source_id)
            .await
            .ok_or_else(|| SourceServiceError::NotInstalled(source_id.into()))?;
        let _decode = if package.capabilities.processes_pages {
            Some(self.image_work.acquire_decode(priority).await)
        } else {
            None
        };
        let _network = self.image_work.acquire_network(priority).await;
        let url = asset.url;
        let context = asset.context;
        let data = self
            .execute(source_id, move |source| {
                source.materialize_page_image(url, context)
            })
            .await?;
        let format = PageImageFormat::detect(&data)
            .map_err(|error| SourceServiceError::InvalidImage(error.to_string()))?;
        self.cache
            .put_reader_page(&cache_key, format, &data)
            .await
            .map_err(SourceServiceError::Cache)?;
        Ok(CachedReaderPage { data, format })
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

fn page_asset_token(
    source_id: &str,
    fingerprint: [u8; 16],
    url: &str,
    context: Option<&PageContext>,
) -> String {
    let mut input = Vec::new();
    input.extend_from_slice(source_id.as_bytes());
    input.extend_from_slice(&fingerprint);
    input.extend_from_slice(url.as_bytes());
    if let Some(context) = context {
        for (key, value) in context.iter().collect::<BTreeMap<_, _>>() {
            input.extend_from_slice(key.as_bytes());
            input.push(0);
            input.extend_from_slice(value.as_bytes());
            input.push(0xff);
        }
    }
    format!("{:x}", md5::compute(input))
}

fn page_asset_key(source_id: &str, token: &str) -> String {
    format!("{source_id}:{token}")
}

fn source_page_cache_key(source_id: &str, token: &str) -> String {
    format!("source:{source_id}:{token}-v{SOURCE_PAGE_CACHE_VERSION}")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::page_asset_token;

    #[test]
    fn page_asset_tokens_are_stable_and_include_context_and_source_version() {
        let mut first_context = HashMap::new();
        first_context.insert("chapter".into(), "10".into());
        first_context.insert("page".into(), "2".into());
        let mut reordered_context = HashMap::new();
        reordered_context.insert("page".into(), "2".into());
        reordered_context.insert("chapter".into(), "10".into());

        let first = page_asset_token(
            "zh.example",
            [1; 16],
            "https://example.com/2.jpg",
            Some(&first_context),
        );
        let reordered = page_asset_token(
            "zh.example",
            [1; 16],
            "https://example.com/2.jpg",
            Some(&reordered_context),
        );
        let changed_context = page_asset_token(
            "zh.example",
            [1; 16],
            "https://example.com/2.jpg",
            Some(&HashMap::from([("page".into(), "3".into())])),
        );
        let changed_source = page_asset_token(
            "zh.example",
            [2; 16],
            "https://example.com/2.jpg",
            Some(&first_context),
        );

        assert_eq!(first, reordered);
        assert_ne!(first, changed_context);
        assert_ne!(first, changed_source);
    }
}
