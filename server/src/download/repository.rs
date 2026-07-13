use super::{
    model::{DownloadChapter, DownloadStatus, DownloadTask, OfflineChapterManifest},
    DownloadResult,
};
use sqlx::SqlitePool;

#[derive(Clone)]
pub(super) struct DownloadRepository {
    db: SqlitePool,
}

impl DownloadRepository {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn load_tasks(&self) -> anyhow::Result<Vec<DownloadTask>> {
        let rows =
            sqlx::query_as::<_, (String, String)>("SELECT task_id, payload FROM download_tasks")
                .fetch_all(&self.db)
                .await?;
        let mut tasks = Vec::with_capacity(rows.len());

        for (task_id, payload) in rows {
            match serde_json::from_str::<DownloadTask>(&payload) {
                Ok(mut task) => {
                    task.task_id = task_id;
                    if task.status == DownloadStatus::Running {
                        task.status = DownloadStatus::Queued;
                        task.total_pages = 0;
                        task.completed_pages = 0;
                        task.eta_seconds = None;
                        task.speed_bytes_per_second = 0;
                        task.completed_at = None;
                    }
                    tasks.push(task);
                }
                Err(error) => tracing::warn!(task_id, %error, "忽略无法解析的下载任务"),
            }
        }
        Ok(tasks)
    }

    pub async fn persist_task(&self, task: &DownloadTask) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO download_tasks (task_id, payload, updated_at) VALUES (?, ?, ?) \
             ON CONFLICT(task_id) DO UPDATE SET payload = excluded.payload, updated_at = excluded.updated_at \
             WHERE excluded.updated_at >= download_tasks.updated_at",
        )
        .bind(&task.task_id)
        .bind(serde_json::to_string(task)?)
        .bind(task.updated_at)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn delete_task(&self, task_id: &str) -> anyhow::Result<()> {
        let mut transaction = self.db.begin().await?;
        sqlx::query("DELETE FROM download_manifests WHERE task_id = ?")
            .bind(task_id)
            .execute(&mut *transaction)
            .await?;
        sqlx::query("DELETE FROM download_tasks WHERE task_id = ?")
            .bind(task_id)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn offline_manifest(
        &self,
        chapter_id: &str,
    ) -> anyhow::Result<Option<OfflineChapterManifest>> {
        let payload = sqlx::query_scalar::<_, String>(
            "SELECT payload FROM download_manifests WHERE chapter_id = ? AND completed = 1 \
             ORDER BY updated_at DESC LIMIT 1",
        )
        .bind(chapter_id)
        .fetch_optional(&self.db)
        .await?;

        payload
            .map(|payload| serde_json::from_str(&payload).map_err(Into::into))
            .transpose()
    }

    pub async fn downloaded_chapters(&self) -> anyhow::Result<Vec<String>> {
        Ok(sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT chapter_id FROM download_manifests \
             WHERE completed = 1 ORDER BY chapter_id",
        )
        .fetch_all(&self.db)
        .await?)
    }

    pub async fn completed_manifest_task_ids(
        &self,
        chapter_id: &str,
    ) -> anyhow::Result<Vec<String>> {
        Ok(sqlx::query_scalar::<_, String>(
            "SELECT task_id FROM download_manifests WHERE chapter_id = ? AND completed = 1 \
             ORDER BY updated_at DESC",
        )
        .bind(chapter_id)
        .fetch_all(&self.db)
        .await?)
    }

    pub async fn persist_manifest(
        &self,
        task: &DownloadTask,
        chapter: &DownloadChapter,
        images: &[String],
    ) -> DownloadResult<()> {
        let manifest = OfflineChapterManifest {
            task_id: task.task_id.clone(),
            album_id: task.album_id.clone(),
            chapter_id: chapter.chapter_id.clone(),
            title: chapter.title.clone(),
            images: images.to_vec(),
            updated_at: chrono::Utc::now().timestamp_millis(),
        };
        sqlx::query(
            "INSERT INTO download_manifests \
             (task_id, album_id, chapter_id, title, payload, updated_at, completed) \
             VALUES (?, ?, ?, ?, ?, ?, 0) \
             ON CONFLICT(task_id, chapter_id) DO UPDATE SET \
             album_id = excluded.album_id, title = excluded.title, \
             payload = excluded.payload, updated_at = excluded.updated_at, completed = 0",
        )
        .bind(&manifest.task_id)
        .bind(&manifest.album_id)
        .bind(&manifest.chapter_id)
        .bind(&manifest.title)
        .bind(serde_json::to_string(&manifest).map_err(anyhow::Error::from)?)
        .bind(manifest.updated_at)
        .execute(&self.db)
        .await
        .map_err(anyhow::Error::from)?;
        Ok(())
    }

    pub async fn complete_manifest(&self, task_id: &str, chapter_id: &str) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE download_manifests SET completed = 1, updated_at = ? \
             WHERE task_id = ? AND chapter_id = ?",
        )
        .bind(chrono::Utc::now().timestamp_millis())
        .bind(task_id)
        .bind(chapter_id)
        .execute(&self.db)
        .await?;
        Ok(())
    }
}
