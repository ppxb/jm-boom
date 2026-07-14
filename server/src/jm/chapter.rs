use super::{client::JmClient, crypto, error::JmError, signature::JmRequestSignature, JmResult};
use crate::{
    domain::reader::ChapterManifest, expiring_cache::ExpiringCache, keyed_lock::KeyedLock,
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::time::Duration;

const JM_API_SECRET: &str = "185Hcomic3PAPP7R";
const CHAPTER_CACHE_TTL: Duration = Duration::from_secs(20 * 60);
const CHAPTER_CACHE_MAX_ENTRIES: usize = 256;

static CHAPTER_CACHE: Lazy<ExpiringCache<ChapterManifest>> =
    Lazy::new(|| ExpiringCache::new(CHAPTER_CACHE_TTL, CHAPTER_CACHE_MAX_ENTRIES));
static CHAPTER_FETCH_LOCKS: Lazy<KeyedLock> = Lazy::new(KeyedLock::new);

#[derive(Debug, Deserialize)]
struct ChapterResponse {
    #[serde(default)]
    code: i32,
    data: Option<serde_json::Value>,
    #[serde(default, rename = "errorMsg")]
    error_msg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChapterData {
    #[serde(default, deserialize_with = "string_from_scalar")]
    id: String,
    #[serde(default)]
    images: Vec<String>,
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

#[derive(Debug, Deserialize)]
struct ChapterDataFallback {
    #[serde(default)]
    images: Vec<String>,
}

impl JmClient {
    /// Get chapter manifest with page images
    pub async fn get_chapter(&self, endpoint: &str, chapter_id: &str) -> JmResult<ChapterManifest> {
        let cache_key = format!("{endpoint}|{chapter_id}");
        if let Some(chapter) = cached_chapter(&cache_key).await {
            return Ok(chapter);
        }

        let _guard = CHAPTER_FETCH_LOCKS.lock(&cache_key).await;
        if let Some(chapter) = cached_chapter(&cache_key).await {
            return Ok(chapter);
        }

        let chapter = self.fetch_chapter(endpoint, chapter_id).await?;
        insert_cached_chapter(&cache_key, chapter.clone()).await;
        Ok(chapter)
    }

    async fn fetch_chapter(&self, endpoint: &str, chapter_id: &str) -> JmResult<ChapterManifest> {
        let signature = JmRequestSignature::new();
        let url = format!("{endpoint}/chapter");
        let host = extract_host(&url);

        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .header("token", &signature.token)
            .header("tokenparam", &signature.tokenparam)
            .header("Host", host)
            .header(
                "user-agent",
                "Mozilla/5.0 (Linux; Android 13; jm-boom Build/TQ1A.230305.002; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/120.0.6099.230 Mobile Safari/537.36",
            )
            .query(&[("skip", ""), ("id", chapter_id)])
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

        // Try decrypt first (plugin payload)
        if let Ok(chapter) = decrypt_chapter_payload(&body, &signature.ts, chapter_id) {
            return ensure_chapter_images(chapter);
        }

        // Fallback to plain JSON
        let fallback: ChapterDataFallback = serde_json::from_str(&body)
            .map_err(|e| JmError::Decode(format!("Invalid chapter response: {e}")))?;

        ensure_chapter_images(ChapterManifest {
            id: chapter_id.to_string(),
            images: fallback.images,
        })
    }
}

async fn cached_chapter(cache_key: &str) -> Option<ChapterManifest> {
    CHAPTER_CACHE.get(cache_key).await
}

async fn insert_cached_chapter(cache_key: &str, chapter: ChapterManifest) {
    CHAPTER_CACHE.insert(cache_key, chapter).await;
}

fn ensure_chapter_images(chapter: ChapterManifest) -> JmResult<ChapterManifest> {
    if chapter.images.iter().any(|image| !image.trim().is_empty()) {
        Ok(chapter)
    } else {
        Err(JmError::MissingData)
    }
}

fn decrypt_chapter_payload(body: &str, ts: &str, chapter_id: &str) -> JmResult<ChapterManifest> {
    let envelope: ChapterResponse = serde_json::from_str(body)
        .map_err(|e| JmError::Decode(format!("Invalid envelope: {e}")))?;

    if envelope.code != 200 {
        return Err(JmError::Api(
            envelope
                .error_msg
                .unwrap_or_else(|| format!("API code {}", envelope.code)),
        ));
    }

    let data = envelope
        .data
        .ok_or_else(|| JmError::Decode("Missing data field".into()))?;

    // Handle both encrypted string and plain object
    match data {
        serde_json::Value::String(encrypted) => {
            // Decrypt using ECB mode with MD5 key
            let key = format!("{:x}", md5::compute(format!("{ts}{JM_API_SECRET}")));
            let decrypted = crypto::decrypt_aes256_ecb(&encrypted, &key)?;

            let chapter_data: ChapterData = serde_json::from_str(&decrypted)
                .map_err(|e| JmError::Decode(format!("Invalid decrypted data: {e}")))?;

            Ok(ChapterManifest {
                id: if chapter_data.id.is_empty() {
                    chapter_id.to_string()
                } else {
                    chapter_data.id
                },
                images: chapter_data.images,
            })
        }
        value => {
            // Plain JSON object
            let chapter_data: ChapterData = serde_json::from_value(value)
                .map_err(|e| JmError::Decode(format!("Invalid payload: {e}")))?;

            Ok(ChapterManifest {
                id: if chapter_data.id.is_empty() {
                    chapter_id.to_string()
                } else {
                    chapter_data.id
                },
                images: chapter_data.images,
            })
        }
    }
}

fn extract_host(url: &str) -> String {
    url.parse::<reqwest::Url>()
        .ok()
        .and_then(|u| u.host_str().map(|s| s.to_string()))
        .unwrap_or_default()
}
