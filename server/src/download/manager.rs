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
        Self::new_with_storage_root(
            db,
            jm,
            endpoints,
            page_materializer,
            PathBuf::from("data/downloads"),
        )
        .await
    }

    async fn new_with_storage_root(
        db: SqlitePool,
        jm: Arc<JmClient>,
        endpoints: Arc<EndpointManager>,
        page_materializer: Arc<PageMaterializer>,
        storage_root: PathBuf,
    ) -> DownloadResult<Self> {
        let repository = DownloadRepository::new(db);
        let storage = DownloadStorage::new(storage_root).await?;
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

#[cfg(test)]
mod tests {
    use super::DownloadManager;
    use crate::{
        cache::{CachedReaderPage, ImageCache},
        download::{
            model::{DownloadChapter, DownloadStatus, DownloadTask},
            repository::DownloadRepository,
            DownloadError,
        },
        endpoint::EndpointManager,
        image_work::ImageWorkBudget,
        jm::JmClient,
        page_materializer::PageMaterializer,
        reader::PageImageFormat,
    };
    use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
    use std::{
        io::Cursor,
        path::PathBuf,
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc,
        },
        time::{Duration, Instant},
    };
    use tokio::{fs, sync::Semaphore, time::timeout};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    #[tokio::test]
    async fn pause_resume_and_cancel_follow_valid_transitions() {
        let db = test_db().await;
        let (manager, root) = test_manager("transitions", db).await;
        let mut task = test_task("task-transitions", DownloadStatus::Running, 1);
        task.total_pages = 12;
        task.completed_pages = 5;
        task.eta_seconds = Some(30);
        task.speed_bytes_per_second = 1024;
        insert_task(&manager, task).await;
        manager
            .progress_persisted_at
            .lock()
            .await
            .insert("task-transitions".into(), Instant::now());

        manager
            .pause("task-transitions")
            .await
            .expect("pause running task");
        let paused = task_snapshot(&manager, "task-transitions").await;
        assert_eq!(paused.status, DownloadStatus::Paused);
        assert_eq!(paused.eta_seconds, None);
        assert_eq!(paused.speed_bytes_per_second, 0);
        assert!(!manager
            .progress_persisted_at
            .lock()
            .await
            .contains_key("task-transitions"));

        manager
            .resume("task-transitions")
            .await
            .expect("resume paused task");
        wait_for_worker(&manager, "task-transitions", true).await;
        let resumed = task_snapshot(&manager, "task-transitions").await;
        assert_eq!(resumed.status, DownloadStatus::Queued);
        assert_eq!(resumed.generation, 2);
        assert_eq!(resumed.total_pages, 0);
        assert_eq!(resumed.completed_pages, 0);

        manager
            .cancel("task-transitions")
            .await
            .expect("cancel queued task");
        wait_for_worker(&manager, "task-transitions", false).await;
        let cancelled = persisted_task(&manager, "task-transitions").await;
        assert_eq!(cancelled.status, DownloadStatus::Cancelled);
        assert!(matches!(
            manager.pause("task-transitions").await,
            Err(DownloadError::InvalidState { .. })
        ));

        cleanup(manager, root).await;
    }

    #[tokio::test]
    async fn stale_generation_cannot_update_or_finish_resumed_task() {
        let db = test_db().await;
        let (manager, root) = test_manager("generation", db).await;
        insert_task(
            &manager,
            test_task("task-generation", DownloadStatus::Queued, 2),
        )
        .await;

        assert!(manager
            .begin_generation("task-generation", 1)
            .await
            .expect("ignore stale begin")
            .is_none());
        assert!(manager
            .begin_generation("task-generation", 2)
            .await
            .expect("begin current generation")
            .is_some());

        manager
            .complete_page("task-generation", 1, 512, Duration::from_secs(1))
            .await
            .expect("ignore stale page completion");
        manager
            .fail_generation("task-generation", 1, "stale failure".into())
            .await
            .expect("ignore stale failure");
        manager
            .complete_generation("task-generation", 1)
            .await
            .expect("ignore stale completion");
        let active = task_snapshot(&manager, "task-generation").await;
        assert_eq!(active.status, DownloadStatus::Running);
        assert_eq!(active.completed_pages, 0);
        assert_eq!(active.error, None);

        manager
            .complete_generation("task-generation", 2)
            .await
            .expect("complete current generation");
        assert_eq!(
            persisted_task(&manager, "task-generation").await.status,
            DownloadStatus::Completed
        );

        cleanup(manager, root).await;
    }

    #[tokio::test]
    async fn remove_waits_for_worker_and_deletes_manifest_and_files() {
        let db = test_db().await;
        let (manager, root) = test_manager("remove", db).await;
        let task = test_task("task-remove", DownloadStatus::Queued, 1);
        let chapter = task.chapters[0].clone();
        insert_task(&manager, task.clone()).await;
        manager
            .repository
            .persist_manifest(&task, &chapter, &["001.png".into()])
            .await
            .expect("persist manifest");
        manager
            .repository
            .complete_manifest(&task.task_id, &chapter.chapter_id)
            .await
            .expect("complete manifest");
        manager
            .storage
            .store_page(
                &task.task_id,
                &chapter.chapter_id,
                0,
                &CachedReaderPage {
                    data: png_data(),
                    format: PageImageFormat::Png,
                },
            )
            .await
            .expect("store downloaded page");
        manager.spawn(task.task_id.clone());
        wait_for_worker(&manager, &task.task_id, true).await;

        manager
            .remove(&task.task_id)
            .await
            .expect("remove active task");
        wait_for_worker(&manager, &task.task_id, false).await;

        assert!(manager.list().await.tasks.is_empty());
        assert!(manager
            .repository
            .load_tasks()
            .await
            .expect("load persisted tasks")
            .is_empty());
        assert!(manager
            .repository
            .offline_manifest(&chapter.chapter_id)
            .await
            .expect("load deleted manifest")
            .is_none());
        assert!(!fs::try_exists(root.join("downloads").join(&task.task_id))
            .await
            .expect("check deleted download directory"));

        cleanup(manager, root).await;
    }

    #[tokio::test]
    async fn offline_page_falls_back_from_corrupt_newer_download_and_expires_after_delete() {
        let db = test_db().await;
        let (manager, root) = test_manager("offline-fallback", db).await;
        let older = test_task("task-older", DownloadStatus::Completed, 1);
        let newer = test_task("task-newer", DownloadStatus::Completed, 1);
        let chapter = older.chapters[0].clone();
        insert_task(&manager, older.clone()).await;
        insert_task(&manager, newer.clone()).await;

        for (task, image) in [(&older, "older-001.png"), (&newer, "newer-001.png")] {
            manager
                .repository
                .persist_manifest(task, &chapter, &[image.into()])
                .await
                .expect("persist offline manifest");
            manager
                .repository
                .complete_manifest(&task.task_id, &chapter.chapter_id)
                .await
                .expect("complete offline manifest");
            tokio::time::sleep(Duration::from_millis(2)).await;
        }

        let older_data = png_data();
        manager
            .storage
            .store_page(
                &older.task_id,
                &chapter.chapter_id,
                0,
                &CachedReaderPage {
                    data: older_data.clone(),
                    format: PageImageFormat::Png,
                },
            )
            .await
            .expect("store older offline page");
        let newer_page = root
            .join("downloads")
            .join(&newer.task_id)
            .join(&chapter.chapter_id)
            .join("0.png");
        fs::create_dir_all(newer_page.parent().expect("newer page parent"))
            .await
            .expect("create newer page parent");
        let mut corrupt = png_data();
        corrupt.truncate(corrupt.len() - 12);
        fs::write(&newer_page, corrupt)
            .await
            .expect("write corrupt newer page");

        let offline = manager
            .offline_page(&chapter.chapter_id, 0)
            .await
            .expect("load offline page")
            .expect("older offline page should be used");
        assert_eq!(offline.format, PageImageFormat::Png);
        assert_eq!(offline.data, older_data);
        assert!(!newer_page.exists());

        manager
            .remove(&older.task_id)
            .await
            .expect("remove older task");
        assert!(manager
            .offline_page(&chapter.chapter_id, 0)
            .await
            .expect("load offline page after deletion")
            .is_none());
        manager
            .remove(&newer.task_id)
            .await
            .expect("remove newer task");

        cleanup(manager, root).await;
    }

    #[tokio::test]
    async fn restart_requeues_running_tasks_without_resuming_paused_tasks() {
        let db = test_db().await;
        let repository = DownloadRepository::new(db.clone());
        let mut running = test_task("task-running", DownloadStatus::Running, 3);
        running.total_pages = 20;
        running.completed_pages = 7;
        running.eta_seconds = Some(15);
        running.speed_bytes_per_second = 4096;
        repository
            .persist_task(&running)
            .await
            .expect("persist running task");
        repository
            .persist_task(&test_task("task-paused", DownloadStatus::Paused, 4))
            .await
            .expect("persist paused task");

        let (manager, root) = test_manager("restart", db).await;
        let recovered = task_snapshot(&manager, "task-running").await;
        assert_eq!(recovered.status, DownloadStatus::Queued);
        assert_eq!(recovered.generation, 3);
        assert_eq!(recovered.total_pages, 0);
        assert_eq!(recovered.completed_pages, 0);
        assert_eq!(recovered.eta_seconds, None);
        assert_eq!(recovered.speed_bytes_per_second, 0);
        assert_eq!(
            task_snapshot(&manager, "task-paused").await.status,
            DownloadStatus::Paused
        );

        manager.resume_pending().await;
        wait_for_worker(&manager, "task-running", true).await;
        assert!(!manager.workers.lock().await.contains_key("task-paused"));

        manager
            .pause("task-running")
            .await
            .expect("pause recovered task");
        wait_for_worker(&manager, "task-running", false).await;
        cleanup(manager, root).await;
    }

    async fn test_manager(name: &str, db: SqlitePool) -> (DownloadManager, PathBuf) {
        let root = test_root(name);
        let cache = Arc::new(
            ImageCache::new_for_test(db.clone(), 1024 * 1024, root.join("cache"))
                .await
                .expect("create image cache"),
        );
        let endpoints = Arc::new(
            EndpointManager::new(db.clone())
                .await
                .expect("create endpoint manager"),
        );
        let jm = Arc::new(JmClient::new().expect("create JM client"));
        let page_materializer = Arc::new(PageMaterializer::new(
            jm.clone(),
            endpoints.clone(),
            cache,
            ImageWorkBudget::new(),
        ));
        let mut manager = DownloadManager::new_with_storage_root(
            db,
            jm,
            endpoints,
            page_materializer,
            root.join("downloads"),
        )
        .await
        .expect("create download manager");
        manager.task_semaphore = Arc::new(Semaphore::new(0));
        (manager, root)
    }

    async fn test_db() -> SqlitePool {
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

    async fn insert_task(manager: &DownloadManager, task: DownloadTask) {
        manager
            .repository
            .persist_task(&task)
            .await
            .expect("persist test task");
        manager
            .tasks
            .write()
            .await
            .insert(task.task_id.clone(), task);
    }

    async fn task_snapshot(manager: &DownloadManager, task_id: &str) -> DownloadTask {
        manager
            .tasks
            .read()
            .await
            .get(task_id)
            .cloned()
            .expect("task should exist")
    }

    async fn persisted_task(manager: &DownloadManager, task_id: &str) -> DownloadTask {
        manager
            .repository
            .load_tasks()
            .await
            .expect("load persisted tasks")
            .into_iter()
            .find(|task| task.task_id == task_id)
            .expect("persisted task should exist")
    }

    async fn wait_for_worker(manager: &DownloadManager, task_id: &str, expected: bool) {
        timeout(Duration::from_secs(1), async {
            loop {
                if manager.workers.lock().await.contains_key(task_id) == expected {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("worker state did not settle");
    }

    async fn cleanup(manager: DownloadManager, root: PathBuf) {
        drop(manager);
        fs::remove_dir_all(root)
            .await
            .expect("remove test manager root");
    }

    fn test_task(task_id: &str, status: DownloadStatus, generation: u64) -> DownloadTask {
        DownloadTask {
            task_id: task_id.into(),
            album_id: "album-1".into(),
            comic_title: "测试漫画".into(),
            chapters: vec![DownloadChapter {
                chapter_id: "1001".into(),
                title: "第一章".into(),
                order: 1,
            }],
            status,
            current_chapter_title: String::new(),
            total_pages: 0,
            completed_pages: 0,
            eta_seconds: None,
            speed_bytes_per_second: 0,
            error: None,
            created_at: 1,
            started_at: None,
            updated_at: 1,
            completed_at: None,
            generation,
        }
    }

    fn png_data() -> Vec<u8> {
        let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            1,
            1,
            image::Rgb([1, 2, 3]),
        ));
        let mut bytes = Cursor::new(Vec::new());
        image
            .write_to(&mut bytes, image::ImageFormat::Png)
            .expect("encode png");
        bytes.into_inner()
    }

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "jm-boom-download-manager-{name}-{}-{}",
            std::process::id(),
            TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }
}
