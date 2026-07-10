use crate::jm::{JmClient, JmResult, SettingAuth};
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyInit};
use aes::Aes256;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ecb::Decryptor;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::{collections::HashSet, future::Future, pin::Pin, sync::Arc, time::Instant};
use tokio::{sync::RwLock, task::JoinSet};

const FALLBACK_ENDPOINTS: [&str; 2] = ["https://www.cdnhjk.net", "https://www.cdnhth.club"];
const HOST_CONFIG_AES_SEED: &str = "diosfjckwpqpdfjkvnqQjsik";
const HOST_CONFIG_URLS: [&str; 2] = [
    "https://rup4a04-c02.tos-cn-hongkong.bytepluses.com/newsvr-2025.txt",
    "https://rup4a04-c01.tos-ap-southeast-1.bytepluses.com/newsvr-2025.txt",
];

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EndpointMode {
    #[default]
    Auto,
    Manual,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointProbe {
    pub endpoint: String,
    pub available: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointState {
    pub mode: EndpointMode,
    pub current_endpoint: String,
    pub selected_endpoint: Option<String>,
    pub endpoints: Vec<EndpointProbe>,
}

#[derive(Debug)]
struct EndpointInner {
    mode: EndpointMode,
    selected_endpoint: Option<String>,
    current_endpoint: String,
    endpoints: Vec<EndpointProbe>,
}

#[derive(Clone)]
pub struct EndpointManager {
    client: Client,
    db: SqlitePool,
    inner: Arc<RwLock<EndpointInner>>,
}

impl EndpointManager {
    pub async fn new(db: SqlitePool) -> anyhow::Result<Self> {
        let client = Client::builder()
            .connect_timeout(std::time::Duration::from_secs(4))
            .timeout(std::time::Duration::from_secs(8))
            .build()?;
        let mode = load_setting(&db, "endpoint_mode")
            .await?
            .and_then(|value| serde_json::from_str(&value).ok())
            .unwrap_or_default();
        let selected_endpoint = load_setting(&db, "endpoint_selected").await?;
        let current_endpoint = selected_endpoint
            .clone()
            .unwrap_or_else(|| FALLBACK_ENDPOINTS[0].to_string());
        let endpoints = FALLBACK_ENDPOINTS
            .iter()
            .map(|endpoint| EndpointProbe {
                endpoint: (*endpoint).to_string(),
                available: false,
                latency_ms: None,
                error: None,
            })
            .collect();

        Ok(Self {
            client,
            db,
            inner: Arc::new(RwLock::new(EndpointInner {
                mode,
                selected_endpoint,
                current_endpoint,
                endpoints,
            })),
        })
    }

    pub async fn current_endpoint(&self) -> String {
        self.inner.read().await.current_endpoint.clone()
    }

    pub async fn state(&self) -> EndpointState {
        let inner = self.inner.read().await;
        EndpointState {
            mode: inner.mode,
            current_endpoint: inner.current_endpoint.clone(),
            selected_endpoint: inner.selected_endpoint.clone(),
            endpoints: inner.endpoints.clone(),
        }
    }

    pub async fn refresh(&self) -> EndpointState {
        let mut candidates = self.discover_candidates().await;
        if let Some(selected) = self.inner.read().await.selected_endpoint.clone() {
            if !candidates.contains(&selected) {
                candidates.push(selected);
            }
        }
        let mut tasks = JoinSet::new();

        for endpoint in candidates {
            let manager = self.clone();
            tasks.spawn(async move { manager.probe_endpoint(endpoint).await });
        }

        let mut probes = Vec::new();
        while let Some(result) = tasks.join_next().await {
            if let Ok(probe) = result {
                probes.push(probe);
            }
        }
        probes.sort_by_key(|probe| probe.latency_ms.unwrap_or(u64::MAX));

        let mut inner = self.inner.write().await;
        inner.endpoints = probes;
        select_current_endpoint(&mut inner);
        endpoint_state(&inner)
    }

    pub async fn set_selected(&self, endpoint: Option<String>) -> anyhow::Result<EndpointState> {
        let endpoint = endpoint
            .map(|value| normalize_endpoint(&value))
            .transpose()?;

        if let Some(selected) = endpoint.as_ref() {
            let known_available = {
                let inner = self.inner.read().await;
                inner
                    .endpoints
                    .iter()
                    .any(|probe| probe.endpoint == *selected && probe.available)
            };

            if !known_available {
                let probe = self.probe_endpoint(selected.clone()).await;
                if !probe.available {
                    anyhow::bail!(
                        "Endpoint is unavailable: {}",
                        probe.error.unwrap_or_else(|| selected.clone())
                    );
                }
                let mut inner = self.inner.write().await;
                if let Some(existing) = inner
                    .endpoints
                    .iter_mut()
                    .find(|item| item.endpoint == probe.endpoint)
                {
                    *existing = probe;
                } else {
                    inner.endpoints.push(probe);
                }
            }
        }

        let mode = if endpoint.is_some() {
            EndpointMode::Manual
        } else {
            EndpointMode::Auto
        };
        save_setting(&self.db, "endpoint_mode", &serde_json::to_string(&mode)?).await?;
        save_optional_setting(&self.db, "endpoint_selected", endpoint.as_deref()).await?;

        let mut inner = self.inner.write().await;
        inner.mode = mode;
        inner.selected_endpoint = endpoint;
        select_current_endpoint(&mut inner);
        Ok(endpoint_state(&inner))
    }

    pub async fn report_failure(&self, endpoint: &str, error: &str) {
        let mut inner = self.inner.write().await;
        if let Some(probe) = inner
            .endpoints
            .iter_mut()
            .find(|probe| probe.endpoint == endpoint)
        {
            probe.available = false;
            probe.latency_ms = None;
            probe.error = Some(error.to_string());
        } else {
            inner.endpoints.push(EndpointProbe {
                endpoint: endpoint.to_string(),
                available: false,
                latency_ms: None,
                error: Some(error.to_string()),
            });
        }

        select_current_endpoint(&mut inner);
    }

    async fn discover_candidates(&self) -> Vec<String> {
        let mut candidates = FALLBACK_ENDPOINTS
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<_>>();

        for url in HOST_CONFIG_URLS {
            match self.fetch_host_config(url).await {
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

    async fn fetch_host_config(&self, url: &str) -> anyhow::Result<Vec<String>> {
        let body = self
            .client
            .get(url)
            .header("accept", "text/plain,*/*")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        let encoded = body
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '/' | '='))
            .collect::<String>();
        let key = format!("{:x}", md5::compute(HOST_CONFIG_AES_SEED));
        let encrypted = BASE64.decode(encoded)?;
        let decrypted = Decryptor::<Aes256>::new_from_slice(key.as_bytes())?
            .decrypt_padded_vec_mut::<Pkcs7>(&encrypted)
            .map_err(|error| anyhow::anyhow!("failed to decrypt endpoint config: {error}"))?;
        let payload: HostConfigPayload = serde_json::from_slice(&decrypted)?;
        Ok(payload.server)
    }

    async fn probe_endpoint(&self, endpoint: String) -> EndpointProbe {
        let auth = SettingAuth::current();
        let started = Instant::now();
        let result = self
            .client
            .get(format!("{endpoint}/setting"))
            .header("token", auth.token)
            .header("tokenparam", auth.tokenparam)
            .query(&[("app_img_shunt", "1"), ("t", auth.ts.as_str())])
            .send()
            .await
            .and_then(reqwest::Response::error_for_status);

        match result {
            Ok(_) => EndpointProbe {
                endpoint,
                available: true,
                latency_ms: Some(started.elapsed().as_millis() as u64),
                error: None,
            },
            Err(error) => EndpointProbe {
                endpoint,
                available: false,
                latency_ms: None,
                error: Some(error.to_string()),
            },
        }
    }
}

pub async fn request_with_failover<T, F>(
    jm: &JmClient,
    endpoints: &EndpointManager,
    operation: F,
) -> JmResult<(String, T)>
where
    F: for<'a> Fn(&'a JmClient, &'a str) -> Pin<Box<dyn Future<Output = JmResult<T>> + Send + 'a>>,
{
    let endpoint = endpoints.current_endpoint().await;

    match operation(jm, &endpoint).await {
        Ok(value) => Ok((endpoint, value)),
        Err(error) if error.is_retryable() => {
            endpoints
                .report_failure(&endpoint, &error.to_string())
                .await;
            let next_endpoint = endpoints.current_endpoint().await;

            if next_endpoint == endpoint {
                return Err(error);
            }

            tracing::warn!(from = %endpoint, to = %next_endpoint, "retrying with next endpoint");
            match operation(jm, &next_endpoint).await {
                Ok(value) => Ok((next_endpoint, value)),
                Err(error) => {
                    endpoints
                        .report_failure(&next_endpoint, &error.to_string())
                        .await;
                    Err(error)
                }
            }
        }
        Err(error) => Err(error),
    }
}

#[derive(Deserialize)]
struct HostConfigPayload {
    server: Vec<String>,
}

fn select_current_endpoint(inner: &mut EndpointInner) {
    if inner.mode == EndpointMode::Manual {
        if let Some(selected) = inner.selected_endpoint.as_ref() {
            let selected_available = inner
                .endpoints
                .iter()
                .find(|probe| probe.endpoint == *selected)
                .map(|probe| probe.available);
            if selected_available.unwrap_or(true) {
                inner.current_endpoint = selected.clone();
                return;
            }
        }
    }

    if let Some(fastest) = inner
        .endpoints
        .iter()
        .filter(|probe| probe.available)
        .min_by_key(|probe| probe.latency_ms.unwrap_or(u64::MAX))
    {
        inner.current_endpoint = fastest.endpoint.clone();
    }
}

fn endpoint_state(inner: &EndpointInner) -> EndpointState {
    EndpointState {
        mode: inner.mode,
        current_endpoint: inner.current_endpoint.clone(),
        selected_endpoint: inner.selected_endpoint.clone(),
        endpoints: inner.endpoints.clone(),
    }
}

fn normalize_endpoint(value: &str) -> anyhow::Result<String> {
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

async fn load_setting(db: &SqlitePool, key: &str) -> anyhow::Result<Option<String>> {
    Ok(
        sqlx::query_scalar::<_, String>("SELECT value FROM app_settings WHERE key = ?")
            .bind(key)
            .fetch_optional(db)
            .await?,
    )
}

async fn save_setting(db: &SqlitePool, key: &str, value: &str) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?, ?, ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(key)
    .bind(value)
    .bind(chrono::Utc::now().timestamp())
    .execute(db)
    .await?;
    Ok(())
}

async fn save_optional_setting(
    db: &SqlitePool,
    key: &str,
    value: Option<&str>,
) -> anyhow::Result<()> {
    if let Some(value) = value {
        save_setting(db, key, value).await
    } else {
        sqlx::query("DELETE FROM app_settings WHERE key = ?")
            .bind(key)
            .execute(db)
            .await?;
        Ok(())
    }
}
