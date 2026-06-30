use super::paths::{
    download_chapter_dir, download_files_root, file_size_bytes, map_download_error,
    remove_task_files, task_output_dir,
};
use super::progress::{current_timestamp, estimate_eta, estimate_speed, short_hash};
use super::storage::{
    load_tasks, migrate_pending_task_output_dirs, persist_tasks, recover_interrupted_tasks,
};
use super::types::{
    DownloadChapterRequest, DownloadTask, DownloadTaskListResult, DownloadTaskStatus,
    EnqueueDownloadRequest,
};
use crate::api::{resolve_api_endpoint, ApiError, ApiErrorKind, ApiResult};
use crate::diagnostics;
use crate::reader;
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::async_runtime::{self, JoinHandle};
use tauri::AppHandle;
use tokio::sync::Mutex as AsyncMutex;

static DOWNLOAD_MANAGER: OnceLock<Arc<DownloadManager>> = OnceLock::new();
const DOWNLOAD_PROGRESS_PERSIST_INTERVAL: Duration = Duration::from_millis(1000);

pub(crate) struct DownloadManager {
    app: AppHandle,
    state: Mutex<DownloadState>,
    persist_lock: AsyncMutex<()>,
}

#[derive(Default)]
struct DownloadState {
    tasks: Vec<DownloadTask>,
    worker: Option<JoinHandle<()>>,
    loaded: bool,
    revision: u64,
    persisted_revision: u64,
    last_persisted_at: Option<Instant>,
}

#[derive(Clone, Copy)]
enum PersistMode {
    Force,
    Throttled,
}

pub(crate) fn download_manager(app: AppHandle) -> Arc<DownloadManager> {
    DOWNLOAD_MANAGER
        .get_or_init(|| {
            Arc::new(DownloadManager {
                app,
                state: Mutex::new(DownloadState::default()),
                persist_lock: AsyncMutex::new(()),
            })
        })
        .clone()
}

impl DownloadManager {
    pub(crate) async fn enqueue(
        self: &Arc<Self>,
        request: EnqueueDownloadRequest,
    ) -> ApiResult<()> {
        let album_id = normalize_required(&request.album_id, "album_id")?;
        let comic_title = normalize_required(&request.comic_title, "comic_title")?;
        let endpoint = resolve_api_endpoint(request.endpoint)?;
        let chapters = normalize_chapters(request.chapters)?;
        self.load_tasks_if_needed().await?;
        let now = current_timestamp();
        let task_id = format!(
            "{now}-{}",
            short_hash(&format!("{album_id}-{comic_title}-{now}"))
        );
        let output_dir = task_output_dir(&self.app, &comic_title)?
            .to_string_lossy()
            .to_string();
        let task = DownloadTask {
            task_id,
            album_id,
            comic_title,
            endpoint,
            chapters,
            status: DownloadTaskStatus::Queued,
            current_chapter_title: String::new(),
            total_pages: 0,
            completed_pages: 0,
            eta_seconds: None,
            speed_bytes_per_second: 0,
            output_dir,
            error: None,
            created_at: now,
            started_at: None,
            updated_at: now,
            completed_at: None,
        };

        {
            let mut state = self.lock_state()?;
            state.tasks.push(task);
            bump_revision(&mut state);
        }
        self.persist_latest_tasks().await?;

        self.ensure_worker();

        Ok(())
    }

    pub(crate) async fn list(self: &Arc<Self>) -> ApiResult<DownloadTaskListResult> {
        self.load_tasks_if_needed().await?;
        let mut tasks = self.lock_state()?.tasks.clone();
        let should_resume_worker = tasks
            .iter()
            .any(|task| task.status == DownloadTaskStatus::Queued);
        tasks.sort_by_key(|task| std::cmp::Reverse(task.created_at));

        if should_resume_worker {
            self.ensure_worker();
        }

        Ok(DownloadTaskListResult {
            root_dir: download_files_root(&self.app)?
                .to_string_lossy()
                .to_string(),
            tasks,
        })
    }

    pub(crate) async fn cancel(self: &Arc<Self>, task_id: String) -> ApiResult<()> {
        let task_id = task_id.trim();
        if task_id.is_empty() {
            return Ok(());
        }

        self.load_tasks_if_needed().await?;
        {
            let mut state = self.lock_state()?;
            if let Some(task) = state.tasks.iter_mut().find(|task| task.task_id == task_id) {
                if matches!(
                    task.status,
                    DownloadTaskStatus::Queued
                        | DownloadTaskStatus::Running
                        | DownloadTaskStatus::Paused
                ) {
                    task.status = DownloadTaskStatus::Cancelled;
                    task.updated_at = current_timestamp();
                    task.completed_at = Some(task.updated_at);
                    task.eta_seconds = None;
                    task.speed_bytes_per_second = 0;
                    task.error = None;
                }
            }
            bump_revision(&mut state);
        }
        self.persist_latest_tasks().await
    }

    pub(crate) async fn pause(&self, task_id: String) -> ApiResult<()> {
        let task_id = task_id.trim();
        if task_id.is_empty() {
            return Ok(());
        }

        self.load_tasks_if_needed().await?;
        {
            let mut state = self.lock_state()?;
            if let Some(task) = state.tasks.iter_mut().find(|task| task.task_id == task_id) {
                if matches!(
                    task.status,
                    DownloadTaskStatus::Queued | DownloadTaskStatus::Running
                ) {
                    task.status = DownloadTaskStatus::Paused;
                    task.updated_at = current_timestamp();
                    task.completed_at = None;
                    task.eta_seconds = None;
                    task.speed_bytes_per_second = 0;
                    task.error = None;
                }
            }
            bump_revision(&mut state);
        }
        self.persist_latest_tasks().await
    }

    pub(crate) async fn resume(self: &Arc<Self>, task_id: String) -> ApiResult<()> {
        let task_id = task_id.trim();
        if task_id.is_empty() {
            return Ok(());
        }

        let mut resumed = false;
        self.load_tasks_if_needed().await?;
        {
            let mut state = self.lock_state()?;
            if let Some(task) = state.tasks.iter_mut().find(|task| task.task_id == task_id) {
                if matches!(
                    task.status,
                    DownloadTaskStatus::Paused | DownloadTaskStatus::Failed
                ) {
                    task.status = DownloadTaskStatus::Queued;
                    task.started_at = None;
                    task.completed_at = None;
                    task.current_chapter_title.clear();
                    task.total_pages = 0;
                    task.completed_pages = 0;
                    task.eta_seconds = None;
                    task.speed_bytes_per_second = 0;
                    task.error = None;
                    task.updated_at = current_timestamp();
                    resumed = true;
                }
            }
            bump_revision(&mut state);
        }
        self.persist_latest_tasks().await?;

        if resumed {
            self.ensure_worker();
        }

        Ok(())
    }

    pub(crate) async fn remove(&self, task_id: String) -> ApiResult<()> {
        let task_id = task_id.trim();
        if task_id.is_empty() {
            return Ok(());
        }

        self.load_tasks_if_needed().await?;
        let task_to_remove = {
            let state = self.lock_state()?;
            state
                .tasks
                .iter()
                .find(|task| task.task_id == task_id && task.status != DownloadTaskStatus::Running)
                .cloned()
        };
        if let Some(task) = &task_to_remove {
            remove_task_files(&self.app, task)?;
        }
        {
            let mut state = self.lock_state()?;
            state.tasks.retain(|task| {
                if task.task_id != task_id {
                    return true;
                }

                matches!(task.status, DownloadTaskStatus::Running)
            });
            bump_revision(&mut state);
        }
        self.persist_latest_tasks().await
    }

    pub(crate) async fn open_task_dir(&self, task_id: String) -> ApiResult<()> {
        self.load_tasks_if_needed().await?;
        let task_id = task_id.trim();
        let task = self
            .lock_state()?
            .tasks
            .iter()
            .find(|task| task.task_id == task_id)
            .cloned()
            .ok_or_else(|| ApiError::new(ApiErrorKind::MissingData, "Download task not found"))?;
        let output_dir = PathBuf::from(task.output_dir);
        fs::create_dir_all(&output_dir).map_err(map_download_error)?;
        tauri_plugin_opener::open_path(&output_dir, None::<&str>)
            .map_err(|error| ApiError::new(ApiErrorKind::Cache, error.to_string()))
    }

    fn ensure_worker(self: &Arc<Self>) {
        let mut state = match self.state.lock() {
            Ok(state) => state,
            Err(_) => return,
        };

        let worker_active = state
            .worker
            .as_ref()
            .is_some_and(|worker| !worker.inner().is_finished());
        if worker_active {
            return;
        }

        let manager = self.clone();
        state.worker = Some(async_runtime::spawn(async move {
            manager.process_queue().await;
        }));
    }

    async fn process_queue(self: Arc<Self>) {
        loop {
            let next_task = match self.mark_next_task_running().await {
                Ok(task) => task,
                Err(error) => {
                    diagnostics::warn(format!("Failed to read download queue: {error}"));
                    None
                }
            };

            let Some(task) = next_task else {
                if let Ok(mut state) = self.state.lock() {
                    state.worker = None;
                }
                return;
            };

            let result = self.run_task(task.clone()).await;
            if let Err(error) = result {
                let _ = self
                    .mark_task_failed(&task.task_id, error.to_string())
                    .await;
            }
        }
    }

    async fn mark_next_task_running(&self) -> ApiResult<Option<DownloadTask>> {
        self.load_tasks_if_needed().await?;
        let task = {
            let mut state = self.lock_state()?;
            let Some(index) = state
                .tasks
                .iter()
                .position(|task| task.status == DownloadTaskStatus::Queued)
            else {
                return Ok(None);
            };
            let now = current_timestamp();
            let task = &mut state.tasks[index];
            task.status = DownloadTaskStatus::Running;
            task.started_at = Some(now);
            task.updated_at = now;
            task.completed_at = None;
            task.current_chapter_title.clear();
            task.total_pages = 0;
            task.completed_pages = 0;
            task.eta_seconds = None;
            task.speed_bytes_per_second = 0;
            task.error = None;
            let task = task.clone();
            bump_revision(&mut state);
            task
        };
        self.persist_latest_tasks().await?;

        Ok(Some(task))
    }

    async fn run_task(&self, task: DownloadTask) -> ApiResult<()> {
        let task_started_at = std::time::Instant::now();
        let mut completed_pages: u32 = 0;
        let mut total_pages: u32 = 0;
        let mut downloaded_bytes: u64 = 0;
        let mut manifests = VecDeque::new();

        for chapter in &task.chapters {
            self.ensure_task_can_continue(&task.task_id)?;
            let manifest = reader::get_or_load_manifest(
                chapter.chapter_id.clone(),
                Some(task.endpoint.clone()),
            )
            .await?;
            total_pages = total_pages.saturating_add(manifest.page_count());
            manifests.push_back((chapter.clone(), manifest));
            self.update_task(&task.task_id, PersistMode::Force, |task| {
                task.total_pages = total_pages;
                task.current_chapter_title = chapter.title.clone();
            })
            .await?;
        }

        while let Some((chapter, manifest)) = manifests.pop_front() {
            self.ensure_task_can_continue(&task.task_id)?;
            self.update_task(&task.task_id, PersistMode::Force, |task| {
                task.current_chapter_title = chapter.title.clone();
            })
            .await?;
            let chapter_dir = download_chapter_dir(&task.output_dir, &chapter);

            for index in 0..manifest.page_count() {
                self.ensure_task_can_continue(&task.task_id)?;
                let extension = reader::reader_page_output_extension(&manifest, index)?;
                let target_path = chapter_dir.join(format!("{:04}.{extension}", index + 1));
                let (_, _, is_cached) =
                    reader::materialize_reader_page_to_path(&manifest, index, target_path.clone())
                        .await?;
                completed_pages = completed_pages.saturating_add(1);
                if !is_cached {
                    downloaded_bytes =
                        downloaded_bytes.saturating_add(file_size_bytes(&target_path).unwrap_or(0));
                }
                let eta_seconds =
                    estimate_eta(task_started_at.elapsed(), completed_pages, total_pages);
                let speed_bytes_per_second =
                    estimate_speed(task_started_at.elapsed(), downloaded_bytes);
                self.update_task(&task.task_id, PersistMode::Throttled, |task| {
                    task.completed_pages = completed_pages;
                    task.total_pages = total_pages;
                    task.eta_seconds = eta_seconds;
                    task.speed_bytes_per_second = speed_bytes_per_second;
                    task.current_chapter_title = chapter.title.clone();
                })
                .await?;
            }
        }

        self.update_task(&task.task_id, PersistMode::Force, |task| {
            let now = current_timestamp();
            task.status = DownloadTaskStatus::Completed;
            task.completed_pages = total_pages;
            task.total_pages = total_pages;
            task.eta_seconds = Some(0);
            task.speed_bytes_per_second = 0;
            task.updated_at = now;
            task.completed_at = Some(now);
        })
        .await
    }

    fn ensure_task_can_continue(&self, task_id: &str) -> ApiResult<()> {
        let status = self
            .lock_state()?
            .tasks
            .iter()
            .find(|task| task.task_id == task_id)
            .map(|task| task.status)
            .unwrap_or(DownloadTaskStatus::Cancelled);

        if status == DownloadTaskStatus::Cancelled {
            return Err(ApiError::new(
                ApiErrorKind::Cache,
                "Download task cancelled",
            ));
        }

        if status == DownloadTaskStatus::Paused {
            return Err(ApiError::new(ApiErrorKind::Cache, "Download task paused"));
        }

        Ok(())
    }

    async fn mark_task_failed(&self, task_id: &str, message: String) -> ApiResult<()> {
        self.update_task(task_id, PersistMode::Force, |task| {
            if matches!(
                task.status,
                DownloadTaskStatus::Cancelled | DownloadTaskStatus::Paused
            ) {
                return;
            }

            let now = current_timestamp();
            task.status = DownloadTaskStatus::Failed;
            task.error = Some(message);
            task.eta_seconds = None;
            task.speed_bytes_per_second = 0;
            task.updated_at = now;
            task.completed_at = Some(now);
        })
        .await
    }

    async fn update_task<F>(
        &self,
        task_id: &str,
        persist_mode: PersistMode,
        update: F,
    ) -> ApiResult<()>
    where
        F: FnOnce(&mut DownloadTask),
    {
        {
            let mut state = self.lock_state()?;
            if let Some(task) = state.tasks.iter_mut().find(|task| task.task_id == task_id) {
                update(task);
                if task.status != DownloadTaskStatus::Completed {
                    task.updated_at = current_timestamp();
                }
            }
            bump_revision(&mut state);
        }

        match persist_mode {
            PersistMode::Force => self.persist_latest_tasks().await,
            PersistMode::Throttled => self.persist_latest_tasks_if_due().await,
        }
    }

    async fn load_tasks_if_needed(&self) -> ApiResult<()> {
        if self.lock_state()?.loaded {
            return Ok(());
        }

        let mut tasks = load_tasks().await?;
        let recovered = recover_interrupted_tasks(&mut tasks);
        let migrated = migrate_pending_task_output_dirs(&self.app, &mut tasks)?;
        if recovered || migrated {
            persist_tasks(&tasks).await?;
        }

        let mut state = self.lock_state()?;
        if !state.loaded {
            state.tasks = tasks;
            state.loaded = true;
            bump_revision(&mut state);
        }

        Ok(())
    }

    async fn persist_latest_tasks(&self) -> ApiResult<()> {
        let _persist_guard = self.persist_lock.lock().await;

        loop {
            let (revision, tasks) = {
                let state = self.lock_state()?;
                (state.revision, state.tasks.clone())
            };
            persist_tasks(&tasks).await?;

            let mut state = self.lock_state()?;
            state.persisted_revision = revision;
            state.last_persisted_at = Some(Instant::now());
            if state.revision == revision {
                return Ok(());
            }
        }
    }

    async fn persist_latest_tasks_if_due(&self) -> ApiResult<()> {
        let should_persist = {
            let state = self.lock_state()?;
            if state.persisted_revision == state.revision {
                false
            } else {
                state
                    .last_persisted_at
                    .map(|last_persisted_at| {
                        last_persisted_at.elapsed() >= DOWNLOAD_PROGRESS_PERSIST_INTERVAL
                    })
                    .unwrap_or(true)
            }
        };

        if should_persist {
            self.persist_latest_tasks().await?;
        }

        Ok(())
    }

    fn lock_state(&self) -> ApiResult<std::sync::MutexGuard<'_, DownloadState>> {
        self.state
            .lock()
            .map_err(|_| ApiError::new(ApiErrorKind::Cache, "Download state lock poisoned"))
    }
}

fn normalize_chapters(
    chapters: Vec<DownloadChapterRequest>,
) -> ApiResult<Vec<DownloadChapterRequest>> {
    let chapters = chapters
        .into_iter()
        .filter_map(|chapter| {
            let chapter_id = chapter.chapter_id.trim().to_string();
            if chapter_id.is_empty() {
                return None;
            }

            Some(DownloadChapterRequest {
                chapter_id,
                title: if chapter.title.trim().is_empty() {
                    "正文".to_string()
                } else {
                    chapter.title.trim().to_string()
                },
                order: chapter.order,
            })
        })
        .collect::<Vec<_>>();

    if chapters.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "Download needs at least one chapter",
        ));
    }

    Ok(chapters)
}

fn normalize_required(value: &str, field: &str) -> ApiResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            format!("Download needs {field}"),
        ));
    }

    Ok(value.to_string())
}

fn bump_revision(state: &mut DownloadState) {
    state.revision = state.revision.saturating_add(1);
}
