use super::{
    error::{DownloadError, DownloadResult},
    model::{
        mark_task_updated, DownloadStatus, DownloadTask, DownloadTaskList, DownloadedChapterList,
        EnqueueDownload, OfflineChapterManifest,
    },
    repository::DownloadRepository,
    storage::DownloadStorage,
    DOWNLOAD_TASK_CONCURRENCY,
};
use crate::{
    cache::CachedReaderPage,
    endpoint::{request_with_failover, EndpointManager},
    jm::{JmClient, JmResult},
    page_materializer::PageMaterializer,
};
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    future::Future,
    path::PathBuf,
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Instant,
};
use tokio::sync::{watch, Mutex, RwLock, Semaphore};

static TASK_SEQUENCE: AtomicU64 = AtomicU64::new(1);
pub(super) struct WorkerHandle {
    pub(super) generation: u64,
    pub(super) cancel: watch::Sender<bool>,
    pub(super) finished: watch::Sender<bool>,
}

#[derive(Clone)]
pub struct DownloadManager {
    pub(super) repository: DownloadRepository,
    pub(super) storage: DownloadStorage,
    pub(super) jm: Arc<JmClient>,
    pub(super) endpoints: Arc<EndpointManager>,
    pub(super) page_materializer: Arc<PageMaterializer>,
    pub(super) tasks: Arc<RwLock<HashMap<String, DownloadTask>>>,
    pub(super) workers: Arc<Mutex<HashMap<String, WorkerHandle>>>,
    pub(super) task_semaphore: Arc<Semaphore>,
    pub(super) progress_persisted_at: Arc<Mutex<HashMap<String, Instant>>>,
}

impl DownloadManager {
    pub async fn new(
        db: SqlitePool,
        jm: Arc<JmClient>,
        endpoints: Arc<EndpointManager>,
        page_materializer: Arc<PageMaterializer>,
    ) -> DownloadResult<Self> {
        let repository = DownloadRepository::new(db);
        let storage = DownloadStorage::new(PathBuf::from("data/downloads")).await?;
        let tasks = repository
            .load_tasks()
            .await?
            .into_iter()
            .map(|task| (task.task_id.clone(), task))
            .collect();

        Ok(Self {
            repository,
            storage,
            jm,
            endpoints,
            page_materializer,
            tasks: Arc::new(RwLock::new(tasks)),
            workers: Arc::new(Mutex::new(HashMap::new())),
            task_semaphore: Arc::new(Semaphore::new(DOWNLOAD_TASK_CONCURRENCY)),
            progress_persisted_at: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn resume_pending(&self) {
        let task_ids = self
            .tasks
            .read()
            .await
            .values()
            .filter(|task| task.status == DownloadStatus::Queued)
            .map(|task| task.task_id.clone())
            .collect::<Vec<_>>();
        for task_id in task_ids {
            self.spawn(task_id);
        }
    }

    pub async fn enqueue(&self, input: EnqueueDownload) -> DownloadResult<DownloadTaskList> {
        validate_enqueue(&input)?;
        let now = chrono::Utc::now().timestamp_millis();
        let task_id = format!("{}-{}", now, TASK_SEQUENCE.fetch_add(1, Ordering::Relaxed));
        let task = DownloadTask {
            task_id: task_id.clone(),
            album_id: input.album_id,
            comic_title: input.comic_title,
            chapters: input.chapters,
            status: DownloadStatus::Queued,
            current_chapter_title: String::new(),
            total_pages: 0,
            completed_pages: 0,
            eta_seconds: None,
            speed_bytes_per_second: 0,
            error: None,
            created_at: now,
            started_at: None,
            updated_at: now,
            completed_at: None,
            generation: 1,
        };

        self.repository.persist_task(&task).await?;
        self.tasks.write().await.insert(task_id.clone(), task);
        self.spawn(task_id);
        Ok(self.list().await)
    }

    pub async fn list(&self) -> DownloadTaskList {
        let mut tasks = self
            .tasks
            .read()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();
        tasks.sort_by_key(|task| std::cmp::Reverse(task.created_at));
        DownloadTaskList { tasks }
    }

    pub(crate) async fn offline_manifest(
        &self,
        chapter_id: &str,
    ) -> DownloadResult<Option<OfflineChapterManifest>> {
        Ok(self.repository.offline_manifest(chapter_id).await?)
    }

    pub async fn downloaded_chapters(&self) -> DownloadResult<DownloadedChapterList> {
        Ok(DownloadedChapterList {
            chapter_ids: self.repository.downloaded_chapters().await?,
        })
    }

    pub(crate) async fn offline_page(
        &self,
        chapter_id: &str,
        page: usize,
    ) -> DownloadResult<Option<CachedReaderPage>> {
        for task_id in self
            .repository
            .completed_manifest_task_ids(chapter_id)
            .await?
        {
            if let Some(page) = self.storage.read_page(&task_id, chapter_id, page).await? {
                return Ok(Some(page));
            }
        }
        Ok(None)
    }

    pub async fn pause(&self, task_id: &str) -> DownloadResult<DownloadTaskList> {
        self.stop_task(task_id, DownloadStatus::Paused, "pause")
            .await?;
        Ok(self.list().await)
    }

    pub async fn cancel(&self, task_id: &str) -> DownloadResult<DownloadTaskList> {
        self.stop_task(task_id, DownloadStatus::Cancelled, "cancel")
            .await?;
        Ok(self.list().await)
    }

    pub async fn resume(&self, task_id: &str) -> DownloadResult<DownloadTaskList> {
        let snapshot = {
            let mut tasks = self.tasks.write().await;
            let task = tasks.get_mut(task_id).ok_or(DownloadError::NotFound)?;
            if !matches!(
                task.status,
                DownloadStatus::Paused | DownloadStatus::Failed | DownloadStatus::Cancelled
            ) {
                return Err(invalid_state("resume", task.status));
            }

            task.status = DownloadStatus::Queued;
            task.error = None;
            task.total_pages = 0;
            task.completed_pages = 0;
            task.eta_seconds = None;
            task.speed_bytes_per_second = 0;
            task.completed_at = None;
            task.generation = task.generation.saturating_add(1);
            mark_task_updated(task);
            task.clone()
        };

        self.repository.persist_task(&snapshot).await?;
        self.progress_persisted_at.lock().await.remove(task_id);
        self.spawn(task_id.to_string());
        Ok(self.list().await)
    }

    pub async fn remove(&self, task_id: &str) -> DownloadResult<DownloadTaskList> {
        let snapshot = {
            let mut tasks = self.tasks.write().await;
            let task = tasks.get_mut(task_id).ok_or(DownloadError::NotFound)?;
            task.status = DownloadStatus::Cancelled;
            task.generation = task.generation.saturating_add(1);
            task.error = None;
            task.eta_seconds = None;
            task.speed_bytes_per_second = 0;
            task.completed_at = None;
            mark_task_updated(task);
            task.clone()
        };

        self.repository.persist_task(&snapshot).await?;
        self.progress_persisted_at.lock().await.remove(task_id);
        self.cancel_worker_and_wait(task_id).await?;
        let staged_deletion = self.storage.stage_task_deletion(task_id).await?;
        if let Err(error) = self.repository.delete_task(task_id).await {
            if let Some(staged) = staged_deletion.as_ref() {
                if let Err(restore_error) = self.storage.restore_staged_deletion(staged).await {
                    tracing::error!(task_id, %restore_error, "下载目录删除暂存恢复失败");
                }
            }
            return Err(error.into());
        }
        if let Err(error) = self.storage.finish_staged_deletion(staged_deletion).await {
            tracing::warn!(task_id, %error, "下载任务已删除，但暂存文件清理失败");
        }
        self.tasks.write().await.remove(task_id);
        Ok(self.list().await)
    }

    async fn stop_task(
        &self,
        task_id: &str,
        status: DownloadStatus,
        operation: &'static str,
    ) -> DownloadResult<()> {
        let snapshot = {
            let mut tasks = self.tasks.write().await;
            let task = tasks.get_mut(task_id).ok_or(DownloadError::NotFound)?;
            if !matches!(
                task.status,
                DownloadStatus::Queued | DownloadStatus::Running
            ) {
                return Err(invalid_state(operation, task.status));
            }
            task.status = status;
            task.eta_seconds = None;
            task.speed_bytes_per_second = 0;
            mark_task_updated(task);
            task.clone()
        };
        self.repository.persist_task(&snapshot).await?;
        self.progress_persisted_at.lock().await.remove(task_id);
        self.cancel_worker(task_id).await;
        Ok(())
    }

    pub(super) async fn jm_request<T, F>(&self, operation: F) -> JmResult<(String, T)>
    where
        F: for<'a> Fn(
            &'a JmClient,
            &'a str,
        ) -> Pin<Box<dyn Future<Output = JmResult<T>> + Send + 'a>>,
    {
        request_with_failover(&self.jm, &self.endpoints, operation).await
    }
}

fn validate_enqueue(input: &EnqueueDownload) -> DownloadResult<()> {
    if input.album_id.trim().is_empty() || input.chapters.is_empty() {
        return Err(DownloadError::InvalidRequest(
            "Download needs an album and at least one chapter".into(),
        ));
    }
    if input
        .chapters
        .iter()
        .any(|chapter| chapter.chapter_id.parse::<u32>().is_err())
    {
        return Err(DownloadError::InvalidRequest(
            "Download chapter ids must be numeric".into(),
        ));
    }
    Ok(())
}

fn invalid_state(operation: &'static str, status: DownloadStatus) -> DownloadError {
    DownloadError::InvalidState {
        operation,
        status: status.to_string(),
    }
}
