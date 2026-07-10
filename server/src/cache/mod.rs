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
        let path = self.get_path(key);

        if path.exists() {
            let data = fs::read(&path).await?;
            // 更新访问时间
            self.touch(key).await?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    pub async fn put(&self, key: &str, data: &[u8]) -> Result<()> {
        let path = self.get_path(key);

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
        .bind(key)
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

    fn get_path(&self, key: &str) -> PathBuf {
        // key 格式: "chapter_id:page"
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() == 2 {
            self.cache_dir.join(parts[0]).join(format!("{}.webp", parts[1]))
        } else {
            self.cache_dir.join(format!("{}.webp", key))
        }
    }
}
