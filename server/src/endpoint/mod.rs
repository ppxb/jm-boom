mod discovery;
mod failover;
mod probe;

#[cfg(test)]
mod tests;

pub use failover::request_with_failover;

use reqwest::Client;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, RwLock},
    task::JoinSet,
    time::MissedTickBehavior,
};

pub(super) const FALLBACK_ENDPOINTS: [&str; 2] =
    ["https://www.cdnhjk.net", "https://www.cdnhth.club"];
const ENDPOINT_REFRESH_INTERVAL: Duration = Duration::from_secs(2 * 60 * 60);

#[derive(Clone, Debug)]
pub struct EndpointProbe {
    pub endpoint: String,
    pub available: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug)]
struct EndpointInner {
    probe_completed: bool,
    current_endpoint: String,
    endpoints: Vec<EndpointProbe>,
}

#[derive(Clone)]
pub struct EndpointManager {
    client: Client,
    inner: Arc<RwLock<EndpointInner>>,
    refresh_lock: Arc<Mutex<()>>,
}

impl EndpointManager {
    pub async fn new() -> anyhow::Result<Self> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(4))
            .timeout(Duration::from_secs(8))
            .build()?;
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
            inner: Arc::new(RwLock::new(EndpointInner {
                probe_completed: false,
                current_endpoint: FALLBACK_ENDPOINTS[0].to_string(),
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
            let endpoint = endpoint_manager.refresh().await;
            tracing::info!(
                endpoint = %endpoint,
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
                let endpoint = endpoint_manager.refresh().await;
                tracing::info!(
                    endpoint = %endpoint,
                    "periodic endpoint probe completed"
                );
            }
        });
    }

    pub async fn refresh(&self) -> String {
        let _refresh = self.refresh_lock.lock().await;
        let candidates = discovery::discover_candidates(&self.client).await;
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
        inner.current_endpoint.clone()
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
    if let Some(fastest) = inner
        .endpoints
        .iter()
        .filter(|probe| probe.available)
        .min_by_key(|probe| probe.latency_ms.unwrap_or(u64::MAX))
    {
        inner.current_endpoint = fastest.endpoint.clone();
    }
}
