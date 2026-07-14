use crate::{
    cache::CachedReaderPage,
    domain::reader::ChapterManifest,
    download::{
        DownloadError, DownloadManager, DownloadResult, DownloadTaskList, DownloadedChapterList,
        EnqueueDownload,
    },
    endpoint::EndpointManager,
    jm::JmClient,
    page_materializer::PageMaterializer,
};
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone)]
pub struct DownloadService {
    manager: DownloadManager,
}

impl DownloadService {
    pub async fn new(
        db: SqlitePool,
        jm: Arc<JmClient>,
        endpoints: Arc<EndpointManager>,
        page_materializer: Arc<PageMaterializer>,
    ) -> DownloadResult<Self> {
        Ok(Self {
            manager: DownloadManager::new(db, jm, endpoints, page_materializer).await?,
        })
    }

    pub async fn resume_pending(&self) {
        self.manager.resume_pending().await;
    }

    pub async fn enqueue(&self, input: EnqueueDownload) -> DownloadResult<DownloadTaskList> {
        self.manager.enqueue(input).await
    }

    pub async fn list(&self) -> DownloadTaskList {
        self.manager.list().await
    }

    pub async fn downloaded_chapters(&self) -> DownloadResult<DownloadedChapterList> {
        self.manager.downloaded_chapters().await
    }

    pub async fn pause(&self, task_id: &str) -> DownloadResult<DownloadTaskList> {
        self.manager.pause(task_id).await
    }

    pub async fn resume(&self, task_id: &str) -> DownloadResult<DownloadTaskList> {
        self.manager.resume(task_id).await
    }

    pub async fn cancel(&self, task_id: &str) -> DownloadResult<DownloadTaskList> {
        self.manager.cancel(task_id).await
    }

    pub async fn remove(&self, task_id: &str) -> DownloadResult<DownloadTaskList> {
        self.manager.remove(task_id).await
    }

    pub async fn offline_manifest(
        &self,
        chapter_id: &str,
    ) -> Result<Option<ChapterManifest>, DownloadError> {
        Ok(self
            .manager
            .offline_manifest(chapter_id)
            .await?
            .map(|manifest| ChapterManifest {
                id: manifest.chapter_id,
                images: manifest.images,
            }))
    }

    pub async fn offline_page(
        &self,
        chapter_id: &str,
        page: usize,
    ) -> Result<Option<CachedReaderPage>, DownloadError> {
        self.manager.offline_page(chapter_id, page).await
    }
}
