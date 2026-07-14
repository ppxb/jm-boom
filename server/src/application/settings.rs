use crate::{
    cache::{CacheStats, ImageCache},
    endpoint::{EndpointManager, EndpointState},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct SettingsService {
    endpoints: Arc<EndpointManager>,
    cache: Arc<ImageCache>,
}

impl SettingsService {
    pub fn new(endpoints: Arc<EndpointManager>, cache: Arc<ImageCache>) -> Self {
        Self { endpoints, cache }
    }

    pub async fn endpoints(&self) -> EndpointState {
        self.endpoints.state().await
    }

    pub async fn refresh_endpoints(&self) -> EndpointState {
        self.endpoints.refresh().await
    }

    pub async fn set_endpoint(&self, endpoint: Option<String>) -> anyhow::Result<EndpointState> {
        self.endpoints.set_selected(endpoint).await
    }

    pub async fn clear_cache(&self) -> anyhow::Result<()> {
        self.cache.clear().await
    }

    pub async fn cache_stats(&self) -> anyhow::Result<CacheStats> {
        self.cache.stats().await
    }
}
