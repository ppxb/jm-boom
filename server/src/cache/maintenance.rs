use super::{repository::CacheCorrection, ImageCache};
use anyhow::Result;
use std::{collections::HashSet, sync::atomic::Ordering};

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
            let path = self.storage.normalize_path(&entry.path);
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
        let _operation = self.operation_lock.write().await;
        if self.current_size_bytes.load(Ordering::Relaxed) <= self.max_size_bytes {
            return Ok(0);
        }
        self.evict_if_needed_locked().await
    }

    async fn evict_if_needed_locked(&self) -> Result<usize> {
        let mut size_bytes = self.repository.total_size().await?.max(0);
        self.current_size_bytes.store(size_bytes, Ordering::Relaxed);
        if size_bytes <= self.max_size_bytes {
            return Ok(0);
        }

        if let Err(error) = self.flush_accesses_inner().await {
            tracing::warn!(%error, "LRU 淘汰前刷新访问时间失败");
        }

        let candidates = self.repository.eviction_candidates().await?;
        let initial_size = size_bytes;
        let target_size_bytes = self.max_size_bytes / 2;
        let mut evicted_entries = 0;

        for entry in candidates {
            if size_bytes <= target_size_bytes {
                break;
            }

            let path = self.storage.normalize_path(&entry.path);
            if path.starts_with(self.storage.root()) {
                self.storage.remove_file(&path).await?;
            }

            self.repository.delete(&entry.key).await?;
            self.pending_accesses.lock().await.remove(&entry.key);
            size_bytes = size_bytes.saturating_sub(entry.size.max(0));
            evicted_entries += 1;
        }
        self.current_size_bytes.store(size_bytes, Ordering::Relaxed);

        tracing::info!(
            initial_size_bytes = initial_size,
            remaining_size_bytes = size_bytes,
            max_size_bytes = self.max_size_bytes,
            target_size_bytes,
            evicted_entries,
            "图片缓存 LRU 淘汰完成"
        );
        Ok(evicted_entries)
    }
}
