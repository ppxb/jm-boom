use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

#[derive(Clone)]
pub struct KeyedLock {
    inner: Arc<KeyedLockInner>,
}

struct KeyedLockInner {
    entries: Mutex<HashMap<String, Weak<KeyedLockEntry>>>,
}

struct KeyedLockEntry {
    key: String,
    registry: Weak<KeyedLockInner>,
    lock: Arc<AsyncMutex<()>>,
}

pub struct KeyedLockGuard {
    guard: Option<OwnedMutexGuard<()>>,
    _entry: Arc<KeyedLockEntry>,
}

impl KeyedLock {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(KeyedLockInner {
                entries: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub async fn lock(&self, key: &str) -> KeyedLockGuard {
        let entry = self.entry(key);
        let guard = entry.lock.clone().lock_owned().await;
        KeyedLockGuard {
            guard: Some(guard),
            _entry: entry,
        }
    }

    fn entry(&self, key: &str) -> Arc<KeyedLockEntry> {
        let mut entries = self
            .inner
            .entries
            .lock()
            .expect("keyed lock registry poisoned");
        if let Some(entry) = entries.get(key).and_then(Weak::upgrade) {
            return entry;
        }

        let entry = Arc::new(KeyedLockEntry {
            key: key.to_string(),
            registry: Arc::downgrade(&self.inner),
            lock: Arc::new(AsyncMutex::new(())),
        });
        entries.insert(key.to_string(), Arc::downgrade(&entry));
        entry
    }

    #[cfg(test)]
    fn entry_count(&self) -> usize {
        self.inner
            .entries
            .lock()
            .expect("keyed lock registry poisoned")
            .len()
    }

    #[cfg(test)]
    fn participant_count(&self, key: &str) -> usize {
        self.inner
            .entries
            .lock()
            .expect("keyed lock registry poisoned")
            .get(key)
            .map_or(0, Weak::strong_count)
    }
}

impl Drop for KeyedLockGuard {
    fn drop(&mut self) {
        self.guard.take();
    }
}

impl Drop for KeyedLockEntry {
    fn drop(&mut self) {
        let Some(registry) = self.registry.upgrade() else {
            return;
        };
        let mut entries = registry
            .entries
            .lock()
            .expect("keyed lock registry poisoned");
        let should_remove = entries
            .get(&self.key)
            .is_some_and(|entry| std::ptr::eq(entry.as_ptr(), self));
        if should_remove {
            entries.remove(&self.key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::KeyedLock;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use tokio::{sync::oneshot, time::Duration};

    #[tokio::test]
    async fn serializes_the_same_key_and_cleans_up_after_release() {
        let locks = KeyedLock::new();
        let first = locks.lock("same").await;
        let acquired = Arc::new(AtomicBool::new(false));
        let waiter_locks = locks.clone();
        let waiter_acquired = acquired.clone();
        let waiter = tokio::spawn(async move {
            let _guard = waiter_locks.lock("same").await;
            waiter_acquired.store(true, Ordering::SeqCst);
        });

        tokio::time::timeout(Duration::from_secs(1), async {
            while locks.participant_count("same") < 2 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("same-key waiter did not register");
        assert!(!acquired.load(Ordering::SeqCst));

        drop(first);
        tokio::time::timeout(Duration::from_secs(1), waiter)
            .await
            .expect("same-key waiter timed out")
            .expect("same-key waiter failed");
        assert!(acquired.load(Ordering::SeqCst));
        assert_eq!(locks.entry_count(), 0);
    }

    #[tokio::test]
    async fn allows_different_keys_to_run_concurrently() {
        let locks = KeyedLock::new();
        let first = locks.lock("first").await;

        let second = tokio::time::timeout(Duration::from_secs(1), locks.lock("second"))
            .await
            .expect("different key should not wait");
        assert_eq!(locks.entry_count(), 2);

        drop(second);
        drop(first);
        assert_eq!(locks.entry_count(), 0);
    }

    #[tokio::test]
    async fn removes_the_entry_when_a_holder_is_cancelled() {
        let locks = KeyedLock::new();
        let task_locks = locks.clone();
        let (ready, acquired) = oneshot::channel();
        let holder = tokio::spawn(async move {
            let _guard = task_locks.lock("cancelled").await;
            let _ = ready.send(());
            std::future::pending::<()>().await;
        });

        acquired.await.expect("holder should acquire the key");
        assert_eq!(locks.entry_count(), 1);
        holder.abort();
        let _ = holder.await;
        assert_eq!(locks.entry_count(), 0);
    }

    #[tokio::test]
    async fn keeps_the_entry_consistent_when_a_waiter_is_cancelled() {
        let locks = KeyedLock::new();
        let holder = locks.lock("waiting-cancelled").await;
        let waiter_locks = locks.clone();
        let waiter = tokio::spawn(async move {
            let _guard = waiter_locks.lock("waiting-cancelled").await;
        });

        tokio::time::timeout(Duration::from_secs(1), async {
            while locks.participant_count("waiting-cancelled") < 2 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("cancelled waiter did not register");
        waiter.abort();
        let _ = waiter.await;
        assert_eq!(locks.participant_count("waiting-cancelled"), 1);

        drop(holder);
        assert_eq!(locks.entry_count(), 0);
    }
}
