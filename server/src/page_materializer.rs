use crate::{
    cache::{reader_page_cache_key, CachedReaderPage, ImageCache},
    endpoint::{request_with_failover, EndpointManager},
    image_work::{ImageWorkBudget, ImageWorkPriority},
    jm::{invalidate_img_host, JmClient, JmError},
    reader::{page_name_from_image, prepare_page_image},
};
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{watch, Mutex};

static PAGE_MATERIALIZE_LOCKS: Lazy<Mutex<HashMap<String, Arc<Mutex<()>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub struct PageMaterializeRequest<'a> {
    pub chapter_id: &'a str,
    pub page: usize,
    pub comic_id: u32,
    pub image_path: Option<&'a str>,
    pub priority: ImageWorkPriority,
    pub cancelled: Option<watch::Receiver<bool>>,
}

#[derive(Debug, thiserror::Error)]
pub enum PageMaterializeError {
    #[error(transparent)]
    Upstream(#[from] JmError),
    #[error("Page index out of range")]
    PageNotFound,
    #[error("Page materialization was cancelled")]
    Cancelled,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

pub struct PageMaterializer {
    jm: Arc<JmClient>,
    endpoints: Arc<EndpointManager>,
    cache: Arc<ImageCache>,
    image_work: ImageWorkBudget,
}

impl PageMaterializer {
    pub fn new(
        jm: Arc<JmClient>,
        endpoints: Arc<EndpointManager>,
        cache: Arc<ImageCache>,
        image_work: ImageWorkBudget,
    ) -> Self {
        Self {
            jm,
            endpoints,
            cache,
            image_work,
        }
    }

    pub async fn cached_page(
        &self,
        chapter_id: &str,
        page: usize,
    ) -> anyhow::Result<Option<CachedReaderPage>> {
        self.cache
            .get_reader_page(&reader_page_cache_key(chapter_id, page))
            .await
    }

    pub async fn materialize(
        &self,
        request: PageMaterializeRequest<'_>,
    ) -> Result<CachedReaderPage, PageMaterializeError> {
        let cache_key = reader_page_cache_key(request.chapter_id, request.page);
        if let Some(cached) = self.cache.get_reader_page(&cache_key).await? {
            return Ok(cached);
        }

        self.materialize_after_cache_miss_with_key(&cache_key, request)
            .await
    }

    /// Continue materialization after the caller checked `cached_page` for the same page.
    pub async fn materialize_after_cache_miss(
        &self,
        request: PageMaterializeRequest<'_>,
    ) -> Result<CachedReaderPage, PageMaterializeError> {
        let cache_key = reader_page_cache_key(request.chapter_id, request.page);
        self.materialize_after_cache_miss_with_key(&cache_key, request)
            .await
    }

    async fn materialize_after_cache_miss_with_key(
        &self,
        cache_key: &str,
        mut request: PageMaterializeRequest<'_>,
    ) -> Result<CachedReaderPage, PageMaterializeError> {
        let materialize_lock = page_materialize_lock(cache_key).await;
        let guard = match request.cancelled.as_mut() {
            Some(cancelled) => {
                tokio::select! {
                    guard = materialize_lock.lock() => guard,
                    _ = wait_for_cancel(cancelled) => {
                        remove_page_materialize_lock(cache_key, &materialize_lock).await;
                        return Err(PageMaterializeError::Cancelled);
                    }
                }
            }
            None => materialize_lock.lock().await,
        };
        let result = self.materialize_with_lock(cache_key, request).await;
        drop(guard);
        remove_page_materialize_lock(cache_key, &materialize_lock).await;
        result
    }

    async fn materialize_with_lock(
        &self,
        cache_key: &str,
        mut request: PageMaterializeRequest<'_>,
    ) -> Result<CachedReaderPage, PageMaterializeError> {
        if let Some(cached) = self.cache.get_reader_page(cache_key).await? {
            return Ok(cached);
        }

        let _work_permit = match request.cancelled.as_mut() {
            Some(cancelled) => {
                tokio::select! {
                    permit = self.image_work.acquire(request.priority) => permit,
                    _ = wait_for_cancel(cancelled) => return Err(PageMaterializeError::Cancelled),
                }
            }
            None => self.image_work.acquire(request.priority).await,
        };
        if request
            .cancelled
            .as_ref()
            .is_some_and(|cancelled| *cancelled.borrow())
        {
            return Err(PageMaterializeError::Cancelled);
        }

        let owned_image_path;
        let image_path = match request.image_path {
            Some(image_path) => image_path,
            None => {
                let request_chapter_id = request.chapter_id.to_string();
                let (_, chapter) =
                    request_with_failover(&self.jm, &self.endpoints, move |client, endpoint| {
                        let chapter_id = request_chapter_id.clone();
                        Box::pin(async move { client.get_chapter(endpoint, &chapter_id).await })
                    })
                    .await?;
                owned_image_path = chapter
                    .images
                    .get(request.page)
                    .cloned()
                    .ok_or(PageMaterializeError::PageNotFound)?;
                &owned_image_path
            }
        };

        let image_data = self
            .download_page_image(request.chapter_id, image_path)
            .await?;
        let page_name = page_name_from_image(image_path);
        let prepared = prepare_page_image(image_data, request.comic_id, page_name).await?;
        tracing::debug!(
            chapter_id = request.chapter_id,
            page = request.page,
            bytes = prepared.data.len(),
            decoded = prepared.decoded,
            "共享阅读页物化完成"
        );
        self.cache
            .put_reader_page(cache_key, prepared.format, &prepared.data)
            .await?;

        Ok(CachedReaderPage {
            data: prepared.data,
            format: prepared.format,
        })
    }

    async fn download_page_image(
        &self,
        chapter_id: &str,
        image_path: &str,
    ) -> Result<Vec<u8>, JmError> {
        let (endpoint, img_host) =
            request_with_failover(&self.jm, &self.endpoints, |client, endpoint| {
                Box::pin(client.get_img_host(endpoint))
            })
            .await?;
        let image_url = format!("{img_host}/media/photos/{chapter_id}/{image_path}");

        match self.jm.download_image(&image_url).await {
            Ok(data) => Ok(data),
            Err(error) if error.is_retryable() => {
                invalidate_img_host(&endpoint).await;
                let (_, refreshed_host) =
                    request_with_failover(&self.jm, &self.endpoints, |client, endpoint| {
                        Box::pin(client.get_img_host(endpoint))
                    })
                    .await?;
                let refreshed_url =
                    format!("{refreshed_host}/media/photos/{chapter_id}/{image_path}");
                self.jm.download_image(&refreshed_url).await
            }
            Err(error) => Err(error),
        }
    }
}

async fn page_materialize_lock(cache_key: &str) -> Arc<Mutex<()>> {
    let mut locks = PAGE_MATERIALIZE_LOCKS.lock().await;
    locks.retain(|_, lock| Arc::strong_count(lock) > 1);
    locks
        .entry(cache_key.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

async fn remove_page_materialize_lock(cache_key: &str, materialize_lock: &Arc<Mutex<()>>) {
    let mut locks = PAGE_MATERIALIZE_LOCKS.lock().await;
    let should_remove = locks.get(cache_key).is_some_and(|current| {
        Arc::ptr_eq(current, materialize_lock) && Arc::strong_count(materialize_lock) == 2
    });

    if should_remove {
        locks.remove(cache_key);
    }
}

async fn wait_for_cancel(cancelled: &mut watch::Receiver<bool>) {
    if *cancelled.borrow() {
        return;
    }
    let _ = cancelled.changed().await;
}
