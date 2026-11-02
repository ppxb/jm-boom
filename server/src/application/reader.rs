use crate::{
    application::{ComicService, DownloadService},
    cache::{page_image_id, CachedReaderPage},
    domain::reader::ChapterManifest,
    download::DownloadError,
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
    downloads: Arc<DownloadService>,
}

impl ReaderService {
    pub fn new(
        comics: Arc<ComicService>,
        page_materializer: Arc<PageMaterializer>,
        downloads: Arc<DownloadService>,
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

    pub async fn page_dimensions(
        &self,
        chapter_id: &str,
    ) -> anyhow::Result<std::collections::HashMap<String, crate::reader::ImageDimensions>> {
        self.page_materializer.page_dimensions(chapter_id).await
    }

    pub async fn cached_page(
        &self,
        chapter_id: &str,
        image_id: &str,
    ) -> anyhow::Result<Option<CachedReaderPage>> {
        self.page_materializer
            .cached_page(chapter_id, image_id)
            .await
    }

    pub async fn offline_manifest(
        &self,
        chapter_id: &str,
    ) -> Result<Option<ChapterManifest>, DownloadError> {
        self.downloads.offline_manifest(chapter_id).await
    }

    pub async fn offline_page(
        &self,
        chapter_id: &str,
        image_id: &str,
    ) -> Result<Option<CachedReaderPage>, DownloadError> {
        let image = self.downloads.offline_page(chapter_id, image_id).await?;
        if let Some(image) = &image {
            self.page_materializer
                .remember_page_dimensions(chapter_id, image_id, image)
                .await;
        }
        Ok(image)
    }

    pub async fn materialize(
        &self,
        chapter_id: &str,
        image_id: &str,
        comic_id: u32,
        priority: ImageWorkPriority,
    ) -> Result<CachedReaderPage, PageMaterializeError> {
        let chapter = self.chapter(chapter_id).await?;
        let (page, image_path) = chapter
            .images
            .iter()
            .enumerate()
            .find(|(_, path)| page_image_id(path) == image_id)
            .ok_or(PageMaterializeError::PageNotFound)?;
        self.page_materializer
            .materialize(PageMaterializeRequest {
                chapter_id,
                page,
                comic_id,
                image_path,
                priority,
                cancelled: None::<watch::Receiver<bool>>,
            })
            .await
    }
}
