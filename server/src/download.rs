use crate::{
    cache::{reader_page_cache_key, ImageCache},
    endpoint::{request_with_failover, EndpointManager},
    jm::{invalidate_img_host, JmClient, JmResult},
    reader::{decode_scrambled_image, encode_webp, needs_decoding, page_name_from_image},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::{
    collections::{HashMap, HashSet},
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Instant,
};
use tokio::{
    sync::{Mutex, RwLock},
    time::{sleep, Duration},
};

static TASK_SEQUENCE: AtomicU64 = AtomicU64::new(1);

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
}

#[derive(Clone, Debug, Serialize)]
pub struct DownloadTaskList {
    pub tasks: Vec<DownloadTask>,
}

#[derive(Clone)]
pub struct DownloadManager {
    db: SqlitePool,
    jm: Arc<JmClient>,
    endpoints: Arc<EndpointManager>,
    cache: Arc<ImageCache>,
    tasks: Arc<RwLock<HashMap<String, DownloadTask>>>,
    workers: Arc<Mutex<HashSet<String>>>,
}

impl DownloadManager {
    pub async fn new(
        db: SqlitePool,
        jm: Arc<JmClient>,
        endpoints: Arc<EndpointManager>,
        cache: Arc<ImageCache>,
    ) -> Result<Self> {
        let rows =
            sqlx::query_as::<_, (String, String)>("SELECT task_id, payload FROM download_tasks")
                .fetch_all(&db)
                .await?;
        let mut tasks = HashMap::new();
        for (task_id, payload) in rows {
            if let Ok(mut task) = serde_json::from_str::<DownloadTask>(&payload) {
                if task.status == DownloadStatus::Running {
                    task.status = DownloadStatus::Queued;
                }
                tasks.insert(task_id, task);
            }
        }
        Ok(Self {
            db,
            jm,
            endpoints,
            cache,
            tasks: Arc::new(RwLock::new(tasks)),
            workers: Arc::new(Mutex::new(HashSet::new())),
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

    pub async fn pause(&self, task_id: &str) -> Result<DownloadTaskList> {
        self.set_status(task_id, DownloadStatus::Paused).await?;
        Ok(self.list().await)
    }

    pub async fn cancel(&self, task_id: &str) -> Result<DownloadTaskList> {
        self.set_status(task_id, DownloadStatus::Cancelled).await?;
        Ok(self.list().await)
    }

    pub async fn resume(&self, task_id: &str) -> Result<DownloadTaskList> {
        let mut should_spawn = false;
        {
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
                task.updated_at = chrono::Utc::now().timestamp_millis();
                self.persist(task).await?;
                should_spawn = true;
            }
        }
        if should_spawn {
            self.spawn(task_id.to_string());
        }
        Ok(self.list().await)
    }

    pub async fn remove(&self, task_id: &str) -> Result<DownloadTaskList> {
        self.tasks.write().await.remove(task_id);
        sqlx::query("DELETE FROM download_tasks WHERE task_id = ?")
            .bind(task_id)
            .execute(&self.db)
            .await?;
        Ok(self.list().await)
    }

    fn spawn(&self, task_id: String) {
        let manager = self.clone();
        tokio::spawn(async move {
            if !manager.workers.lock().await.insert(task_id.clone()) {
                return;
            }
            if let Err(error) = manager.run_task(&task_id).await {
                let _ = manager.fail(&task_id, error.to_string()).await;
            }
            manager.workers.lock().await.remove(&task_id);

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

    async fn run_task(&self, task_id: &str) -> Result<()> {
        let chapters = {
            let mut tasks = self.tasks.write().await;
            let task = tasks
                .get_mut(task_id)
                .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
            if task.status != DownloadStatus::Queued {
                return Ok(());
            }
            task.status = DownloadStatus::Running;
            task.started_at
                .get_or_insert_with(|| chrono::Utc::now().timestamp_millis());
            task.updated_at = chrono::Utc::now().timestamp_millis();
            let chapters = task.chapters.clone();
            self.persist(task).await?;
            chapters
        };
        let started = Instant::now();
        let mut downloaded_bytes = 0u64;

        for chapter_request in chapters {
            if !self.wait_until_active(task_id).await? {
                return Ok(());
            }
            let comic_id = chapter_request.chapter_id.parse::<u32>()?;
            self.update_chapter(task_id, &chapter_request.title).await?;
            let request_chapter_id = chapter_request.chapter_id.clone();
            let (_, chapter) = self
                .jm_request(move |client, endpoint| {
                    let chapter_id = request_chapter_id.clone();
                    Box::pin(async move { client.get_chapter(endpoint, &chapter_id).await })
                })
                .await?;
            self.add_total_pages(task_id, chapter.images.len() as u32)
                .await?;

            for (index, image_path) in chapter.images.iter().enumerate() {
                if !self.wait_until_active(task_id).await? {
                    return Ok(());
                }
                let cache_key = reader_page_cache_key(&chapter_request.chapter_id, index);
                let file_name = image_path.split('/').last().unwrap_or(image_path);
                let page_name = page_name_from_image(image_path);
                let is_gif = file_name.to_ascii_lowercase().ends_with(".gif");
                let cached = if is_gif {
                    self.cache.get_gif(&cache_key).await?
                } else {
                    self.cache.get(&cache_key).await?
                };
                let size = match cached {
                    Some(data) => data.len() as u64,
                    None => {
                        let data = self
                            .download_page_image(&chapter_request.chapter_id, image_path)
                            .await?;
                        if is_gif {
                            self.cache.put_gif(&cache_key, &data).await?;
                            data.len() as u64
                        } else {
                            let encoded = tokio::task::spawn_blocking(move || {
                                let original = image::load_from_memory(&data)?;
                                let rgb = if needs_decoding(comic_id, &page_name, false) {
                                    decode_scrambled_image(original, comic_id, &page_name)
                                } else {
                                    original.to_rgb8()
                                };
                                Ok::<_, anyhow::Error>(encode_webp(&rgb))
                            })
                            .await??;
                            self.cache.put(&cache_key, &encoded).await?;
                            encoded.len() as u64
                        }
                    }
                };
                downloaded_bytes = downloaded_bytes.saturating_add(size);
                self.complete_page(task_id, downloaded_bytes, started.elapsed())
                    .await?;
            }
        }

        if !self.wait_until_active(task_id).await? {
            return Ok(());
        }

        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
        task.status = DownloadStatus::Completed;
        task.eta_seconds = Some(0);
        task.completed_at = Some(chrono::Utc::now().timestamp_millis());
        task.updated_at = chrono::Utc::now().timestamp_millis();
        self.persist(task).await
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

    async fn wait_until_active(&self, task_id: &str) -> Result<bool> {
        loop {
            let status = self
                .tasks
                .read()
                .await
                .get(task_id)
                .map(|task| task.status)
                .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
            match status {
                DownloadStatus::Paused => sleep(Duration::from_millis(300)).await,
                DownloadStatus::Cancelled => return Ok(false),
                DownloadStatus::Running | DownloadStatus::Queued => return Ok(true),
                _ => return Ok(false),
            }
        }
    }

    async fn set_status(&self, task_id: &str, status: DownloadStatus) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
        task.status = status;
        task.updated_at = chrono::Utc::now().timestamp_millis();
        self.persist(task).await
    }

    async fn update_chapter(&self, task_id: &str, title: &str) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
        task.current_chapter_title = title.to_string();
        task.updated_at = chrono::Utc::now().timestamp_millis();
        self.persist(task).await
    }

    async fn add_total_pages(&self, task_id: &str, count: u32) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
        task.total_pages = task.total_pages.saturating_add(count);
        task.updated_at = chrono::Utc::now().timestamp_millis();
        self.persist(task).await
    }

    async fn complete_page(&self, task_id: &str, bytes: u64, elapsed: Duration) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
        task.completed_pages = task.completed_pages.saturating_add(1);
        let seconds = elapsed.as_secs_f64().max(0.001);
        task.speed_bytes_per_second = (bytes as f64 / seconds) as u64;
        let remaining = task.total_pages.saturating_sub(task.completed_pages) as f64;
        let pages_per_second = task.completed_pages as f64 / seconds;
        task.eta_seconds =
            (pages_per_second > 0.0).then_some((remaining / pages_per_second) as u64);
        task.updated_at = chrono::Utc::now().timestamp_millis();
        self.persist(task).await
    }

    async fn fail(&self, task_id: &str, error: String) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| anyhow::anyhow!("Download task not found"))?;
        task.status = DownloadStatus::Failed;
        task.error = Some(error);
        task.updated_at = chrono::Utc::now().timestamp_millis();
        self.persist(task).await
    }

    async fn persist(&self, task: &DownloadTask) -> Result<()> {
        sqlx::query(
            "INSERT INTO download_tasks (task_id, payload, updated_at) VALUES (?, ?, ?) \
             ON CONFLICT(task_id) DO UPDATE SET payload = excluded.payload, updated_at = excluded.updated_at",
        )
        .bind(&task.task_id)
        .bind(serde_json::to_string(task)?)
        .bind(task.updated_at)
        .execute(&self.db)
        .await?;
        Ok(())
    }
}
