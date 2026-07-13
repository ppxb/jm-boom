use crate::cache::cover_cache_key;
use crate::endpoint::request_with_failover;
use crate::http_error::HttpError;
use crate::jm::invalidate_img_host;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, Semaphore};

const COVER_FETCH_CONCURRENCY: usize = 6;

static COVER_FETCH_LOCKS: Lazy<Mutex<HashMap<String, Arc<Mutex<()>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static COVER_FETCH_SEMAPHORE: Lazy<Arc<Semaphore>> =
    Lazy::new(|| Arc::new(Semaphore::new(COVER_FETCH_CONCURRENCY)));

pub async fn get_cover(
    State(app): State<AppState>,
    Path(comic_id): Path<String>,
) -> Result<Response, CoverError> {
    validate_comic_id(&comic_id)?;
    let cache_key = cover_cache_key(&comic_id);

    if let Some(cached) = app
        .cache
        .get_cover(&cache_key)
        .await
        .map_err(CoverError::Cache)?
    {
        return Ok(cover_response(cached));
    }

    let fetch_lock = cover_fetch_lock(&cache_key).await;
    let guard = fetch_lock.lock().await;
    let result = materialize_cover(&app, &comic_id, &cache_key).await;
    drop(guard);
    remove_cover_fetch_lock(&cache_key, &fetch_lock).await;

    result.map(cover_response)
}

async fn materialize_cover(
    app: &AppState,
    comic_id: &str,
    cache_key: &str,
) -> Result<Vec<u8>, CoverError> {
    if let Some(cached) = app
        .cache
        .get_cover(cache_key)
        .await
        .map_err(CoverError::Cache)?
    {
        return Ok(cached);
    }

    let _permit = COVER_FETCH_SEMAPHORE
        .clone()
        .acquire_owned()
        .await
        .map_err(|error| CoverError::Internal(error.to_string()))?;
    let data = download_cover(app, comic_id).await?;

    app.cache
        .put_cover(cache_key, &data)
        .await
        .map_err(CoverError::Cache)?;

    Ok(data)
}

async fn download_cover(app: &AppState, comic_id: &str) -> crate::jm::JmResult<Vec<u8>> {
    let (endpoint, img_host) =
        request_with_failover(&app.jm, &app.endpoints, |client, endpoint| {
            Box::pin(client.get_img_host(endpoint))
        })
        .await?;
    let cover_url = format!("{img_host}/media/albums/{comic_id}_3x4.jpg");

    match app.jm.download_image(&cover_url).await {
        Ok(data) => Ok(data),
        Err(_) => {
            invalidate_img_host(&endpoint).await;
            let (_, refreshed_host) =
                request_with_failover(&app.jm, &app.endpoints, |client, endpoint| {
                    Box::pin(client.get_img_host(endpoint))
                })
                .await?;
            let refreshed_url = format!("{refreshed_host}/media/albums/{comic_id}_3x4.jpg");
            app.jm.download_image(&refreshed_url).await
        }
    }
}

async fn cover_fetch_lock(cache_key: &str) -> Arc<Mutex<()>> {
    let mut locks = COVER_FETCH_LOCKS.lock().await;
    locks.retain(|_, lock| Arc::strong_count(lock) > 1);
    locks
        .entry(cache_key.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

async fn remove_cover_fetch_lock(cache_key: &str, fetch_lock: &Arc<Mutex<()>>) {
    let mut locks = COVER_FETCH_LOCKS.lock().await;
    let should_remove = locks.get(cache_key).is_some_and(|current| {
        Arc::ptr_eq(current, fetch_lock) && Arc::strong_count(fetch_lock) == 2
    });

    if should_remove {
        locks.remove(cache_key);
    }
}

fn validate_comic_id(comic_id: &str) -> Result<(), CoverError> {
    if comic_id.is_empty() || !comic_id.chars().all(|character| character.is_ascii_digit()) {
        return Err(CoverError::BadRequest("Comic id must be numeric".into()));
    }

    Ok(())
}

fn cover_response(body: Vec<u8>) -> Response {
    let content_type = match image::guess_format(&body) {
        Ok(image::ImageFormat::Gif) => "image/gif",
        Ok(image::ImageFormat::Png) => "image/png",
        Ok(image::ImageFormat::WebP) => "image/webp",
        _ => "image/jpeg",
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (
                header::CACHE_CONTROL,
                "public, max-age=604800, stale-while-revalidate=86400",
            ),
        ],
        body,
    )
        .into_response()
}

#[derive(Debug)]
pub enum CoverError {
    Jm(crate::jm::JmError),
    BadRequest(String),
    Cache(anyhow::Error),
    Internal(String),
}

impl From<crate::jm::JmError> for CoverError {
    fn from(error: crate::jm::JmError) -> Self {
        Self::Jm(error)
    }
}

impl IntoResponse for CoverError {
    fn into_response(self) -> Response {
        let (status, message, retryable) = match self {
            Self::Jm(error) => {
                let retryable = error.is_retryable();
                (StatusCode::BAD_GATEWAY, error.to_string(), retryable)
            }
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message, false),
            Self::Cache(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string(), false),
            Self::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message, false),
        };

        HttpError::new(status, message, retryable).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::validate_comic_id;

    #[test]
    fn validates_numeric_comic_ids() {
        assert!(validate_comic_id("1444583").is_ok());
        assert!(validate_comic_id("").is_err());
        assert!(validate_comic_id("../1444583").is_err());
    }
}
