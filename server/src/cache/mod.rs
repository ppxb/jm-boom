use crate::reader::PageImageFormat;
use anyhow::Result;
use serde::Serialize;
use std::{path::PathBuf, time::Instant};
use tokio::fs;
use tokio::sync::RwLock;

pub const READER_CACHE_VERSION: u8 = 3;
const COVER_CACHE_VERSION: u8 = 1;

pub fn reader_page_cache_key(chapter_id: &str, page: usize) -> String {
    format!("{chapter_id}:{page}-v{READER_CACHE_VERSION}")
}

pub fn cover_cache_key(comic_id: &str) -> String {
    format!("covers:{comic_id}-v{COVER_CACHE_VERSION}")
}

pub struct ImageCache {
    cache_dir: PathBuf,
    db: sqlx::SqlitePool,
    operation_lock: RwLock<()>,
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
}

impl ImageCache {
    pub async fn new(db: sqlx::SqlitePool) -> Result<Self> {
        let cache_dir = PathBuf::from("data/cache/images");
        fs::create_dir_all(&cache_dir).await?;

        Ok(Self {
            cache_dir,
            db,
            operation_lock: RwLock::new(()),
        })
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

        if path.exists() {
            let read_started = Instant::now();
            let data = fs::read(&path).await?;
            let read_ms = read_started.elapsed().as_millis();
            // 更新访问时间
            let touch_started = Instant::now();
            self.touch(&cache_key).await?;
            tracing::debug!(
                cache_key,
                lock_wait_ms,
                read_ms,
                touch_ms = touch_started.elapsed().as_millis(),
                elapsed_ms = started.elapsed().as_millis(),
                bytes = data.len(),
                "图片缓存命中"
            );
            Ok(Some(data))
        } else {
            tracing::debug!(
                cache_key,
                lock_wait_ms,
                elapsed_ms = started.elapsed().as_millis(),
                "图片缓存未命中"
            );
            Ok(None)
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
        let _operation = self.operation_lock.read().await;
        let lock_wait_ms = lock_started.elapsed().as_millis();
        let cache_key = cache_index_key(key, extension);
        let path = self.get_path(key, extension);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let file_write_started = Instant::now();
        fs::write(&path, data).await?;
        let file_write_ms = file_write_started.elapsed().as_millis();

        // 记录到数据库
        let now = chrono::Utc::now().timestamp();
        let path_str = path.to_str().unwrap_or("");
        let size = data.len() as i64;
        let index_write_started = Instant::now();
        sqlx::query(
            "INSERT OR REPLACE INTO cache_index (key, path, size, created_at, accessed_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&cache_key)
        .bind(path_str)
        .bind(size)
        .bind(now)
        .bind(now)
        .execute(&self.db)
        .await?;
        tracing::debug!(
            cache_key,
            lock_wait_ms,
            file_write_ms,
            index_write_ms = index_write_started.elapsed().as_millis(),
            elapsed_ms = started.elapsed().as_millis(),
            bytes = data.len(),
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
        })
    }

    pub async fn clear(&self) -> Result<()> {
        let _operation = self.operation_lock.write().await;

        match fs::remove_dir_all(&self.cache_dir).await {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        fs::create_dir_all(&self.cache_dir).await?;
        sqlx::query("DELETE FROM cache_index")
            .execute(&self.db)
            .await?;

        Ok(())
    }

    async fn touch(&self, key: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        sqlx::query("UPDATE cache_index SET accessed_at = ? WHERE key = ?")
            .bind(now)
            .bind(key)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    fn get_path(&self, key: &str, extension: &str) -> PathBuf {
        // key 格式: "chapter_id:page"
        if let Some((chapter_id, page)) = key.split_once(':') {
            self.cache_dir
                .join(chapter_id)
                .join(format!("{page}.{extension}"))
        } else {
            self.cache_dir.join(format!("{key}.{extension}"))
        }
    }
}

fn cache_index_key(key: &str, extension: &str) -> String {
    if extension == "webp" {
        key.to_string()
    } else {
        format!("{key}:{extension}")
    }
}
