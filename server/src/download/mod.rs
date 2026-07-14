mod error;
mod manager;
mod model;
mod repository;
mod scheduler;
mod state;
mod storage;
mod worker;

pub use error::{DownloadError, DownloadResult};
pub(crate) use manager::DownloadManager;
pub use model::{DownloadTaskList, DownloadedChapterList, EnqueueDownload};

pub(super) const DOWNLOAD_TASK_CONCURRENCY: usize = 2;
pub(super) const DOWNLOAD_PAGE_CONCURRENCY_PER_TASK: usize = 5;
