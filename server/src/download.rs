use crate::{
    cache::{reader_page_cache_key, CachedReaderPage, ImageCache},
    endpoint::{request_with_failover, EndpointManager},
    image_work::{ImageWorkBudget, ImageWorkPriority},
    jm::{invalidate_img_host, JmClient, JmResult},
    reader::{page_name_from_image, prepare_page_image},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
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
use tokio::{
    fs,
    sync::{watch, Mutex, RwLock, Semaphore},
    task::JoinSet,
    time::Duration,
};

static TASK_SEQUENCE: AtomicU64 = AtomicU64::new(1);
const PROGRESS_PERSIST_INTERVAL: Duration = Duration::from_secs(1);
const DOWNLOAD_TASK_CONCURRENCY: usize = 2;
const DOWNLOAD_PAGE_CONCURRENCY_PER_TASK: usize = 5;

fn default_generation() -> u64 {
    1
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadChapter {
    pub chapter_id: String,
    pub title: String,
    pub order: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnqueueDownload {
    pub album_id: String,
    pub comic_title: String,
    pub chapters: Vec<DownloadChapter>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Queued,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTask {
    pub task_id: String,
    pub album_id: String,
    pub comic_title: String,
    pub chapters: Vec<DownloadChapter>,
    pub status: DownloadStatus,
    pub current_chapter_title: String,
    pub total_pages: u32,
    pub completed_pages: u32,
    pub eta_seconds: Option<u64>,
    pub speed_bytes_per_second: u64,
    pub error: Option<String>,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
    #[serde(default = "default_generation")]
    pub generation: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct DownloadTaskList {
    pub tasks: Vec<DownloadTask>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadedChapterList {
    pub chapter_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OfflineChapterManifest {
    pub task_id: String,
    pub album_id: String,
    pub chapter_id: String,
    pub title: String,
    pub images: Vec<String>,
    pub updated_at: i64,
}

struct WorkerHandle {
    generation: u64,
    cancel: watch::Sender<bool>,
    finished: watch::Sender<bool>,
}

#[derive(Clone)]
pub struct DownloadManager {
    db: SqlitePool,
    jm: Arc<JmClient>,
    endpoints: Arc<EndpointManager>,
    cache: Arc<ImageCache>,
    image_work: ImageWorkBudget,
    download_dir: PathBuf,
    tasks: Arc<RwLock<HashMap<String, DownloadTask>>>,
    workers: Arc<Mutex<HashMap<String, WorkerHandle>>>,
    task_semaphore: Arc<Semaphore>,
    progress_persisted_at: Arc<Mutex<HashMap<String, Instant>>>,
}

impl DownloadManager {
    pub async fn new(
        db: SqlitePool,
        jm: Arc<JmClient>,
        endpoints: Arc<EndpointManager>,
        cache: Arc<ImageCache>,
        image_work: ImageWorkBudget,
    ) -> Result<Self> {
        let download_dir = PathBuf::from("data/downloads");
        fs::create_dir_all(&download_dir).await?;
        let rows =
            sqlx::query_as::<_, (String, String)>("SELECT task_id, payload FROM download_tasks")
                .fetch_all(&db)
                .await?;
        let mut tasks = HashMap::new();
        for (task_id, payload) in rows {
            if let Ok(mut task) = serde_json::from_str::<DownloadTask>(&payload) {
                if task.status == DownloadStatus::Running {
                    task.status = DownloadStatus::Queued;
                    task.total_pages = 0;
                    task.completed_pages = 0;
                    task.eta_seconds = None;
                    task.speed_bytes_per_second = 0;
                    task.completed_at = None;
                }
                tasks.insert(task_id, task);
            }
        }
        Ok(Self {
            db,
            jm,
            endpoints,
            cache,
            image_work,
            download_dir,
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

    pub async fn enqueue(&self, input: EnqueueDownload) -> Result<DownloadTaskList> {
        if input.album_id.trim().is_empty() || input.chapters.is_empty() {
            anyhow::bail!("Download needs an album and at least one chapter");
        }
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
        self.tasks
            .write()
            .await
            .insert(task_id.clone(), task.clone());
        self.persist(&task).await?;
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

    pub async fn offline_manifest(
        &self,
        chapter_id: &str,
    ) -> Result<Option<OfflineChapterManifest>> {
        let payload = sqlx::query_scalar::<_, String>(
            "SELECT payload FROM download_manifests WHERE chapter_id = ? \
             ORDER BY updated_at DESC LIMIT 1",
        )
        .bind(chapter_id)
        .fetch_optional(&self.db)
        .await?;

        payload
            .map(|payload| serde_json::from_str(&payload).map_err(Into::into))
            .transpose()
    }

    pub async fn downloaded_chapters(&self) -> Result<DownloadedChapterList> {
        let chapter_ids = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT chapter_id FROM download_manifests \
             WHERE completed = 1 ORDER BY chapter_id",
        )
        .fetch_all(&self.db)
        .await?;
        Ok(DownloadedChapterList { chapter_ids })
    }

    pub async fn offline_page(
        &self,
        chapter_id: &str,
        page: usize,
    ) -> Result<Option<CachedReaderPage>> {
        let task_ids = sqlx::query_scalar::<_, String>(
            "SELECT task_id FROM download_manifests WHERE chapter_id = ? \
             ORDER BY updated_at DESC",
        )
        .bind(chapter_id)
        .fetch_all(&self.db)
        .await?;

        for task_id in task_ids {
            if let Some(page) = self.read_offline_page(&task_id, chapter_id, page).await? {
                return Ok(Some(page));
            }
        }
        Ok(None)
    }

    pub async fn pause(&self, task_id: &str) -> Result<DownloadTaskList> {
        self.set_status(task_id, DownloadStatus::Paused).await?;
        self.cancel_worker(task_id).await;
        Ok(self.list().await)
    }

    pub async fn cancel(&self, task_id: &str) -> Result<DownloadTaskList> {
        self.set_status(task_id, DownloadStatus::Cancelled).await?;
        self.cancel_worker(task_id).await;
        Ok(self.list().await)
    }

    pub async fn resume(&self, task_id: &str) -> Result<DownloadTaskList> {
        let snapshot = {
            let mut tasks = self.tasks.write().await;
            let task = tasks
                .get_mut(task_id)
                .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
            if matches!(
                task.status,
                DownloadStatus::Paused | DownloadStatus::Failed | DownloadStatus::Cancelled
            ) {
                task.status = DownloadStatus::Queued;
                task.error = None;
                task.total_pages = 0;
                task.completed_pages = 0;
                task.eta_seconds = None;
                task.speed_bytes_per_second = 0;
                task.completed_at = None;
                task.generation = task.generation.saturating_add(1);
                mark_task_updated(task);
                Some(task.clone())
            } else {
                None
            }
        };
        if let Some(snapshot) = snapshot {
            self.persist(&snapshot).await?;
            self.progress_persisted_at.lock().await.remove(task_id);
            self.spawn(task_id.to_string());
        }
        Ok(self.list().await)
    }

    pub async fn remove(&self, task_id: &str) -> Result<DownloadTaskList> {
        self.cancel_worker_and_wait(task_id).await?;
        self.tasks.write().await.remove(task_id);
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
        match fs::remove_dir_all(self.download_dir.join(task_id)).await {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        Ok(self.list().await)
    }

    fn spawn(&self, task_id: String) {
        let manager = self.clone();
        tokio::spawn(async move {
            let generation = match manager.task_generation(&task_id).await {
                Some(generation) => generation,
                None => return,
            };
            let (cancel, mut cancelled) = watch::channel(false);
            let (finished, _) = watch::channel(false);
            {
                let mut workers = manager.workers.lock().await;
                if workers.contains_key(&task_id) {
                    return;
                }
                workers.insert(
                    task_id.clone(),
                    WorkerHandle {
                        generation,
                        cancel,
                        finished,
                    },
                );
            }

            let permit = tokio::select! {
                permit = manager.task_semaphore.clone().acquire_owned() => permit.ok(),
                _ = wait_for_cancel(&mut cancelled) => None,
            };
            if let Some(_permit) = permit {
                if let Err(error) = manager.run_task(&task_id, generation, &cancelled).await {
                    let _ = manager
                        .fail_generation(&task_id, generation, error.to_string())
                        .await;
                }
            }
            manager.finish_worker(&task_id, generation).await;

            let should_restart = manager
                .tasks
                .read()
                .await
                .get(&task_id)
                .is_some_and(|task| task.status == DownloadStatus::Queued);
            if should_restart {
                manager.spawn(task_id);
            }
        });
    }

    async fn run_task(
        &self,
        task_id: &str,
        generation: u64,
        cancelled: &watch::Receiver<bool>,
    ) -> Result<()> {
        let (chapters, snapshot) = {
            let mut tasks = self.tasks.write().await;
            let task = tasks
                .get_mut(task_id)
                .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
            if task.status != DownloadStatus::Queued || task.generation != generation {
                return Ok(());
            }
            task.status = DownloadStatus::Running;
            task.started_at
                .get_or_insert_with(|| chrono::Utc::now().timestamp_millis());
            mark_task_updated(task);
            let chapters = task.chapters.clone();
            (chapters, task.clone())
        };
        self.persist(&snapshot).await?;
        let started = Instant::now();
        let mut downloaded_bytes = 0u64;
        let mut resolved_chapters = Vec::with_capacity(chapters.len());

        for chapter_request in chapters {
            if !self
                .is_generation_active(task_id, generation, cancelled)
                .await
            {
                return Ok(());
            }
            self.update_chapter(task_id, generation, &chapter_request.title)
                .await?;
            let request_chapter_id = chapter_request.chapter_id.clone();
            let (_, chapter) = self
                .jm_request(move |client, endpoint| {
                    let chapter_id = request_chapter_id.clone();
                    Box::pin(async move { client.get_chapter(endpoint, &chapter_id).await })
                })
                .await?;
            if !self
                .is_generation_active(task_id, generation, cancelled)
                .await
            {
                return Ok(());
            }
            self.persist_offline_manifest(&snapshot, &chapter_request, &chapter.images)
                .await?;
            resolved_chapters.push((chapter_request, chapter));
        }

        let total_pages = resolved_chapters
            .iter()
            .map(|(_, chapter)| chapter.images.len() as u32)
            .sum();
        self.set_total_pages(task_id, generation, total_pages)
            .await?;

        for (chapter_request, chapter) in resolved_chapters {
            if !self
                .is_generation_active(task_id, generation, cancelled)
                .await
            {
                return Ok(());
            }
            let comic_id = chapter_request.chapter_id.parse::<u32>()?;
            self.update_chapter(task_id, generation, &chapter_request.title)
                .await?;
            let mut jobs = JoinSet::new();
            let mut next_page = 0usize;
            while next_page < chapter.images.len() || !jobs.is_empty() {
                while next_page < chapter.images.len()
                    && jobs.len() < DOWNLOAD_PAGE_CONCURRENCY_PER_TASK
                {
                    let manager = self.clone();
                    let task_id = task_id.to_string();
                    let chapter_id = chapter_request.chapter_id.clone();
                    let image_path = chapter.images[next_page].clone();
                    let page = next_page;
                    let cancelled = cancelled.clone();
                    jobs.spawn(async move {
                        manager
                            .process_page_job(
                                &task_id,
                                generation,
                                &cancelled,
                                comic_id,
                                &chapter_id,
                                page,
                                &image_path,
                            )
                            .await
                    });
                    next_page += 1;
                }

                if let Some(result) = jobs.join_next().await {
                    if let Some(size) = result?? {
                        downloaded_bytes = downloaded_bytes.saturating_add(size);
                        self.complete_page(
                            task_id,
                            generation,
                            downloaded_bytes,
                            started.elapsed(),
                        )
                        .await?;
                    }
                }
            }
            self.mark_offline_manifest_completed(task_id, &chapter_request.chapter_id)
                .await?;
        }

        if !self
            .is_generation_active(task_id, generation, cancelled)
            .await
        {
            return Ok(());
        }

        let snapshot = {
            let mut tasks = self.tasks.write().await;
            let task = tasks
                .get_mut(task_id)
                .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
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
        self.persist(&snapshot).await
    }

    async fn jm_request<T, F>(&self, operation: F) -> JmResult<(String, T)>
    where
        F: for<'a> Fn(
            &'a JmClient,
            &'a str,
        ) -> Pin<Box<dyn Future<Output = JmResult<T>> + Send + 'a>>,
    {
        request_with_failover(&self.jm, &self.endpoints, operation).await
    }

    async fn process_page_job(
        &self,
        task_id: &str,
        generation: u64,
        cancelled: &watch::Receiver<bool>,
        comic_id: u32,
        chapter_id: &str,
        page: usize,
        image_path: &str,
    ) -> Result<Option<u64>> {
        if !self
            .is_generation_active(task_id, generation, cancelled)
            .await
        {
            return Ok(None);
        }

        let offline = self.read_offline_page(task_id, chapter_id, page).await?;
        let was_offline = offline.is_some();
        let cache_key = reader_page_cache_key(chapter_id, page);
        let page_image = match offline {
            Some(offline) => offline,
            None => match self.cache.get_reader_page(&cache_key).await? {
                Some(cached) => cached,
                None => {
                    let mut cancel_wait = cancelled.clone();
                    let work_permit = tokio::select! {
                        permit = self.image_work.acquire(ImageWorkPriority::Download) => Some(permit),
                        _ = wait_for_cancel(&mut cancel_wait) => None,
                    };
                    let Some(_work_permit) = work_permit else {
                        return Ok(None);
                    };

                    match self.cache.get_reader_page(&cache_key).await? {
                        Some(cached) => cached,
                        None => {
                            let data = self.download_page_image(chapter_id, image_path).await?;
                            if !self
                                .is_generation_active(task_id, generation, cancelled)
                                .await
                            {
                                return Ok(None);
                            }
                            let page_name = page_name_from_image(image_path);
                            let prepared = prepare_page_image(data, comic_id, page_name).await?;
                            if !self
                                .is_generation_active(task_id, generation, cancelled)
                                .await
                            {
                                return Ok(None);
                            }
                            self.cache
                                .put_reader_page(&cache_key, prepared.format, &prepared.data)
                                .await?;
                            CachedReaderPage {
                                data: prepared.data,
                                format: prepared.format,
                            }
                        }
                    }
                }
            },
        };

        if !self
            .is_generation_active(task_id, generation, cancelled)
            .await
        {
            return Ok(None);
        }
        if !was_offline {
            self.store_offline_page(task_id, chapter_id, page, &page_image)
                .await?;
        }
        Ok(Some(page_image.data.len() as u64))
    }

    async fn download_page_image(&self, chapter_id: &str, image_path: &str) -> JmResult<Vec<u8>> {
        let (endpoint, img_host) = self
            .jm_request(|client, endpoint| Box::pin(client.get_img_host(endpoint)))
            .await?;
        let image_url = format!("{img_host}/media/photos/{chapter_id}/{image_path}");

        match self.jm.download_image(&image_url).await {
            Ok(data) => Ok(data),
            Err(_) => {
                invalidate_img_host(&endpoint).await;
                let (_, refreshed_host) = self
                    .jm_request(|client, endpoint| Box::pin(client.get_img_host(endpoint)))
                    .await?;
                let refreshed_url =
                    format!("{refreshed_host}/media/photos/{chapter_id}/{image_path}");
                self.jm.download_image(&refreshed_url).await
            }
        }
    }

    async fn persist_offline_manifest(
        &self,
        task: &DownloadTask,
        chapter: &DownloadChapter,
        images: &[String],
    ) -> Result<()> {
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
        .bind(serde_json::to_string(&manifest)?)
        .bind(manifest.updated_at)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn mark_offline_manifest_completed(&self, task_id: &str, chapter_id: &str) -> Result<()> {
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

    async fn store_offline_page(
        &self,
        task_id: &str,
        chapter_id: &str,
        page: usize,
        page_image: &CachedReaderPage,
    ) -> Result<()> {
        let path = self.offline_page_path(task_id, chapter_id, page, page_image.format.extension());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, &page_image.data).await?;
        Ok(())
    }

    async fn read_offline_page(
        &self,
        task_id: &str,
        chapter_id: &str,
        page: usize,
    ) -> Result<Option<CachedReaderPage>> {
        for format in crate::reader::PageImageFormat::supported() {
            let path = self.offline_page_path(task_id, chapter_id, page, format.extension());
            match fs::read(path).await {
                Ok(data) => return Ok(Some(CachedReaderPage { data, format })),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(None)
    }

    fn offline_page_path(
        &self,
        task_id: &str,
        chapter_id: &str,
        page: usize,
        extension: &str,
    ) -> PathBuf {
        self.download_dir
            .join(task_id)
            .join(chapter_id)
            .join(format!("{page}.{extension}"))
    }

    async fn task_generation(&self, task_id: &str) -> Option<u64> {
        self.tasks
            .read()
            .await
            .get(task_id)
            .map(|task| task.generation)
    }

    async fn is_generation_active(
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

    async fn cancel_worker(&self, task_id: &str) {
        let cancel = self
            .workers
            .lock()
            .await
            .get(task_id)
            .map(|worker| worker.cancel.clone());
        if let Some(cancel) = cancel {
            let _ = cancel.send(true);
        }
    }

    async fn cancel_worker_and_wait(&self, task_id: &str) -> Result<()> {
        let handles = self
            .workers
            .lock()
            .await
            .get(task_id)
            .map(|worker| (worker.cancel.clone(), worker.finished.subscribe()));
        let Some((cancel, mut finished)) = handles else {
            return Ok(());
        };
        let _ = cancel.send(true);
        if !*finished.borrow() {
            tokio::time::timeout(Duration::from_secs(30), finished.changed())
                .await
                .map_err(|_| anyhow::anyhow!("等待下载 worker 退出超时"))??;
        }
        Ok(())
    }

    async fn finish_worker(&self, task_id: &str, generation: u64) {
        let mut workers = self.workers.lock().await;
        if workers
            .get(task_id)
            .is_some_and(|worker| worker.generation == generation)
        {
            if let Some(worker) = workers.get(task_id) {
                let _ = worker.finished.send(true);
            }
            workers.remove(task_id);
        }
    }

    async fn set_status(&self, task_id: &str, status: DownloadStatus) -> Result<()> {
        let snapshot = {
            let mut tasks = self.tasks.write().await;
            let task = tasks
                .get_mut(task_id)
                .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
            task.status = status;
            mark_task_updated(task);
            task.clone()
        };
        self.persist(&snapshot).await
    }

    async fn update_chapter(&self, task_id: &str, generation: u64, title: &str) -> Result<()> {
        let snapshot = {
            let mut tasks = self.tasks.write().await;
            let task = tasks
                .get_mut(task_id)
                .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
            if task.generation != generation || task.status != DownloadStatus::Running {
                return Ok(());
            }
            task.current_chapter_title = title.to_string();
            mark_task_updated(task);
            task.clone()
        };
        self.persist(&snapshot).await
    }

    async fn set_total_pages(&self, task_id: &str, generation: u64, total: u32) -> Result<()> {
        let snapshot = {
            let mut tasks = self.tasks.write().await;
            let task = tasks
                .get_mut(task_id)
                .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
            if task.generation != generation || task.status != DownloadStatus::Running {
                return Ok(());
            }
            task.total_pages = total;
            mark_task_updated(task);
            task.clone()
        };
        self.persist(&snapshot).await
    }

    async fn complete_page(
        &self,
        task_id: &str,
        generation: u64,
        bytes: u64,
        elapsed: Duration,
    ) -> Result<()> {
        let snapshot = {
            let mut tasks = self.tasks.write().await;
            let task = tasks
                .get_mut(task_id)
                .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
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
            self.persist(&snapshot).await?;
        }
        Ok(())
    }

    async fn fail_generation(&self, task_id: &str, generation: u64, error: String) -> Result<()> {
        let snapshot = {
            let mut tasks = self.tasks.write().await;
            let task = tasks
                .get_mut(task_id)
                .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
            if task.generation != generation || task.status != DownloadStatus::Running {
                return Ok(());
            }
            task.status = DownloadStatus::Failed;
            task.error = Some(error);
            mark_task_updated(task);
            task.clone()
        };
        self.progress_persisted_at.lock().await.remove(task_id);
        self.persist(&snapshot).await
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

    async fn persist(&self, task: &DownloadTask) -> Result<()> {
        let started = Instant::now();
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
        tracing::debug!(
            task_id = task.task_id,
            status = ?task.status,
            completed_pages = task.completed_pages,
            total_pages = task.total_pages,
            elapsed_ms = started.elapsed().as_millis(),
            "下载任务持久化完成"
        );
        Ok(())
    }
}

fn mark_task_updated(task: &mut DownloadTask) {
    task.updated_at = chrono::Utc::now()
        .timestamp_millis()
        .max(task.updated_at.saturating_add(1));
}

async fn wait_for_cancel(cancelled: &mut watch::Receiver<bool>) {
    if *cancelled.borrow() {
        return;
    }
    let _ = cancelled.changed().await;
}
