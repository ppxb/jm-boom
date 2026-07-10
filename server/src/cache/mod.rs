use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;

pub struct ImageCache {
    cache_dir: PathBuf,
    db: sqlx::SqlitePool,
}

impl ImageCache {
    pub async fn new(db: sqlx::SqlitePool) -> Result<Self> {
        let cache_dir = PathBuf::from("data/cache/images");
        fs::create_dir_all(&cache_dir).await?;

        Ok(Self { cache_dir, db })
    }

    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.get_with_extension(key, "webp").await
    }

    pub async fn get_gif(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.get_with_extension(key, "gif").await
    }

    async fn get_with_extension(&self, key: &str, extension: &str) -> Result<Option<Vec<u8>>> {
        let cache_key = cache_index_key(key, extension);
        let path = self.get_path(key, extension);

        if path.exists() {
            let data = fs::read(&path).await?;
            // 更新访问时间
            self.touch(&cache_key).await?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    pub async fn put(&self, key: &str, data: &[u8]) -> Result<()> {
        self.put_with_extension(key, "webp", data).await
    }

    pub async fn put_gif(&self, key: &str, data: &[u8]) -> Result<()> {
        self.put_with_extension(key, "gif", data).await
    }

    async fn put_with_extension(&self, key: &str, extension: &str, data: &[u8]) -> Result<()> {
        let cache_key = cache_index_key(key, extension);
        let path = self.get_path(key, extension);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(&path, data).await?;

        // 记录到数据库
        let now = chrono::Utc::now().timestamp();
        let path_str = path.to_str().unwrap_or("");
        let size = data.len() as i64;
        sqlx::query(
            "INSERT OR REPLACE INTO cache_index (key, path, size, created_at, accessed_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(cache_key)
        .bind(path_str)
        .bind(size)
        .bind(now)
        .bind(now)
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
