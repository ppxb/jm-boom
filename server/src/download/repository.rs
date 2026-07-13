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
             payload = excluded.payload, updated_at = excluded.updated_at, \
             completed = CASE \
                 WHEN download_manifests.completed = 1 \
                  AND json_extract(download_manifests.payload, '$.images') = \
                      json_extract(excluded.payload, '$.images') \
                 THEN 1 ELSE 0 \
             END",
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

#[cfg(test)]
mod tests {
    use super::DownloadRepository;
    use crate::download::model::{DownloadChapter, DownloadStatus, DownloadTask};
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn preserves_completed_manifest_only_when_images_are_unchanged() {
        let repository = test_repository().await;
        let task = test_task();
        let chapter = DownloadChapter {
            chapter_id: "1001".into(),
            title: "第一章".into(),
            order: 1,
        };
        let images = vec!["001.jpg".into(), "002.jpg".into()];

        repository
            .persist_manifest(&task, &chapter, &images)
            .await
            .expect("persist initial manifest");
        repository
            .complete_manifest(&task.task_id, &chapter.chapter_id)
            .await
            .expect("complete initial manifest");

        let renamed_chapter = DownloadChapter {
            title: "第一章（更新标题）".into(),
            ..chapter.clone()
        };
        repository
            .persist_manifest(&task, &renamed_chapter, &images)
            .await
            .expect("persist unchanged manifest");

        let preserved = repository
            .offline_manifest(&chapter.chapter_id)
            .await
            .expect("load preserved manifest")
            .expect("completed manifest should stay available");
        assert_eq!(preserved.title, renamed_chapter.title);
        assert_eq!(preserved.images, images);

        let changed_images = vec!["001.jpg".into(), "002-new.jpg".into()];
        repository
            .persist_manifest(&task, &renamed_chapter, &changed_images)
            .await
            .expect("persist changed manifest");

        assert!(repository
            .offline_manifest(&chapter.chapter_id)
            .await
            .expect("load changed manifest")
            .is_none());
    }

    async fn test_repository() -> DownloadRepository {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect in-memory sqlite");
        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .expect("run migrations");
        DownloadRepository::new(db)
    }

    fn test_task() -> DownloadTask {
        DownloadTask {
            task_id: "task-1".into(),
            album_id: "album-1".into(),
            comic_title: "测试漫画".into(),
            chapters: Vec::new(),
            status: DownloadStatus::Running,
            current_chapter_title: String::new(),
            total_pages: 0,
            completed_pages: 0,
            eta_seconds: None,
            speed_bytes_per_second: 0,
            error: None,
            created_at: 1,
            started_at: Some(1),
            updated_at: 1,
            completed_at: None,
            generation: 1,
        }
    }
}
