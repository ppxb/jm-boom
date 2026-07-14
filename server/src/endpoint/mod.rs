mod discovery;
mod failover;
mod probe;
mod repository;

#[cfg(test)]
mod tests;

pub use failover::request_with_failover;

use discovery::normalize_endpoint;
use repository::EndpointRepository;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, RwLock},
    task::JoinSet,
    time::MissedTickBehavior,
};

pub(super) const FALLBACK_ENDPOINTS: [&str; 2] =
    ["https://www.cdnhjk.net", "https://www.cdnhth.club"];
const ENDPOINT_REFRESH_INTERVAL: Duration = Duration::from_secs(2 * 60 * 60);

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
    probe_completed: bool,
    selected_endpoint: Option<String>,
    current_endpoint: String,
    endpoints: Vec<EndpointProbe>,
}

#[derive(Clone)]
pub struct EndpointManager {
    client: Client,
    repository: EndpointRepository,
    inner: Arc<RwLock<EndpointInner>>,
    refresh_lock: Arc<Mutex<()>>,
}

impl EndpointManager {
    pub async fn new(db: sqlx::SqlitePool) -> anyhow::Result<Self> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(4))
            .timeout(Duration::from_secs(8))
            .build()?;
        let repository = EndpointRepository::new(db);
        let mode = repository.load_mode().await?;
        let selected_endpoint = repository.load_selected().await?;
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
            repository,
            inner: Arc::new(RwLock::new(EndpointInner {
                mode,
                probe_completed: false,
                selected_endpoint,
                current_endpoint,
                endpoints,
            })),
            refresh_lock: Arc::new(Mutex::new(())),
        })
    }

    pub fn start_maintenance(self: &Arc<Self>) {
        let manager = Arc::downgrade(self);
        tokio::spawn(async move {
            let Some(endpoint_manager) = manager.upgrade() else {
                return;
            };
            let state = endpoint_manager.refresh().await;
            tracing::info!(
                endpoint = %state.current_endpoint,
                "initial endpoint probe completed"
            );
            drop(endpoint_manager);

            let mut interval = tokio::time::interval(ENDPOINT_REFRESH_INTERVAL);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            interval.tick().await;

            loop {
                interval.tick().await;
                let Some(endpoint_manager) = manager.upgrade() else {
                    break;
                };
                let state = endpoint_manager.refresh().await;
                tracing::info!(
                    endpoint = %state.current_endpoint,
                    "periodic endpoint probe completed"
                );
            }
        });
    }

    pub async fn state(&self) -> EndpointState {
        let inner = self.inner.read().await;
        endpoint_state(&inner)
    }

    pub async fn refresh(&self) -> EndpointState {
        let _refresh = self.refresh_lock.lock().await;
        let mut candidates = discovery::discover_candidates(&self.client).await;
        if let Some(selected) = self.inner.read().await.selected_endpoint.clone() {
            if !candidates.contains(&selected) {
                candidates.push(selected);
            }
        }
        let mut tasks = JoinSet::new();

        for endpoint in candidates {
            let client = self.client.clone();
            tasks.spawn(async move { probe::probe_endpoint(&client, endpoint).await });
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
        inner.probe_completed = true;
        select_current_endpoint(&mut inner);
        endpoint_state(&inner)
    }

    pub async fn set_selected(&self, endpoint: Option<String>) -> anyhow::Result<EndpointState> {
        let _refresh = self.refresh_lock.lock().await;
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
                let probe = probe::probe_endpoint(&self.client, selected.clone()).await;
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
        self.repository
            .save_selection(mode, endpoint.as_deref())
            .await?;

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

    pub async fn report_success(&self, endpoint: &str, latency_ms: u64) {
        let mut inner = self.inner.write().await;
        if let Some(probe) = inner
            .endpoints
            .iter_mut()
            .find(|probe| probe.endpoint == endpoint)
        {
            probe.available = true;
            probe.latency_ms = Some(latency_ms);
            probe.error = None;
        } else {
            inner.endpoints.push(EndpointProbe {
                endpoint: endpoint.to_string(),
                available: true,
                latency_ms: Some(latency_ms),
                error: None,
            });
        }
        inner.current_endpoint = endpoint.to_string();
    }
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
