use anyhow::Result;
use std::{collections::HashMap, path::Path};

pub(super) struct CacheRepository {
    db: sqlx::SqlitePool,
}

pub(super) struct CacheEntry {
    pub key: String,
    pub path: String,
    pub size: i64,
}

pub(super) struct CacheCorrection {
    pub key: String,
    pub path: String,
    pub size: i64,
}

impl CacheRepository {
    pub(super) fn new(db: sqlx::SqlitePool) -> Self {
        Self { db }
    }

    pub(super) async fn entry_size(&self, key: &str) -> Result<i64> {
        Ok(
            sqlx::query_scalar::<_, i64>("SELECT size FROM cache_index WHERE key = ?")
                .bind(key)
                .fetch_optional(&self.db)
                .await?
                .unwrap_or(0),
        )
    }

    pub(super) async fn upsert(&self, key: &str, path: &Path, size: i64, now: i64) -> Result<()> {
        sqlx::query(
            "INSERT INTO cache_index (key, path, size, created_at, accessed_at) VALUES (?, ?, ?, ?, ?) \
             ON CONFLICT(key) DO UPDATE SET path = excluded.path, size = excluded.size, \
             accessed_at = excluded.accessed_at",
        )
        .bind(key)
        .bind(path.to_string_lossy().as_ref())
        .bind(size)
        .bind(now)
        .bind(now)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub(super) async fn stats(&self) -> Result<(i64, i64)> {
        Ok(sqlx::query_as::<_, (i64, i64)>(
            "SELECT COALESCE(SUM(size), 0), COUNT(*) FROM cache_index",
        )
        .fetch_one(&self.db)
        .await?)
    }

    pub(super) async fn clear(&self) -> Result<()> {
        sqlx::query("DELETE FROM cache_index")
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub(super) async fn update_accesses(&self, accesses: &HashMap<String, i64>) -> Result<()> {
        let mut transaction = self.db.begin().await?;
        for (key, accessed_at) in accesses {
            sqlx::query("UPDATE cache_index SET accessed_at = MAX(accessed_at, ?) WHERE key = ?")
                .bind(accessed_at)
                .bind(key)
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub(super) async fn entries(&self) -> Result<Vec<CacheEntry>> {
        Ok(
            sqlx::query_as::<_, (String, String, i64)>("SELECT key, path, size FROM cache_index")
                .fetch_all(&self.db)
                .await?
                .into_iter()
                .map(|(key, path, size)| CacheEntry { key, path, size })
                .collect(),
        )
    }

    pub(super) async fn apply_repairs(
        &self,
        removed_keys: &[String],
        corrections: &[CacheCorrection],
    ) -> Result<()> {
        let mut transaction = self.db.begin().await?;
        for key in removed_keys {
            sqlx::query("DELETE FROM cache_index WHERE key = ?")
                .bind(key)
                .execute(&mut *transaction)
                .await?;
        }
        for correction in corrections {
            sqlx::query("UPDATE cache_index SET path = ?, size = ? WHERE key = ?")
                .bind(&correction.path)
                .bind(correction.size)
                .bind(&correction.key)
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub(super) async fn total_size(&self) -> Result<i64> {
        Ok(
            sqlx::query_scalar::<_, i64>("SELECT COALESCE(SUM(size), 0) FROM cache_index")
                .fetch_one(&self.db)
                .await?,
        )
    }

    pub(super) async fn eviction_candidates(&self) -> Result<Vec<CacheEntry>> {
        Ok(sqlx::query_as::<_, (String, String, i64)>(
            "SELECT key, path, size FROM cache_index ORDER BY accessed_at ASC, created_at ASC",
        )
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(|(key, path, size)| CacheEntry { key, path, size })
        .collect())
    }

    pub(super) async fn delete(&self, key: &str) -> Result<()> {
        sqlx::query("DELETE FROM cache_index WHERE key = ?")
            .bind(key)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}
