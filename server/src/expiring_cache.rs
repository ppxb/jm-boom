use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

pub struct ExpiringCache<V> {
    entries: RwLock<HashMap<String, CacheEntry<V>>>,
    ttl: Duration,
    max_entries: usize,
}

struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

impl<V: Clone> ExpiringCache<V> {
    pub fn new(ttl: Duration, max_entries: usize) -> Self {
        assert!(max_entries > 0, "expiring cache capacity must be positive");
        Self {
            entries: RwLock::new(HashMap::new()),
            ttl,
            max_entries,
        }
    }

    pub async fn get(&self, key: &str) -> Option<V> {
        let now = Instant::now();
        {
            let entries = self.entries.read().await;
            match entries.get(key) {
                Some(entry) if now < entry.expires_at => return Some(entry.value.clone()),
                Some(_) => {}
                None => return None,
            }
        }

        let now = Instant::now();
        let mut entries = self.entries.write().await;
        if entries
            .get(key)
            .is_some_and(|entry| now >= entry.expires_at)
        {
            entries.remove(key);
            None
        } else {
            entries.get(key).map(|entry| entry.value.clone())
        }
    }

    pub async fn insert(&self, key: impl Into<String>, value: V) {
        let now = Instant::now();
        let mut entries = self.entries.write().await;
        entries.retain(|_, entry| now < entry.expires_at);
        entries.insert(
            key.into(),
            CacheEntry {
                value,
                expires_at: now + self.ttl,
            },
        );

        while entries.len() > self.max_entries {
            let Some(oldest_key) = entries
                .iter()
                .min_by_key(|(_, entry)| entry.expires_at)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            entries.remove(&oldest_key);
        }
    }

    pub async fn remove(&self, key: &str) {
        self.entries.write().await.remove(key);
    }

    #[cfg(test)]
    async fn len(&self) -> usize {
        self.entries.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::ExpiringCache;
    use std::time::Duration;

    #[tokio::test]
    async fn returns_valid_entries() {
        let cache = ExpiringCache::new(Duration::from_secs(60), 2);
        cache.insert("key", "value").await;

        assert_eq!(cache.get("key").await, Some("value"));
        assert_eq!(cache.len().await, 1);
    }

    #[tokio::test]
    async fn removes_expired_entries_on_access() {
        let cache = ExpiringCache::new(Duration::ZERO, 2);
        cache.insert("expired", "value").await;

        assert_eq!(cache.get("expired").await, None);
        assert_eq!(cache.len().await, 0);
    }

    #[tokio::test]
    async fn evicts_the_oldest_entry_when_capacity_is_exceeded() {
        let cache = ExpiringCache::new(Duration::from_secs(60), 2);
        cache.insert("first", 1).await;
        tokio::time::sleep(Duration::from_millis(1)).await;
        cache.insert("second", 2).await;
        tokio::time::sleep(Duration::from_millis(1)).await;
        cache.insert("third", 3).await;

        assert_eq!(cache.get("first").await, None);
        assert_eq!(cache.get("second").await, Some(2));
        assert_eq!(cache.get("third").await, Some(3));
        assert_eq!(cache.len().await, 2);
    }
}
