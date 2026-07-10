use super::{auth::JmAuth, client::JmClient, crypto, error::JmError, models::Chapter, JmResult};
use serde::Deserialize;

const JM_API_SECRET: &str = "185Hcomic3PAPP7R";

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
    #[serde(default)]
    id: String,
    #[serde(default)]
    images: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ChapterDataFallback {
    #[serde(default)]
    images: Vec<String>,
}

impl JmClient {
    /// Get chapter manifest with page images
    pub async fn get_chapter(&self, endpoint: &str, chapter_id: &str) -> JmResult<Chapter> {
        let auth = JmAuth::new();
        let url = format!("{endpoint}/chapter");
        let host = extract_host(&url);

        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .header("token", &auth.token)
            .header("tokenparam", &auth.tokenparam)
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
        if let Ok(chapter) = decrypt_chapter_payload(&body, &auth.ts, chapter_id) {
            return Ok(chapter);
        }

        // Fallback to plain JSON
        let fallback: ChapterDataFallback = serde_json::from_str(&body)
            .map_err(|e| JmError::Decode(format!("Invalid chapter response: {e}")))?;

        Ok(Chapter {
            id: chapter_id.to_string(),
            name: String::new(),
            sort: String::new(),
            images: fallback.images,
        })
    }
}

fn decrypt_chapter_payload(body: &str, ts: &str, chapter_id: &str) -> JmResult<Chapter> {
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

            Ok(Chapter {
                id: if chapter_data.id.is_empty() {
                    chapter_id.to_string()
                } else {
                    chapter_data.id
                },
                name: String::new(),
                sort: String::new(),
                images: chapter_data.images,
            })
        }
        value => {
            // Plain JSON object
            let chapter_data: ChapterData = serde_json::from_value(value)
                .map_err(|e| JmError::Decode(format!("Invalid payload: {e}")))?;

            Ok(Chapter {
                id: if chapter_data.id.is_empty() {
                    chapter_id.to_string()
                } else {
                    chapter_data.id
                },
                name: String::new(),
                sort: String::new(),
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
