use crate::reader::PageImageFormat;
use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{
    fs,
    sync::{Mutex, RwLock},
    time::{self, MissedTickBehavior},
};

pub const READER_CACHE_VERSION: u8 = 3;
const COVER_CACHE_VERSION: u8 = 1;
const DEFAULT_CACHE_MAX_MB: u64 = 5 * 1024;
const ACCESS_FLUSH_INTERVAL: Duration = Duration::from_secs(30);

pub fn reader_page_cache_key(chapter_id: &str, page: usize) -> String {
    format!("{chapter_id}:{page}-v{READER_CACHE_VERSION}")
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
    cache_dir: PathBuf,
    db: sqlx::SqlitePool,
    max_size_bytes: i64,
    current_size_bytes: AtomicI64,
    operation_lock: RwLock<()>,
    pending_accesses: Mutex<HashMap<String, i64>>,
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

#[derive(Debug, Default)]
struct RepairSummary {
    removed_index_entries: usize,
    removed_orphan_files: usize,
    corrected_index_entries: usize,
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
        fs::create_dir_all(&cache_dir).await?;

        let cache = Self {
            cache_dir,
            db,
            max_size_bytes: config.max_size_bytes,
            current_size_bytes: AtomicI64::new(0),
            operation_lock: RwLock::new(()),
            pending_accesses: Mutex::new(HashMap::new()),
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
                return Ok(Some(CachedReaderPage { data, format }));
            }
        }
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
        let path = self.get_path(key, extension);

        let read_started = Instant::now();
        match fs::read(&path).await {
            Ok(data) => {
                let read_ms = read_started.elapsed().as_millis();
                self.record_access(&cache_key).await;
                tracing::debug!(
                    cache_key,
                    lock_wait_ms,
                    read_ms,
                    elapsed_ms = started.elapsed().as_millis(),
                    bytes = data.len(),
                    "图片缓存命中"
                );
                Ok(Some(data))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                tracing::debug!(
                    cache_key,
                    lock_wait_ms,
                    elapsed_ms = started.elapsed().as_millis(),
                    "图片缓存未命中"
                );
                Ok(None)
            }
            Err(error) => Err(error.into()),
        }
    }

    pub async fn put_reader_page(
        &self,
        key: &str,
        format: PageImageFormat,
        data: &[u8],
    ) -> Result<()> {
        self.put_with_extension(key, format.extension(), data).await
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
        let path = self.get_path(key, extension);
        let previous_size =
            sqlx::query_scalar::<_, i64>("SELECT size FROM cache_index WHERE key = ?")
                .bind(&cache_key)
                .fetch_optional(&self.db)
                .await?
                .unwrap_or(0)
                .max(0);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let file_write_started = Instant::now();
        fs::write(&path, data).await?;
        let file_write_ms = file_write_started.elapsed().as_millis();

        let now = chrono::Utc::now().timestamp();
        let path_str = path.to_string_lossy();
        let size = i64::try_from(data.len()).context("cache entry is too large")?;
        let index_write_started = Instant::now();
        sqlx::query(
            "INSERT INTO cache_index (key, path, size, created_at, accessed_at) VALUES (?, ?, ?, ?, ?) \
             ON CONFLICT(key) DO UPDATE SET path = excluded.path, size = excluded.size, \
             accessed_at = excluded.accessed_at",
        )
        .bind(&cache_key)
        .bind(path_str.as_ref())
        .bind(size)
        .bind(now)
        .bind(now)
        .execute(&self.db)
        .await?;
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
        let (size_bytes, entry_count) = sqlx::query_as::<_, (i64, i64)>(
            "SELECT COALESCE(SUM(size), 0), COUNT(*) FROM cache_index",
        )
        .fetch_one(&self.db)
        .await?;

        Ok(CacheStats {
            size_bytes,
            entry_count,
            max_size_bytes: self.max_size_bytes,
        })
    }

    pub async fn clear(&self) -> Result<()> {
        let _operation = self.operation_lock.write().await;
        self.pending_accesses.lock().await.clear();

        match fs::remove_dir_all(&self.cache_dir).await {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        fs::create_dir_all(&self.cache_dir).await?;
        sqlx::query("DELETE FROM cache_index")
            .execute(&self.db)
            .await?;
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
        let accesses = {
            let mut pending = self.pending_accesses.lock().await;
            std::mem::take(&mut *pending)
        };
        if accesses.is_empty() {
            return Ok(0);
        }

        let result = async {
            let mut transaction = self.db.begin().await?;
            for (key, accessed_at) in &accesses {
                sqlx::query(
                    "UPDATE cache_index SET accessed_at = MAX(accessed_at, ?) WHERE key = ?",
                )
                .bind(accessed_at)
                .bind(key)
                .execute(&mut *transaction)
                .await?;
            }
            transaction.commit().await?;
            Ok::<(), anyhow::Error>(())
        }
        .await;

        if let Err(error) = result {
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

    async fn repair_orphans(&self) -> Result<RepairSummary> {
        let _operation = self.operation_lock.write().await;
        let rows =
            sqlx::query_as::<_, (String, String, i64)>("SELECT key, path, size FROM cache_index")
                .fetch_all(&self.db)
                .await?;
        let mut indexed_paths = HashSet::new();
        let mut removed_keys = Vec::new();
        let mut corrected_entries = Vec::new();

        for (key, stored_path, stored_size) in rows {
            let path = self.normalize_stored_path(&stored_path);
            if !path.starts_with(&self.cache_dir) {
                removed_keys.push(key);
                continue;
            }

            match fs::metadata(&path).await {
                Ok(metadata) if metadata.is_file() => {
                    let actual_size = i64::try_from(metadata.len()).unwrap_or(i64::MAX);
                    let normalized_path = path.to_string_lossy().into_owned();
                    if actual_size != stored_size || normalized_path != stored_path {
                        corrected_entries.push((key, normalized_path, actual_size));
                    }
                    indexed_paths.insert(path);
                }
                Ok(_) => removed_keys.push(key),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    removed_keys.push(key)
                }
                Err(error) => return Err(error.into()),
            }
        }

        let cache_files = scan_cache_files(self.cache_dir.clone()).await?;
        let orphan_files = cache_files
            .into_iter()
            .filter(|path| !indexed_paths.contains(path))
            .collect::<Vec<_>>();

        for path in &orphan_files {
            match fs::remove_file(path).await {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }

        let mut transaction = self.db.begin().await?;
        for key in &removed_keys {
            sqlx::query("DELETE FROM cache_index WHERE key = ?")
                .bind(key)
                .execute(&mut *transaction)
                .await?;
        }
        for (key, path, size) in &corrected_entries {
            sqlx::query("UPDATE cache_index SET path = ?, size = ? WHERE key = ?")
                .bind(path)
                .bind(size)
                .bind(key)
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        let size_bytes =
            sqlx::query_scalar::<_, i64>("SELECT COALESCE(SUM(size), 0) FROM cache_index")
                .fetch_one(&self.db)
                .await?;
        self.current_size_bytes
            .store(size_bytes.max(0), Ordering::Relaxed);

        Ok(RepairSummary {
            removed_index_entries: removed_keys.len(),
            removed_orphan_files: orphan_files.len(),
            corrected_index_entries: corrected_entries.len(),
        })
    }

    async fn evict_if_needed(&self) -> Result<usize> {
        let _operation = self.operation_lock.write().await;
        if self.current_size_bytes.load(Ordering::Relaxed) <= self.max_size_bytes {
            return Ok(0);
        }
        self.evict_if_needed_locked().await
    }

    async fn evict_if_needed_locked(&self) -> Result<usize> {
        let mut size_bytes =
            sqlx::query_scalar::<_, i64>("SELECT COALESCE(SUM(size), 0) FROM cache_index")
                .fetch_one(&self.db)
                .await?;
        size_bytes = size_bytes.max(0);
        self.current_size_bytes.store(size_bytes, Ordering::Relaxed);
        if size_bytes <= self.max_size_bytes {
            return Ok(0);
        }

        if let Err(error) = self.flush_accesses_inner().await {
            tracing::warn!(%error, "LRU 淘汰前刷新访问时间失败");
        }

        let candidates = sqlx::query_as::<_, (String, String, i64)>(
            "SELECT key, path, size FROM cache_index ORDER BY accessed_at ASC, created_at ASC",
        )
        .fetch_all(&self.db)
        .await?;
        let initial_size = size_bytes;
        let target_size_bytes = self.max_size_bytes / 2;
        let mut evicted_entries = 0;

        for (key, stored_path, stored_size) in candidates {
            if size_bytes <= target_size_bytes {
                break;
            }

            let path = self.normalize_stored_path(&stored_path);
            if path.starts_with(&self.cache_dir) {
                match fs::remove_file(&path).await {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
            }

            sqlx::query("DELETE FROM cache_index WHERE key = ?")
                .bind(&key)
                .execute(&self.db)
                .await?;
            self.pending_accesses.lock().await.remove(&key);
            size_bytes = size_bytes.saturating_sub(stored_size.max(0));
            evicted_entries += 1;
        }
        self.current_size_bytes.store(size_bytes, Ordering::Relaxed);

        tracing::info!(
            initial_size_bytes = initial_size,
            remaining_size_bytes = size_bytes,
            max_size_bytes = self.max_size_bytes,
            target_size_bytes,
            evicted_entries,
            "图片缓存 LRU 淘汰完成"
        );
        Ok(evicted_entries)
    }

    fn get_path(&self, key: &str, extension: &str) -> PathBuf {
        if let Some((chapter_id, page)) = key.split_once(':') {
            self.cache_dir
                .join(chapter_id)
                .join(format!("{page}.{extension}"))
        } else {
            self.cache_dir.join(format!("{key}.{extension}"))
        }
    }

    fn normalize_stored_path(&self, path: &str) -> PathBuf {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            path
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(path)
        }
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

async fn scan_cache_files(cache_dir: PathBuf) -> Result<Vec<PathBuf>> {
    tokio::task::spawn_blocking(move || scan_cache_files_sync(&cache_dir))
        .await
        .context("cache directory scan task failed")?
}

fn scan_cache_files_sync(cache_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut directories = vec![cache_dir.to_path_buf()];

    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                directories.push(entry.path());
            } else if file_type.is_file() {
                files.push(entry.path());
            }
        }
    }

    Ok(files)
}

fn cache_index_key(key: &str, extension: &str) -> String {
    if extension == "webp" {
        key.to_string()
    } else {
        format!("{key}:{extension}")
    }
}

#[cfg(test)]
mod tests {
    use super::{CacheConfig, ImageCache};
    use sqlx::sqlite::SqlitePoolOptions;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };
    use tokio::fs;

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    #[tokio::test]
    async fn tracks_replacement_size_delta_and_resets_after_clear() {
        let (cache, cache_dir) = test_cache("replace", 100).await;

        cache
            .put_cover("cover", &[1; 20])
            .await
            .expect("write initial cache entry");
        assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 20);

        cache
            .put_cover("cover", &[2; 8])
            .await
            .expect("replace cache entry");
        assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 8);
        let stats = cache.stats().await.expect("load cache stats");
        assert_eq!(stats.size_bytes, 8);
        assert_eq!(stats.entry_count, 1);

        cache.clear().await.expect("clear cache");
        assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 0);
        fs::remove_dir_all(cache_dir)
            .await
            .expect("remove test cache");
    }

    #[tokio::test]
    async fn evicts_to_half_capacity_only_after_limit_is_exceeded() {
        let (cache, cache_dir) = test_cache("evict", 10).await;

        cache.put_cover("one", &[1; 4]).await.expect("write one");
        cache.put_cover("two", &[2; 4]).await.expect("write two");
        let before = cache.stats().await.expect("load pre-eviction stats");
        assert_eq!(before.size_bytes, 8);
        assert_eq!(before.entry_count, 2);

        cache
            .put_cover("three", &[3; 4])
            .await
            .expect("write over capacity");
        let after = cache.stats().await.expect("load post-eviction stats");
        assert_eq!(after.size_bytes, 4);
        assert_eq!(after.entry_count, 1);
        assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 4);

        fs::remove_dir_all(cache_dir)
            .await
            .expect("remove test cache");
    }

    #[tokio::test]
    async fn calibrates_size_from_repaired_index_on_startup() {
        let db = test_db().await;
        let cache_dir = test_cache_dir("calibrate");
        fs::create_dir_all(&cache_dir)
            .await
            .expect("create test cache directory");
        let path = cache_dir.join("seed.img");
        fs::write(&path, [1; 7])
            .await
            .expect("write seeded cache file");
        sqlx::query(
            "INSERT INTO cache_index (key, path, size, created_at, accessed_at) VALUES (?, ?, ?, 1, 1)",
        )
        .bind("seed:img")
        .bind(path.to_string_lossy().as_ref())
        .bind(99_i64)
        .execute(&db)
        .await
        .expect("seed cache index");

        let cache = ImageCache::new_with_cache_dir(
            db,
            CacheConfig {
                max_size_bytes: 100,
            },
            cache_dir.clone(),
        )
        .await
        .expect("create calibrated cache");

        assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 7);
        assert_eq!(cache.stats().await.expect("load cache stats").size_bytes, 7);
        fs::remove_dir_all(cache_dir)
            .await
            .expect("remove test cache");
    }

    async fn test_cache(name: &str, max_size_bytes: i64) -> (ImageCache, PathBuf) {
        let cache_dir = test_cache_dir(name);
        let cache = ImageCache::new_with_cache_dir(
            test_db().await,
            CacheConfig { max_size_bytes },
            cache_dir.clone(),
        )
        .await
        .expect("create test cache");
        (cache, cache_dir)
    }

    async fn test_db() -> sqlx::SqlitePool {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect in-memory sqlite");
        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .expect("run migrations");
        db
    }

    fn test_cache_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "jm-boom-image-cache-{name}-{}-{}",
            std::process::id(),
            TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }
}
