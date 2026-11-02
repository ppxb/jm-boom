use super::{repository::CacheCorrection, ImageCache};
use anyhow::Result;
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

const EVICTION_BATCH_ENTRIES: i64 = 64;
const EVICTION_BATCH_TIME: Duration = Duration::from_millis(10);

#[derive(Debug, Default)]
pub(super) struct RepairSummary {
    pub removed_index_entries: usize,
    pub removed_orphan_files: usize,
    pub corrected_index_entries: usize,
}

impl ImageCache {
    pub(super) async fn repair_orphans(&self) -> Result<RepairSummary> {
        let _operation = self.operation_lock.write().await;
        let rows = self.repository.entries().await?;
        let mut indexed_paths = HashSet::new();
        let mut removed_keys = Vec::new();
        let mut corrections = Vec::new();

        for entry in rows {
            let path = PathBuf::from(&entry.path);
            if !path.starts_with(self.storage.root()) {
                removed_keys.push(entry.key);
                continue;
            }

            match self.storage.metadata(&path).await? {
                Some(metadata) if metadata.is_file() => {
                    let actual_size = i64::try_from(metadata.len()).unwrap_or(i64::MAX);
                    let normalized_path = path.to_string_lossy().into_owned();
                    if actual_size != entry.size || normalized_path != entry.path {
                        corrections.push(CacheCorrection {
                            key: entry.key,
                            path: normalized_path,
                            size: actual_size,
                        });
                    }
                    indexed_paths.insert(path);
                }
                Some(_) | None => removed_keys.push(entry.key),
            }
        }

        let orphan_files = self
            .storage
            .scan_files()
            .await?
            .into_iter()
            .filter(|path| !indexed_paths.contains(path))
            .collect::<Vec<_>>();
        for path in &orphan_files {
            self.storage.remove_file(path).await?;
        }

        self.repository
            .apply_repairs(&removed_keys, &corrections)
            .await?;
        let size_bytes = self.repository.total_size().await?.max(0);
        self.current_size_bytes.store(size_bytes, Ordering::Relaxed);

        Ok(RepairSummary {
            removed_index_entries: removed_keys.len(),
            removed_orphan_files: orphan_files.len(),
            corrected_index_entries: corrections.len(),
        })
    }

    pub(super) async fn evict_if_needed(&self) -> Result<usize> {
        if self.current_size_bytes.load(Ordering::Relaxed) <= self.max_size_bytes {
            return Ok(0);
        }
        let Ok(eviction) = self.eviction_lock.try_lock() else {
            // The active eviction will account for writes between its batches.
            return Ok(0);
        };

        // Flush the accumulated access log without excluding cache readers.
        if let Err(error) = self.flush_accesses().await {
            tracing::warn!(%error, "LRU 淘汰前刷新访问时间失败");
        }
        let mut operation = self.operation_lock.write().await;
        let mut size_bytes = self.repository.total_size().await?.max(0);
        self.current_size_bytes.store(size_bytes, Ordering::Relaxed);
        if size_bytes <= self.max_size_bytes {
            drop(eviction);
            return Ok(0);
        }

        let initial_size = size_bytes;
        let target_size_bytes = self.max_size_bytes / 2;
        let mut evicted_entries = 0;

        while size_bytes > target_size_bytes {
            let batch_started = Instant::now();
            if let Err(error) = self.flush_accesses_inner().await {
                tracing::warn!(%error, "LRU 淘汰批次刷新访问时间失败");
            }
            // Re-select under the lock so replacements and recent reads are respected.
            let candidates = self
                .repository
                .eviction_candidates(EVICTION_BATCH_ENTRIES)
                .await?;
            if candidates.is_empty() {
                break;
            }

            let mut removed_keys = Vec::new();
            let mut removed_size = 0_i64;
            let mut file_error = None;
            for entry in candidates {
                let path = PathBuf::from(&entry.path);
                if path.starts_with(self.storage.root()) {
                    if let Err(error) = self.storage.remove_file(&path).await {
                        file_error = Some(error);
                        break;
                    }
                }
                removed_size = removed_size.saturating_add(entry.size.max(0));
                removed_keys.push(entry.key);
                if size_bytes.saturating_sub(removed_size) <= target_size_bytes
                    || batch_started.elapsed() >= EVICTION_BATCH_TIME
                {
                    break;
                }
            }

            // Commit successful removals even when a later file in the batch failed.
            self.repository.delete_many(&removed_keys).await?;
            {
                let mut pending = self.pending_accesses.lock().await;
                for key in &removed_keys {
                    pending.remove(key);
                }
            }
            size_bytes = self.adjust_current_size(-removed_size);
            evicted_entries += removed_keys.len();
            if let Some(error) = file_error {
                return Err(error);
            }
            if size_bytes <= target_size_bytes {
                break;
            }

            drop(operation);
            tokio::task::yield_now().await;
            operation = self.operation_lock.write().await;
            size_bytes = self.current_size_bytes.load(Ordering::Relaxed);
        }

        tracing::info!(
            initial_size_bytes = initial_size,
            remaining_size_bytes = size_bytes,
            max_size_bytes = self.max_size_bytes,
            target_size_bytes,
            evicted_entries,
            "图片缓存 LRU 淘汰完成"
        );
        // Release ownership before admitting writers that may need a new eviction.
        drop(eviction);
        Ok(evicted_entries)
    }
}
