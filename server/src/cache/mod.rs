mod maintenance;
mod repository;
mod storage;

#[cfg(test)]
mod tests;

use crate::reader::{ImageDimensions, PageImageFormat};
use anyhow::{Context, Result};
use repository::CacheRepository;
use serde::Serialize;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use storage::CacheStorage;
use tokio::{
    sync::{Mutex, RwLock},
    time::{self, MissedTickBehavior},
};

pub const READER_CACHE_VERSION: u8 = 4;
const COVER_CACHE_VERSION: u8 = 1;
const DEFAULT_CACHE_MAX_MB: u64 = 5 * 1024;
const ACCESS_FLUSH_INTERVAL: Duration = Duration::from_secs(30);

pub fn page_image_id(image_path: &str) -> String {
    format!("{:x}", md5::compute(image_path.as_bytes()))
}

pub fn reader_page_cache_key(chapter_id: &str, image_id: &str) -> String {
    format!("{chapter_id}:{image_id}-v{READER_CACHE_VERSION}")
}

pub fn cover_cache_key(comic_id: &str) -> String {
    format!("covers:{comic_id}-v{COVER_CACHE_VERSION}")
}

#[derive(Clone, Copy)]
pub struct CacheConfig {
    max_size_bytes: i64,
}

impl CacheConfig {
    pub fn from_env() -> Result<Self> {
        let max_size_mb = match std::env::var("JM_BOOM_CACHE_MAX_MB") {
            Ok(value) => value
                .parse::<u64>()
                .context("JM_BOOM_CACHE_MAX_MB must be a positive integer")?,
            Err(std::env::VarError::NotPresent) => DEFAULT_CACHE_MAX_MB,
            Err(error) => return Err(error.into()),
        };

        anyhow::ensure!(
            max_size_mb > 0,
            "JM_BOOM_CACHE_MAX_MB must be greater than zero"
        );
        let max_size_bytes = max_size_mb
            .checked_mul(1024 * 1024)
            .context("JM_BOOM_CACHE_MAX_MB is too large")?;
        let max_size_bytes = i64::try_from(max_size_bytes)
            .context("JM_BOOM_CACHE_MAX_MB exceeds the supported range")?;

        Ok(Self { max_size_bytes })
    }
}

pub struct ImageCache {
    storage: CacheStorage,
    repository: CacheRepository,
    max_size_bytes: i64,
    current_size_bytes: AtomicI64,
    operation_lock: RwLock<()>,
    eviction_lock: Mutex<()>,
    pending_accesses: Mutex<HashMap<String, i64>>,
    pending_dimensions: Mutex<HashMap<String, ImageDimensions>>,
}

pub struct CachedReaderPage {
    pub data: Vec<u8>,
    pub format: PageImageFormat,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheStats {
    pub size_bytes: i64,
    pub entry_count: i64,
    pub max_size_bytes: i64,
}

impl ImageCache {
    pub async fn new(db: sqlx::SqlitePool, config: CacheConfig) -> Result<Self> {
        let cache_dir = std::env::current_dir()?.join("data/cache/images");
        Self::new_with_cache_dir(db, config, cache_dir).await
    }

    async fn new_with_cache_dir(
        db: sqlx::SqlitePool,
        config: CacheConfig,
        cache_dir: PathBuf,
    ) -> Result<Self> {
        let cache = Self {
            storage: CacheStorage::new(cache_dir).await?,
            repository: CacheRepository::new(db),
            max_size_bytes: config.max_size_bytes,
            current_size_bytes: AtomicI64::new(0),
            operation_lock: RwLock::new(()),
            eviction_lock: Mutex::new(()),
            pending_accesses: Mutex::new(HashMap::new()),
            pending_dimensions: Mutex::new(HashMap::new()),
        };

        let repair = cache.repair_orphans().await?;
        tracing::info!(
            removed_index_entries = repair.removed_index_entries,
            removed_orphan_files = repair.removed_orphan_files,
            corrected_index_entries = repair.corrected_index_entries,
            "缓存索引与文件修复完成"
        );
        tracing::info!(
            max_size_bytes = cache.max_size_bytes,
            current_size_bytes = cache.current_size_bytes.load(Ordering::Relaxed),
            "图片缓存容量管理已启用"
        );

        Ok(cache)
    }

    #[cfg(test)]
    pub(crate) async fn new_for_test(
        db: sqlx::SqlitePool,
        max_size_bytes: i64,
        cache_dir: PathBuf,
    ) -> Result<Self> {
        Self::new_with_cache_dir(db, CacheConfig { max_size_bytes }, cache_dir).await
    }

    pub fn start_maintenance(self: &Arc<Self>) {
        let cache = Arc::downgrade(self);
        tokio::spawn(async move {
            let mut interval = time::interval(ACCESS_FLUSH_INTERVAL);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            interval.tick().await;

            loop {
                interval.tick().await;
                let Some(cache) = cache.upgrade() else {
                    break;
                };

                match cache.flush_accesses().await {
                    Ok(0) => {}
                    Ok(count) => tracing::debug!(count, "缓存访问时间批量写入完成"),
                    Err(error) => tracing::warn!(%error, "缓存访问时间批量写入失败"),
                }
            }
        });
    }

    pub async fn get_reader_page(&self, key: &str) -> Result<Option<CachedReaderPage>> {
        for format in PageImageFormat::supported() {
            if let Some(data) = self.get_with_extension(key, format.extension()).await? {
                if format.is_complete(&data) {
                    self.remember_reader_dimensions(key, format, &data).await;
                    return Ok(Some(CachedReaderPage { data, format }));
                }
                drop(data);
                if let Some(page) = self.recheck_reader_page(key, format).await? {
                    return Ok(Some(page));
                }
            }
        }
        Ok(None)
    }

    async fn recheck_reader_page(
        &self,
        key: &str,
        format: PageImageFormat,
    ) -> Result<Option<CachedReaderPage>> {
        // A writer may have replaced the bad file since the initial read.
        let _operation = self.operation_lock.write().await;
        let cache_key = cache_index_key(key, format.extension());
        let path = self.storage.path_for(key, format.extension());
        if let Some(data) = self.storage.read(&path).await? {
            if format.is_complete(&data) {
                self.record_access(&cache_key).await;
                self.remember_reader_dimensions(key, format, &data).await;
                return Ok(Some(CachedReaderPage { data, format }));
            }
            tracing::warn!(
                cache_key,
                path = %path.display(),
                bytes = data.len(),
                "清理不完整或格式不匹配的阅读缓存"
            );
        }

        let previous_size = self.repository.entry_size(&cache_key).await?.max(0);
        self.storage.remove_file(&path).await?;
        self.repository.delete(&cache_key).await?;
        self.repository.delete_dimensions(key).await?;
        self.pending_dimensions.lock().await.remove(key);
        self.pending_accesses.lock().await.remove(&cache_key);
        self.adjust_current_size(-previous_size);
        Ok(None)
    }

    pub async fn get_cover(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.get_with_extension(key, "img").await
    }

    async fn get_with_extension(&self, key: &str, extension: &str) -> Result<Option<Vec<u8>>> {
        let started = Instant::now();
        let lock_started = Instant::now();
        let _operation = self.operation_lock.read().await;
        let lock_wait_ms = lock_started.elapsed().as_millis();
        let cache_key = cache_index_key(key, extension);
        let path = self.storage.path_for(key, extension);

        let read_started = Instant::now();
        let data = self.storage.read(&path).await?;
        let read_ms = read_started.elapsed().as_millis();
        match &data {
            Some(data) => {
                self.record_access(&cache_key).await;
                tracing::debug!(
                    cache_key,
                    lock_wait_ms,
                    read_ms,
                    elapsed_ms = started.elapsed().as_millis(),
                    bytes = data.len(),
                    "图片缓存命中"
                );
            }
            None => tracing::debug!(
                cache_key,
                lock_wait_ms,
                elapsed_ms = started.elapsed().as_millis(),
                "图片缓存未命中"
            ),
        }
        Ok(data)
    }

    pub async fn put_reader_page(
        &self,
        key: &str,
        format: PageImageFormat,
        data: &[u8],
    ) -> Result<()> {
        anyhow::ensure!(
            format.is_complete(data),
            "拒绝缓存不完整或格式不匹配的阅读图片"
        );
        self.put_with_extension(key, format.extension(), data)
            .await?;
        if let Some(dimensions) = format.dimensions(data) {
            self.repository.upsert_dimensions(key, dimensions).await?;
            self.pending_dimensions.lock().await.remove(key);
        }
        Ok(())
    }

    pub async fn remember_reader_dimensions(
        &self,
        key: &str,
        format: PageImageFormat,
        data: &[u8],
    ) {
        if let Some(dimensions) = format.dimensions(data) {
            self.pending_dimensions
                .lock()
                .await
                .insert(key.to_string(), dimensions);
        }
    }

    pub async fn reader_dimensions(
        &self,
        chapter_id: &str,
    ) -> Result<HashMap<String, ImageDimensions>> {
        let prefix = format!("{chapter_id}:");
        let mut dimensions = self.repository.reader_dimensions(&prefix).await?;
        for (key, size) in self.pending_dimensions.lock().await.iter() {
            if key.starts_with(&prefix) {
                dimensions.entry(key.clone()).or_insert(*size);
            }
        }
        Ok(dimensions)
    }

    pub async fn put_cover(&self, key: &str, data: &[u8]) -> Result<()> {
        self.put_with_extension(key, "img", data).await
    }

    async fn put_with_extension(&self, key: &str, extension: &str, data: &[u8]) -> Result<()> {
        let started = Instant::now();
        let lock_started = Instant::now();
        let operation = self.operation_lock.read().await;
        let lock_wait_ms = lock_started.elapsed().as_millis();
        let cache_key = cache_index_key(key, extension);
        let path = self.storage.path_for(key, extension);
        let previous_size = self.repository.entry_size(&cache_key).await?.max(0);

        let file_write_started = Instant::now();
        self.storage.write(&path, data).await?;
        let file_write_ms = file_write_started.elapsed().as_millis();

        let now = chrono::Utc::now().timestamp();
        let size = i64::try_from(data.len()).context("cache entry is too large")?;
        let index_write_started = Instant::now();
        self.repository.upsert(&cache_key, &path, size, now).await?;
        self.pending_accesses.lock().await.remove(&cache_key);
        let current_size = self.adjust_current_size(size.saturating_sub(previous_size));
        drop(operation);
        let evicted_entries = if current_size > self.max_size_bytes {
            self.evict_if_needed().await?
        } else {
            0
        };
        tracing::debug!(
            cache_key,
            lock_wait_ms,
            file_write_ms,
            index_write_ms = index_write_started.elapsed().as_millis(),
            elapsed_ms = started.elapsed().as_millis(),
            bytes = data.len(),
            evicted_entries,
            "图片缓存写入完成"
        );

        Ok(())
    }

    pub async fn stats(&self) -> Result<CacheStats> {
        let _operation = self.operation_lock.read().await;
        let (size_bytes, entry_count) = self.repository.stats().await?;
        Ok(CacheStats {
            size_bytes,
            entry_count,
            max_size_bytes: self.max_size_bytes,
        })
    }

    pub async fn clear(&self) -> Result<()> {
        let _operation = self.operation_lock.write().await;
        self.pending_accesses.lock().await.clear();
        self.pending_dimensions.lock().await.clear();
        self.storage.reset().await?;
        self.repository.clear().await?;
        self.current_size_bytes.store(0, Ordering::Relaxed);
        Ok(())
    }

    async fn record_access(&self, key: &str) {
        self.pending_accesses
            .lock()
            .await
            .insert(key.to_string(), chrono::Utc::now().timestamp());
    }

    async fn flush_accesses(&self) -> Result<usize> {
        let _operation = self.operation_lock.read().await;
        self.flush_accesses_inner().await
    }

    async fn flush_accesses_inner(&self) -> Result<usize> {
        let dimensions = std::mem::take(&mut *self.pending_dimensions.lock().await);
        if let Err(error) = self.repository.upsert_dimensions_batch(&dimensions).await {
            let mut pending = self.pending_dimensions.lock().await;
            for (key, size) in dimensions {
                pending.entry(key).or_insert(size);
            }
            return Err(error);
        }
        let accesses = {
            let mut pending = self.pending_accesses.lock().await;
            std::mem::take(&mut *pending)
        };
        if accesses.is_empty() {
            return Ok(0);
        }

        if let Err(error) = self.repository.update_accesses(&accesses).await {
            let mut pending = self.pending_accesses.lock().await;
            for (key, accessed_at) in accesses {
                pending
                    .entry(key)
                    .and_modify(|current| *current = (*current).max(accessed_at))
                    .or_insert(accessed_at);
            }
            return Err(error);
        }

        Ok(accesses.len())
    }

    fn adjust_current_size(&self, delta: i64) -> i64 {
        let previous = self
            .current_size_bytes
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                Some(current.saturating_add(delta).max(0))
            })
            .expect("cache size update always succeeds");
        previous.saturating_add(delta).max(0)
    }
}

fn cache_index_key(key: &str, extension: &str) -> String {
    format!("{key}:{extension}")
}
