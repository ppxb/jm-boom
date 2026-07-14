use crate::{
    application::ComicService,
    cache::CachedReaderPage,
    domain::reader::ChapterManifest,
    download::{DownloadError, DownloadManager},
    image_work::ImageWorkPriority,
    jm::JmResult,
    page_materializer::{PageMaterializeError, PageMaterializeRequest, PageMaterializer},
};
use std::sync::Arc;
use tokio::sync::watch;

#[derive(Clone)]
pub struct ReaderService {
    comics: Arc<ComicService>,
    page_materializer: Arc<PageMaterializer>,
    downloads: Arc<DownloadManager>,
}

impl ReaderService {
    pub fn new(
        comics: Arc<ComicService>,
        page_materializer: Arc<PageMaterializer>,
        downloads: Arc<DownloadManager>,
    ) -> Self {
        Self {
            comics,
            page_materializer,
            downloads,
        }
    }

    pub async fn chapter(&self, chapter_id: &str) -> JmResult<ChapterManifest> {
        self.comics.get_chapter(chapter_id.to_string()).await
    }

    pub async fn cached_page(
        &self,
        chapter_id: &str,
        page: usize,
    ) -> anyhow::Result<Option<CachedReaderPage>> {
        self.page_materializer.cached_page(chapter_id, page).await
    }

    pub async fn offline_manifest(
        &self,
        chapter_id: &str,
    ) -> Result<Option<ChapterManifest>, DownloadError> {
        Ok(self
            .downloads
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
        self.downloads.offline_page(chapter_id, page).await
    }

    pub async fn materialize(
        &self,
        chapter_id: &str,
        page: usize,
        comic_id: u32,
        priority: ImageWorkPriority,
    ) -> Result<CachedReaderPage, PageMaterializeError> {
        self.page_materializer
            .materialize_after_cache_miss(PageMaterializeRequest {
                chapter_id,
                page,
                comic_id,
                image_path: None,
                priority,
                cancelled: None::<watch::Receiver<bool>>,
            })
            .await
    }
}
