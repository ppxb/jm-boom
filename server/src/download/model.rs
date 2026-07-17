use serde::{Deserialize, Serialize};

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

impl std::fmt::Display for DownloadStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        };
        formatter.write_str(value)
    }
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
pub(crate) struct OfflineChapterManifest {
    pub task_id: String,
    pub album_id: String,
    pub chapter_id: String,
    pub title: String,
    pub images: Vec<String>,
    pub updated_at: i64,
}

pub(super) fn mark_task_updated(task: &mut DownloadTask) {
    task.updated_at = chrono::Utc::now()
        .timestamp_millis()
        .max(task.updated_at.saturating_add(1));
}
