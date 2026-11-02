use super::{probe, select_current_endpoint, EndpointManager};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EndpointHealth {
    failures: u32,
    retry_at: Instant,
}

impl EndpointHealth {
    pub(super) fn fail(previous: Option<&Self>, now: Instant) -> Self {
        let failures = previous.map_or(1, |health| health.failures.saturating_add(1));
        let seconds = (30u64 << (failures - 1).min(5)).min(600);
        Self {
            failures,
            retry_at: now + Duration::from_secs(seconds),
        }
    }

    pub(super) fn cooling(&self, now: Instant) -> bool {
        self.retry_at > now
    }
}

impl EndpointManager {
    pub(super) async fn recover_cooling_endpoints(&self) {
        let _refresh = self.refresh_lock.lock().await;
        let now = Instant::now();
        let due: HashMap<_, _> = self
            .inner
            .read()
            .await
            .api_health
            .iter()
            .filter(|(_, health)| !health.cooling(now))
            .map(|(host, health)| (host.clone(), health.clone()))
            .collect();
        let mut image_hosts = Vec::new();
        for (probe, image_host) in
            probe::probe_all(&self.client, due.keys().cloned().collect()).await
        {
            let mut inner = self.inner.write().await;
            // Do not overwrite a newer request's success or failure with an older probe.
            if inner.api_health.get(&probe.endpoint) != due.get(&probe.endpoint) {
                continue;
            }
            if probe.available {
                inner.api_health.remove(&probe.endpoint);
                if let Some(host) = image_host {
                    image_hosts.push(host);
                }
            } else {
                let health =
                    EndpointHealth::fail(inner.api_health.get(&probe.endpoint), Instant::now());
                inner.api_health.insert(probe.endpoint.clone(), health);
            }
            if let Some(existing) = inner
                .endpoints
                .iter_mut()
                .find(|item| item.endpoint == probe.endpoint)
            {
                *existing = probe;
            }
            select_current_endpoint(&mut inner);
        }
        if !image_hosts.is_empty() {
            let known = self.inner.read().await.image_hosts.clone();
            image_hosts.retain(|host| !known.contains(host));
            if !image_hosts.is_empty() {
                image_hosts.extend(known);
                self.refresh_image_hosts(image_hosts).await;
            }
        }
        self.recover_image_hosts().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backs_off_transient_failures_and_caps_cooldown() {
        let now = Instant::now();
        let mut health = EndpointHealth::fail(None, now);
        assert!(health.cooling(now + Duration::from_secs(29)));
        assert!(!health.cooling(now + Duration::from_secs(30)));
        health = EndpointHealth::fail(Some(&health), now);
        assert_eq!(health.retry_at.duration_since(now), Duration::from_secs(60));
        for _ in 0..20 {
            health = EndpointHealth::fail(Some(&health), now);
        }
        assert_eq!(
            health.retry_at.duration_since(now),
            Duration::from_secs(600)
        );
    }
}
