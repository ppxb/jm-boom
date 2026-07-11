use super::{crypto, error::JmError, JmResult, SettingRequestSignature, API_SECRET};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{Mutex, RwLock};

const IMG_HOST_CACHE_TTL: Duration = Duration::from_secs(60 * 60);
const IMG_HOST_CACHE_MAX_ENTRIES: usize = 32;
const SETTING_AES_SEEDS: [&str; 2] = [API_SECRET, "18comicAPPContent"];

#[derive(Clone)]
struct CachedImgHost {
    host: String,
    expires_at: Instant,
}

static IMG_HOST_CACHE: Lazy<RwLock<HashMap<String, CachedImgHost>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
static IMG_HOST_FETCH_LOCKS: Lazy<Mutex<HashMap<String, Arc<Mutex<()>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Deserialize)]
struct SettingResponse {
    #[serde(default)]
    code: i32,
    data: Option<serde_json::Value>,
    #[serde(default, rename = "errorMsg")]
    error_msg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SettingData {
    #[serde(
        default,
        alias = "app_img_shunt",
        deserialize_with = "string_from_scalar"
    )]
    img_host: String,
}

impl super::client::JmClient {
    pub async fn get_img_host(&self, endpoint: &str) -> JmResult<String> {
        if let Some(host) = cached_img_host(endpoint).await {
            return Ok(host);
        }

        let fetch_lock = endpoint_fetch_lock(endpoint).await;
        let guard = fetch_lock.lock().await;
        let result = async {
            if let Some(host) = cached_img_host(endpoint).await {
                return Ok(host);
            }

            let host = self.fetch_img_host(endpoint).await?;
            insert_cached_img_host(endpoint, host.clone()).await;
            tracing::debug!(endpoint, img_host = %host, "cached image host");
            Ok(host)
        }
        .await;
        drop(guard);
        remove_endpoint_fetch_lock(endpoint, &fetch_lock).await;
        result
    }

    async fn fetch_img_host(&self, endpoint: &str) -> JmResult<String> {
        let signature = SettingRequestSignature::current();
        let url = format!("{endpoint}/setting");
        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .header("token", &signature.token)
            .header("tokenparam", &signature.tokenparam)
            .query(&[("app_img_shunt", "1"), ("t", signature.ts.as_str())])
            .send()
            .await
            .map_err(|error| JmError::Network(error.to_string()))?;

        if !response.status().is_success() {
            return Err(JmError::Http(format!("HTTP {}: {url}", response.status())));
        }

        let body = response
            .text()
            .await
            .map_err(|error| JmError::Network(error.to_string()))?;
        let envelope: SettingResponse = serde_json::from_str(&body)
            .map_err(|error| JmError::Decode(format!("Invalid setting response: {error}")))?;

        if envelope.code != 200 {
            return Err(JmError::Api(
                envelope
                    .error_msg
                    .unwrap_or_else(|| format!("API code {}", envelope.code)),
            ));
        }

        let value = envelope.data.ok_or(JmError::MissingData)?;
        let setting = match value {
            serde_json::Value::String(encrypted) => {
                let decrypted = decrypt_setting_data(&encrypted, &signature.ts)?;
                serde_json::from_str::<SettingData>(&decrypted)
                    .map_err(|error| JmError::Decode(format!("Invalid setting data: {error}")))?
            }
            value => serde_json::from_value::<SettingData>(value)
                .map_err(|error| JmError::Decode(format!("Invalid setting data: {error}")))?,
        };

        let host = setting.img_host.trim().trim_end_matches('/');
        if host.is_empty() {
            return Err(JmError::MissingData);
        }
        Ok(host.to_string())
    }

    pub async fn download_image(&self, url: &str) -> JmResult<Vec<u8>> {
        let response = self
            .client
            .get(url)
            .header("referer", "https://18comic.vip/")
            .send()
            .await
            .map_err(|error| JmError::Network(error.to_string()))?;

        if !response.status().is_success() {
            return Err(JmError::Http(format!("HTTP {}: {url}", response.status())));
        }

        response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(|error| JmError::Network(error.to_string()))
    }
}

fn decrypt_setting_data(data: &str, ts: &str) -> JmResult<String> {
    let mut last_error = None;

    for seed in SETTING_AES_SEEDS {
        let key = format!("{:x}", md5::compute(format!("{ts}{seed}")));
        match crypto::decrypt_base64(data, &key) {
            Ok(decrypted) => return Ok(decrypted),
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| JmError::Decrypt("Unable to decrypt setting data".into())))
}

fn string_from_scalar<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(value) => value,
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        _ => String::new(),
    })
}

pub async fn invalidate_img_host(endpoint: &str) {
    IMG_HOST_CACHE.write().await.remove(endpoint);
}

async fn cached_img_host(endpoint: &str) -> Option<String> {
    let now = Instant::now();
    let mut cache = IMG_HOST_CACHE.write().await;
    cache.retain(|_, cached| now < cached.expires_at);
    cache.get(endpoint).map(|cached| cached.host.clone())
}

async fn insert_cached_img_host(endpoint: &str, host: String) {
    let now = Instant::now();
    let mut cache = IMG_HOST_CACHE.write().await;
    cache.retain(|_, cached| now < cached.expires_at);
    cache.insert(
        endpoint.to_string(),
        CachedImgHost {
            host,
            expires_at: now + IMG_HOST_CACHE_TTL,
        },
    );

    while cache.len() > IMG_HOST_CACHE_MAX_ENTRIES {
        let Some(oldest_endpoint) = cache
            .iter()
            .min_by_key(|(_, cached)| cached.expires_at)
            .map(|(endpoint, _)| endpoint.clone())
        else {
            break;
        };
        cache.remove(&oldest_endpoint);
    }
}

async fn endpoint_fetch_lock(endpoint: &str) -> Arc<Mutex<()>> {
    let mut locks = IMG_HOST_FETCH_LOCKS.lock().await;
    locks.retain(|_, lock| Arc::strong_count(lock) > 1);
    locks
        .entry(endpoint.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

async fn remove_endpoint_fetch_lock(endpoint: &str, fetch_lock: &Arc<Mutex<()>>) {
    let mut locks = IMG_HOST_FETCH_LOCKS.lock().await;
    let should_remove = locks.get(endpoint).is_some_and(|current| {
        Arc::ptr_eq(current, fetch_lock) && Arc::strong_count(fetch_lock) == 2
    });

    if should_remove {
        locks.remove(endpoint);
    }
}
