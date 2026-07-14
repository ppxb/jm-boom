use crate::{
    cache::{reader_page_cache_key, CachedReaderPage, ImageCache},
    domain::reader::ChapterManifest,
    endpoint::{request_with_failover, EndpointManager},
    image_work::{ImageWorkBudget, ImageWorkPriority},
    jm::{invalidate_img_host, JmClient, JmError, JmResult},
    keyed_lock::KeyedLock,
    reader::{page_name_from_image, prepare_page_image},
};
use once_cell::sync::Lazy;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::watch;

static PAGE_MATERIALIZE_LOCKS: Lazy<KeyedLock> = Lazy::new(KeyedLock::new);

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

type PageSourceFuture<'a, T> = Pin<Box<dyn Future<Output = JmResult<T>> + Send + 'a>>;

trait PageSource: Send + Sync {
    fn get_chapter<'a>(&'a self, chapter_id: &'a str) -> PageSourceFuture<'a, ChapterManifest>;

    fn download_page_image<'a>(
        &'a self,
        chapter_id: &'a str,
        image_path: &'a str,
    ) -> PageSourceFuture<'a, Vec<u8>>;
}

struct JmPageSource {
    jm: Arc<JmClient>,
    endpoints: Arc<EndpointManager>,
}

pub struct PageMaterializer {
    source: Arc<dyn PageSource>,
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
        Self::with_source(Arc::new(JmPageSource { jm, endpoints }), cache, image_work)
    }

    fn with_source(
        source: Arc<dyn PageSource>,
        cache: Arc<ImageCache>,
        image_work: ImageWorkBudget,
    ) -> Self {
        Self {
            source,
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
        let _guard = match request.cancelled.as_mut() {
            Some(cancelled) => {
                tokio::select! {
                    guard = PAGE_MATERIALIZE_LOCKS.lock(cache_key) => guard,
                    _ = wait_for_cancel(cancelled) => return Err(PageMaterializeError::Cancelled),
                }
            }
            None => PAGE_MATERIALIZE_LOCKS.lock(cache_key).await,
        };
        self.materialize_with_lock(cache_key, request).await
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
                let chapter = self.source.get_chapter(request.chapter_id).await?;
                owned_image_path = chapter
                    .images
                    .get(request.page)
                    .cloned()
                    .ok_or(PageMaterializeError::PageNotFound)?;
                &owned_image_path
            }
        };

        let image_data = self
            .source
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
}

impl PageSource for JmPageSource {
    fn get_chapter<'a>(&'a self, chapter_id: &'a str) -> PageSourceFuture<'a, ChapterManifest> {
        Box::pin(async move {
            let request_chapter_id = chapter_id.to_string();
            let (_, chapter) =
                request_with_failover(&self.jm, &self.endpoints, move |client, endpoint| {
                    let chapter_id = request_chapter_id.clone();
                    Box::pin(async move { client.get_chapter(endpoint, &chapter_id).await })
                })
                .await?;
            Ok(chapter)
        })
    }

    fn download_page_image<'a>(
        &'a self,
        chapter_id: &'a str,
        image_path: &'a str,
    ) -> PageSourceFuture<'a, Vec<u8>> {
        Box::pin(async move {
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
        })
    }
}

async fn wait_for_cancel(cancelled: &mut watch::Receiver<bool>) {
    if *cancelled.borrow() {
        return;
    }
    let _ = cancelled.changed().await;
}

#[cfg(test)]
mod tests {
    use super::{
        PageMaterializeError, PageMaterializeRequest, PageMaterializer, PageSource,
        PageSourceFuture,
    };
    use crate::{
        cache::ImageCache,
        domain::reader::ChapterManifest,
        image_work::{ImageWorkBudget, ImageWorkPriority},
        jm::JmError,
        reader::PageImageFormat,
    };
    use sqlx::sqlite::SqlitePoolOptions;
    use std::{
        io::Cursor,
        path::{Path, PathBuf},
        sync::{
            atomic::{AtomicU64, AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    };
    use tokio::{fs, sync::watch, task::JoinSet, time::timeout};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    struct TestPageSource {
        data: Vec<u8>,
        delay: Duration,
        remaining_failures: AtomicUsize,
        download_count: AtomicUsize,
        active_downloads: AtomicUsize,
        max_active_downloads: AtomicUsize,
    }

    struct ActiveDownload<'a> {
        counter: &'a AtomicUsize,
    }

    impl TestPageSource {
        fn new(data: Vec<u8>, delay: Duration, failures: usize) -> Self {
            Self {
                data,
                delay,
                remaining_failures: AtomicUsize::new(failures),
                download_count: AtomicUsize::new(0),
                active_downloads: AtomicUsize::new(0),
                max_active_downloads: AtomicUsize::new(0),
            }
        }

        fn download_count(&self) -> usize {
            self.download_count.load(Ordering::SeqCst)
        }

        fn active_downloads(&self) -> usize {
            self.active_downloads.load(Ordering::SeqCst)
        }

        fn max_active_downloads(&self) -> usize {
            self.max_active_downloads.load(Ordering::SeqCst)
        }
    }

    impl Drop for ActiveDownload<'_> {
        fn drop(&mut self) {
            self.counter.fetch_sub(1, Ordering::SeqCst);
        }
    }

    impl PageSource for TestPageSource {
        fn get_chapter<'a>(&'a self, chapter_id: &'a str) -> PageSourceFuture<'a, ChapterManifest> {
            Box::pin(async move {
                Ok(ChapterManifest {
                    id: chapter_id.to_string(),
                    images: vec!["001.png".into(), "002.png".into()],
                })
            })
        }

        fn download_page_image<'a>(
            &'a self,
            _chapter_id: &'a str,
            _image_path: &'a str,
        ) -> PageSourceFuture<'a, Vec<u8>> {
            Box::pin(async move {
                self.download_count.fetch_add(1, Ordering::SeqCst);
                let active = self.active_downloads.fetch_add(1, Ordering::SeqCst) + 1;
                self.max_active_downloads
                    .fetch_max(active, Ordering::SeqCst);
                let _active = ActiveDownload {
                    counter: &self.active_downloads,
                };

                if !self.delay.is_zero() {
                    tokio::time::sleep(self.delay).await;
                }
                let should_fail = self
                    .remaining_failures
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
                        (remaining > 0).then(|| remaining - 1)
                    })
                    .is_ok();
                if should_fail {
                    return Err(JmError::Network("test failure".into()));
                }
                Ok(self.data.clone())
            })
        }
    }

    #[tokio::test]
    async fn materializes_the_same_page_once_for_concurrent_requests() {
        let data = png_data();
        let source = Arc::new(TestPageSource::new(
            data.clone(),
            Duration::from_millis(50),
            0,
        ));
        let (materializer, cache, root) = test_materializer("same-page", source.clone()).await;
        let mut jobs = JoinSet::new();

        for _ in 0..8 {
            let materializer = materializer.clone();
            jobs.spawn(async move {
                materializer
                    .materialize(test_request("chapter-same", 0, "001.png"))
                    .await
            });
        }

        while let Some(result) = jobs.join_next().await {
            let page = result
                .expect("same-page task should finish")
                .expect("same-page materialization should succeed");
            assert_eq!(page.format, PageImageFormat::Png);
            assert_eq!(page.data, data);
        }

        assert_eq!(source.download_count(), 1);
        let stats = cache.stats().await.expect("load cache stats");
        assert_eq!(stats.entry_count, 1);
        assert_eq!(stats.size_bytes, i64::try_from(data.len()).unwrap());
        assert!(!cache_files(&root.join("cache"))
            .iter()
            .any(|path| path.to_string_lossy().contains(".tmp-")));

        cleanup(materializer, cache, root).await;
    }

    #[tokio::test]
    async fn materializes_different_pages_concurrently() {
        let data = png_data();
        let source = Arc::new(TestPageSource::new(data, Duration::from_millis(50), 0));
        let (materializer, cache, root) =
            test_materializer("different-pages", source.clone()).await;
        let first_materializer = materializer.clone();
        let first = tokio::spawn(async move {
            first_materializer
                .materialize(test_request("chapter-parallel", 0, "001.png"))
                .await
        });
        let second_materializer = materializer.clone();
        let second = tokio::spawn(async move {
            second_materializer
                .materialize(test_request("chapter-parallel", 1, "002.png"))
                .await
        });

        first
            .await
            .expect("first page task should finish")
            .expect("first page should materialize");
        second
            .await
            .expect("second page task should finish")
            .expect("second page should materialize");

        assert_eq!(source.download_count(), 2);
        assert!(source.max_active_downloads() >= 2);
        assert_eq!(
            cache.stats().await.expect("load cache stats").entry_count,
            2
        );

        cleanup(materializer, cache, root).await;
    }

    #[tokio::test]
    async fn cancelling_a_waiter_does_not_cancel_the_owner() {
        let source = Arc::new(TestPageSource::new(
            png_data(),
            Duration::from_millis(200),
            0,
        ));
        let (materializer, cache, root) = test_materializer("cancel-waiter", source.clone()).await;
        let owner_materializer = materializer.clone();
        let owner = tokio::spawn(async move {
            owner_materializer
                .materialize(test_request("chapter-cancel", 0, "001.png"))
                .await
        });
        wait_for_active_download(&source).await;

        let (cancel, cancelled) = watch::channel(false);
        let waiter_materializer = materializer.clone();
        let waiter = tokio::spawn(async move {
            let mut request = test_request("chapter-cancel", 0, "001.png");
            request.cancelled = Some(cancelled);
            waiter_materializer.materialize(request).await
        });
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }
        cancel.send(true).expect("cancel waiting request");

        let waiter_result = timeout(Duration::from_secs(1), waiter)
            .await
            .expect("cancelled waiter timed out")
            .expect("cancelled waiter task failed");
        let waiter_error = match waiter_result {
            Ok(_) => panic!("waiter should be cancelled"),
            Err(error) => error,
        };
        assert!(matches!(waiter_error, PageMaterializeError::Cancelled));
        owner
            .await
            .expect("owner task should finish")
            .expect("owner should materialize the page");
        assert_eq!(source.download_count(), 1);
        assert_eq!(
            cache.stats().await.expect("load cache stats").entry_count,
            1
        );

        cleanup(materializer, cache, root).await;
    }

    #[tokio::test]
    async fn retries_after_the_first_materialization_fails() {
        let source = Arc::new(TestPageSource::new(png_data(), Duration::ZERO, 1));
        let (materializer, cache, root) = test_materializer("retry", source.clone()).await;

        let first_result = materializer
            .materialize(test_request("chapter-retry", 0, "001.png"))
            .await;
        let first = match first_result {
            Ok(_) => panic!("first materialization should fail"),
            Err(error) => error,
        };
        assert!(matches!(
            first,
            PageMaterializeError::Upstream(JmError::Network(_))
        ));
        materializer
            .materialize(test_request("chapter-retry", 0, "001.png"))
            .await
            .expect("second materialization should retry");

        assert_eq!(source.download_count(), 2);
        assert_eq!(
            cache.stats().await.expect("load cache stats").entry_count,
            1
        );
        assert!(!cache_files(&root.join("cache"))
            .iter()
            .any(|path| path.to_string_lossy().contains(".tmp-")));

        cleanup(materializer, cache, root).await;
    }

    fn test_request(
        chapter_id: &'static str,
        page: usize,
        image_path: &'static str,
    ) -> PageMaterializeRequest<'static> {
        PageMaterializeRequest {
            chapter_id,
            page,
            comic_id: 100_000,
            image_path: Some(image_path),
            priority: ImageWorkPriority::Foreground,
            cancelled: None,
        }
    }

    async fn test_materializer(
        name: &str,
        source: Arc<TestPageSource>,
    ) -> (Arc<PageMaterializer>, Arc<ImageCache>, PathBuf) {
        let root = test_root(name);
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect in-memory sqlite");
        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .expect("run migrations");
        let cache = Arc::new(
            ImageCache::new_for_test(db, 1024 * 1024, root.join("cache"))
                .await
                .expect("create test image cache"),
        );
        let materializer = Arc::new(PageMaterializer::with_source(
            source,
            cache.clone(),
            ImageWorkBudget::new(),
        ));
        (materializer, cache, root)
    }

    async fn wait_for_active_download(source: &TestPageSource) {
        timeout(Duration::from_secs(1), async {
            while source.active_downloads() == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("page source did not start downloading");
    }

    async fn cleanup(materializer: Arc<PageMaterializer>, cache: Arc<ImageCache>, root: PathBuf) {
        drop(materializer);
        drop(cache);
        fs::remove_dir_all(root)
            .await
            .expect("remove materializer test root");
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

    fn cache_files(root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let mut directories = vec![root.to_path_buf()];
        while let Some(directory) = directories.pop() {
            for entry in std::fs::read_dir(directory).expect("read cache test directory") {
                let entry = entry.expect("read cache test entry");
                if entry.file_type().expect("read cache entry type").is_dir() {
                    directories.push(entry.path());
                } else {
                    files.push(entry.path());
                }
            }
        }
        files
    }

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "jm-boom-page-materializer-{name}-{}-{}",
            std::process::id(),
            TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }
}
