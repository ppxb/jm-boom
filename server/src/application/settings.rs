use crate::cache::{CacheStats, ImageCache};
use std::sync::Arc;

#[derive(Clone)]
pub struct SettingsService {
    cache: Arc<ImageCache>,
}

impl SettingsService {
    pub fn new(cache: Arc<ImageCache>) -> Self {
        Self { cache }
    }

    pub async fn clear_cache(&self) -> anyhow::Result<()> {
        self.cache.clear().await
    }

    pub async fn cache_stats(&self) -> anyhow::Result<CacheStats> {
        self.cache.stats().await
    }
}
