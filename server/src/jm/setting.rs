use super::{auth::JmAuth, crypto, error::JmError, JmResult};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

/// Cached img_host with expiration
#[derive(Clone)]
struct CachedImgHost {
    host: String,
    expires_at: SystemTime,
}

static IMG_HOST_CACHE: Lazy<RwLock<Option<CachedImgHost>>> = Lazy::new(|| RwLock::new(None));

const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

#[derive(Debug, Deserialize)]
struct SettingResponse {
    #[serde(default)]
    code: i32,
    data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct SettingData {
    #[serde(rename = "app_img_shunt")]
    img_host: String,
}

impl super::client::JmClient {
    /// Get image host from remote settings (with cache)
    pub async fn get_img_host(&self, endpoint: &str) -> JmResult<String> {
        // Try cache first
        if let Some(cached) = try_get_cached_host() {
            tracing::debug!("Using cached img_host: {}", cached);
            return Ok(cached);
        }

        // Cache miss, fetch from API
        tracing::info!("Fetching img_host from API");
        let host = self.fetch_img_host(endpoint).await?;

        // Update cache
        update_cache(&host);

        Ok(host)
    }

    /// Fetch img_host from remote API
    async fn fetch_img_host(&self, endpoint: &str) -> JmResult<String> {
        let auth = JmAuth::new();
        let url = format!("{endpoint}/setting");

        let response = self
            .client
            .get(&url)
            .header("token", &auth.token)
            .header("tokenparam", &auth.tokenparam)
            .query(&[("app_img_shunt", "1"), ("t", &auth.ts)])
            .send()
            .await
            .map_err(|e| JmError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(JmError::Http(format!("HTTP {}", response.status())));
        }

        let body = response
            .text()
            .await
            .map_err(|e| JmError::Network(e.to_string()))?;

        let envelope: SettingResponse = serde_json::from_str(&body)
            .map_err(|e| JmError::Decode(format!("Invalid setting response: {e}")))?;

        if envelope.code != 200 {
            return Err(JmError::Api(format!("API code {}", envelope.code)));
        }

        let data_value = envelope
            .data
            .ok_or_else(|| JmError::Decode("Missing data".into()))?;

        let data_str = data_value
            .as_str()
            .ok_or_else(|| JmError::Decode("Data is not string".into()))?;

        let decrypted = crypto::decrypt_data(data_str, &auth.ts)?;

        let setting: SettingData = serde_json::from_str(&decrypted)
            .map_err(|e| JmError::Decode(format!("Invalid setting data: {e}")))?;

        Ok(setting.img_host.trim_end_matches('/').to_string())
    }

    /// Download image from JM
    pub async fn download_image(&self, url: &str) -> JmResult<Vec<u8>> {
        let response = self
            .client
            .get(url)
            .header("referer", "https://www.jmapinode1.cc/")
            .send()
            .await
            .map_err(|e| JmError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(JmError::Http(format!("HTTP {}", response.status())));
        }

        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| JmError::Network(e.to_string()))
    }
}

/// Try to get cached img_host if not expired
fn try_get_cached_host() -> Option<String> {
    let cache = IMG_HOST_CACHE.read().ok()?;
    let cached = cache.as_ref()?;

    if SystemTime::now() < cached.expires_at {
        Some(cached.host.clone())
    } else {
        None
    }
}

/// Update cache with new img_host
fn update_cache(host: &str) {
    if let Ok(mut cache) = IMG_HOST_CACHE.write() {
        *cache = Some(CachedImgHost {
            host: host.to_string(),
            expires_at: SystemTime::now() + CACHE_TTL,
        });
        tracing::info!("Cached img_host for {} seconds", CACHE_TTL.as_secs());
    }
}

/// Clear the img_host cache (for testing or manual refresh)
#[allow(dead_code)]
pub fn clear_img_host_cache() {
    if let Ok(mut cache) = IMG_HOST_CACHE.write() {
        *cache = None;
        tracing::info!("Cleared img_host cache");
    }
}
