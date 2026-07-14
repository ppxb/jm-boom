use super::FALLBACK_ENDPOINTS;
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyInit};
use aes::Aes256;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ecb::Decryptor;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;

const HOST_CONFIG_AES_SEED: &str = "diosfjckwpqpdfjkvnqQjsik";
const HOST_CONFIG_URLS: [&str; 2] = [
    "https://rup4a04-c02.tos-cn-hongkong.bytepluses.com/newsvr-2025.txt",
    "https://rup4a04-c01.tos-ap-southeast-1.bytepluses.com/newsvr-2025.txt",
];

pub(super) async fn discover_candidates(client: &Client) -> Vec<String> {
    let mut candidates = FALLBACK_ENDPOINTS
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();

    for url in HOST_CONFIG_URLS {
        match fetch_host_config(client, url).await {
            Ok(hosts) => {
                candidates.extend(hosts);
                break;
            }
            Err(error) => tracing::warn!(url, %error, "failed to load endpoint config"),
        }
    }

    let mut unique = HashSet::new();
    candidates
        .into_iter()
        .filter_map(|value| normalize_endpoint(&value).ok())
        .filter(|value| unique.insert(value.clone()))
        .collect()
}

async fn fetch_host_config(client: &Client, url: &str) -> anyhow::Result<Vec<String>> {
    let body = client
        .get(url)
        .header("accept", "text/plain,*/*")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    decode_host_config(&body)
}

fn decode_host_config(body: &str) -> anyhow::Result<Vec<String>> {
    let encoded = body
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '/' | '='))
        .collect::<String>();
    let key = format!("{:x}", md5::compute(HOST_CONFIG_AES_SEED));
    let encrypted = BASE64.decode(encoded)?;
    let decryptor = Decryptor::<Aes256>::new_from_slice(key.as_bytes())
        .map_err(|_| anyhow::anyhow!("invalid endpoint config key length"))?;
    let decrypted = decryptor
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted)
        .map_err(|error| anyhow::anyhow!("failed to decrypt endpoint config: {error}"))?;
    let payload: HostConfigPayload = serde_json::from_slice(&decrypted)?;
    if payload.server.is_empty() {
        anyhow::bail!("host config did not include Server endpoints");
    }
    Ok(payload.server)
}

pub(super) fn normalize_endpoint(value: &str) -> anyhow::Result<String> {
    let value = value.trim().trim_end_matches('/');
    let value = if value.starts_with("http://") || value.starts_with("https://") {
        value.to_string()
    } else {
        format!("https://{value}")
    };
    let url = reqwest::Url::parse(&value)?;

    if url.scheme() != "https" || url.host_str().is_none() || url.path() != "/" {
        anyhow::bail!("Endpoint must be an HTTPS origin");
    }

    let host = url.host_str().expect("validated host");
    let port = url
        .port()
        .map(|port| format!(":{port}"))
        .unwrap_or_default();
    Ok(format!("https://{host}{port}"))
}

#[derive(Deserialize)]
struct HostConfigPayload {
    #[serde(default, rename = "Server", alias = "server")]
    server: Vec<String>,
}
