use crate::{
    cache::{cover_cache_key, ImageCache},
    endpoint::{request_with_failover, EndpointManager},
    jm::{invalidate_img_host, JmClient, JmError},
    keyed_lock::KeyedLock,
};
use anyhow::Error as AnyhowError;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Semaphore;

const COVER_FETCH_CONCURRENCY: usize = 6;

static COVER_FETCH_LOCKS: Lazy<KeyedLock> = Lazy::new(KeyedLock::new);
static COVER_FETCH_SEMAPHORE: Lazy<Arc<Semaphore>> =
    Lazy::new(|| Arc::new(Semaphore::new(COVER_FETCH_CONCURRENCY)));

#[derive(Clone)]
pub struct CoverService {
    cache: Arc<ImageCache>,
    jm: Arc<JmClient>,
    endpoints: Arc<EndpointManager>,
}

#[derive(Debug)]
pub enum CoverServiceError {
    Jm(JmError),
    Cache(AnyhowError),
    Internal(String),
}

impl CoverService {
    pub fn new(cache: Arc<ImageCache>, jm: Arc<JmClient>, endpoints: Arc<EndpointManager>) -> Self {
        Self {
            cache,
            jm,
            endpoints,
        }
    }

    pub async fn get_cover(&self, comic_id: &str) -> Result<Vec<u8>, CoverServiceError> {
        let cache_key = cover_cache_key(comic_id);
        if let Some(cached) = self
            .cache
            .get_cover(&cache_key)
            .await
            .map_err(CoverServiceError::Cache)?
        {
            return Ok(cached);
        }

        let _guard = COVER_FETCH_LOCKS.lock(&cache_key).await;
        if let Some(cached) = self
            .cache
            .get_cover(&cache_key)
            .await
            .map_err(CoverServiceError::Cache)?
        {
            return Ok(cached);
        }

        let _permit = COVER_FETCH_SEMAPHORE
            .clone()
            .acquire_owned()
            .await
            .map_err(|error| CoverServiceError::Internal(error.to_string()))?;
        let data = self.download_cover(comic_id).await?;
        self.cache
            .put_cover(&cache_key, &data)
            .await
            .map_err(CoverServiceError::Cache)?;
        Ok(data)
    }

    async fn download_cover(&self, comic_id: &str) -> Result<Vec<u8>, CoverServiceError> {
        let (endpoint, img_host) =
            request_with_failover(&self.jm, &self.endpoints, |client, endpoint| {
                Box::pin(client.get_img_host(endpoint))
            })
            .await
            .map_err(CoverServiceError::Jm)?;
        let cover_url = format!("{img_host}/media/albums/{comic_id}_3x4.jpg");

        match self.jm.download_image(&cover_url).await {
            Ok(data) => Ok(data),
            Err(error) if error.is_retryable() => {
                invalidate_img_host(&endpoint).await;
                let (_, refreshed_host) =
                    request_with_failover(&self.jm, &self.endpoints, |client, endpoint| {
                        Box::pin(client.get_img_host(endpoint))
                    })
                    .await
                    .map_err(CoverServiceError::Jm)?;
                let refreshed_url = format!("{refreshed_host}/media/albums/{comic_id}_3x4.jpg");
                self.jm
                    .download_image(&refreshed_url)
                    .await
                    .map_err(CoverServiceError::Jm)
            }
            Err(error) => Err(CoverServiceError::Jm(error)),
        }
    }
}
