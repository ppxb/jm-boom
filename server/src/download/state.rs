use super::{
    manager::DownloadManager,
    model::{mark_task_updated, DownloadChapter, DownloadStatus, DownloadTask},
    DownloadError, DownloadResult,
};
use std::time::{Duration, Instant};
use tokio::sync::watch;

const PROGRESS_PERSIST_INTERVAL: Duration = Duration::from_secs(1);

impl DownloadManager {
    pub(super) async fn is_generation_active(
        &self,
        task_id: &str,
        generation: u64,
        cancelled: &watch::Receiver<bool>,
    ) -> bool {
        if *cancelled.borrow() {
            return false;
        }
        self.tasks.read().await.get(task_id).is_some_and(|task| {
            task.generation == generation
                && matches!(
                    task.status,
                    DownloadStatus::Queued | DownloadStatus::Running
                )
        })
    }

    pub(super) async fn begin_generation(
        &self,
        task_id: &str,
        generation: u64,
    ) -> DownloadResult<Option<(Vec<DownloadChapter>, DownloadTask)>> {
        let snapshot = {
            let mut tasks = self.tasks.write().await;
            let task = tasks.get_mut(task_id).ok_or(DownloadError::NotFound)?;
            if task.status != DownloadStatus::Queued || task.generation != generation {
                return Ok(None);
            }
            task.status = DownloadStatus::Running;
            task.started_at
                .get_or_insert_with(|| chrono::Utc::now().timestamp_millis());
            mark_task_updated(task);
            (task.chapters.clone(), task.clone())
        };
        self.repository.persist_task(&snapshot.1).await?;
        Ok(Some(snapshot))
    }

    pub(super) async fn update_chapter(
        &self,
        task_id: &str,
        generation: u64,
        title: &str,
    ) -> DownloadResult<()> {
        self.update_running_task(task_id, generation, |task| {
            task.current_chapter_title = title.to_string();
        })
        .await
    }

    pub(super) async fn set_total_pages(
        &self,
        task_id: &str,
        generation: u64,
        total: u32,
    ) -> DownloadResult<()> {
        self.update_running_task(task_id, generation, |task| {
            task.total_pages = total;
        })
        .await
    }

    pub(super) async fn complete_page(
        &self,
        task_id: &str,
        generation: u64,
        bytes: u64,
        elapsed: Duration,
    ) -> DownloadResult<()> {
        let snapshot = {
            let mut tasks = self.tasks.write().await;
            let task = tasks.get_mut(task_id).ok_or(DownloadError::NotFound)?;
            if task.generation != generation || task.status != DownloadStatus::Running {
                return Ok(());
            }
            task.completed_pages = task.completed_pages.saturating_add(1);
            let seconds = elapsed.as_secs_f64().max(0.001);
            task.speed_bytes_per_second = (bytes as f64 / seconds) as u64;
            let remaining = task.total_pages.saturating_sub(task.completed_pages) as f64;
            let pages_per_second = task.completed_pages as f64 / seconds;
            task.eta_seconds =
                (pages_per_second > 0.0).then_some((remaining / pages_per_second) as u64);
            mark_task_updated(task);
            task.clone()
        };

        if self.should_persist_progress(task_id, &snapshot).await {
            self.repository.persist_task(&snapshot).await?;
        }
        Ok(())
    }

    pub(super) async fn complete_generation(
        &self,
        task_id: &str,
        generation: u64,
    ) -> DownloadResult<()> {
        let snapshot = {
            let mut tasks = self.tasks.write().await;
            let task = tasks.get_mut(task_id).ok_or(DownloadError::NotFound)?;
            if task.generation != generation || task.status != DownloadStatus::Running {
                return Ok(());
            }
            task.status = DownloadStatus::Completed;
            task.eta_seconds = Some(0);
            task.completed_at = Some(chrono::Utc::now().timestamp_millis());
            mark_task_updated(task);
            task.clone()
        };
        self.progress_persisted_at.lock().await.remove(task_id);
        self.repository.persist_task(&snapshot).await?;
        Ok(())
    }

    pub(super) async fn fail_generation(
        &self,
        task_id: &str,
        generation: u64,
        error: String,
    ) -> DownloadResult<()> {
        let snapshot = {
            let mut tasks = self.tasks.write().await;
            let Some(task) = tasks.get_mut(task_id) else {
                return Ok(());
            };
            if task.generation != generation || task.status != DownloadStatus::Running {
                return Ok(());
            }
            task.status = DownloadStatus::Failed;
            task.error = Some(error);
            mark_task_updated(task);
            task.clone()
        };
        self.progress_persisted_at.lock().await.remove(task_id);
        self.repository.persist_task(&snapshot).await?;
        Ok(())
    }

    async fn update_running_task(
        &self,
        task_id: &str,
        generation: u64,
        update: impl FnOnce(&mut DownloadTask),
    ) -> DownloadResult<()> {
        let snapshot = {
            let mut tasks = self.tasks.write().await;
            let task = tasks.get_mut(task_id).ok_or(DownloadError::NotFound)?;
            if task.generation != generation || task.status != DownloadStatus::Running {
                return Ok(());
            }
            update(task);
            mark_task_updated(task);
            task.clone()
        };
        self.repository.persist_task(&snapshot).await?;
        Ok(())
    }

    async fn should_persist_progress(&self, task_id: &str, task: &DownloadTask) -> bool {
        if task.completed_pages >= task.total_pages {
            return true;
        }

        let now = Instant::now();
        let mut persisted_at = self.progress_persisted_at.lock().await;
        match persisted_at.get_mut(task_id) {
            Some(previous) if now.duration_since(*previous) < PROGRESS_PERSIST_INTERVAL => false,
            Some(previous) => {
                *previous = now;
                true
            }
            None => {
                persisted_at.insert(task_id.to_string(), now);
                true
            }
        }
    }
}
