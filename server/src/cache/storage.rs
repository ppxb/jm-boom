use anyhow::{Context, Result};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use tokio::fs;

static CACHE_WRITE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub(super) struct CacheStorage {
    root: PathBuf,
}

impl CacheStorage {
    pub(super) async fn new(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root).await?;
        Ok(Self { root })
    }

    pub(super) fn root(&self) -> &Path {
        &self.root
    }

    pub(super) fn path_for(&self, key: &str, extension: &str) -> PathBuf {
        if let Some((chapter_id, page)) = key.split_once(':') {
            self.root
                .join(chapter_id)
                .join(format!("{page}.{extension}"))
        } else {
            self.root.join(format!("{key}.{extension}"))
        }
    }

    pub(super) fn normalize_path(&self, path: &str) -> PathBuf {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            path
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(path)
        }
    }

    pub(super) async fn read(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        match fs::read(path).await {
            Ok(data) => Ok(Some(data)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub(super) async fn write(&self, path: &Path, data: &[u8]) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let pending_file = PendingCacheFile::new(temporary_path(path)?);
        fs::write(pending_file.path(), data).await?;
        fs::rename(pending_file.path(), path).await?;
        pending_file.commit();
        Ok(())
    }

    pub(super) async fn reset(&self) -> Result<()> {
        match fs::remove_dir_all(&self.root).await {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        fs::create_dir_all(&self.root).await?;
        Ok(())
    }

    pub(super) async fn metadata(&self, path: &Path) -> Result<Option<std::fs::Metadata>> {
        match fs::metadata(path).await {
            Ok(metadata) => Ok(Some(metadata)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub(super) async fn remove_file(&self, path: &Path) -> Result<()> {
        match fs::remove_file(path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    pub(super) async fn scan_files(&self) -> Result<Vec<PathBuf>> {
        let root = self.root.clone();
        tokio::task::spawn_blocking(move || scan_files_sync(&root))
            .await
            .context("cache directory scan task failed")?
    }
}

struct PendingCacheFile {
    path: PathBuf,
    active: bool,
}

impl PendingCacheFile {
    fn new(path: PathBuf) -> Self {
        Self { path, active: true }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn commit(mut self) {
        self.active = false;
    }
}

impl Drop for PendingCacheFile {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        match std::fs::remove_file(&self.path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => tracing::warn!(
                path = %self.path.display(),
                %error,
                "图片缓存临时文件清理失败"
            ),
        }
    }
}

fn scan_files_sync(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut directories = vec![root.to_path_buf()];

    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                directories.push(entry.path());
            } else if file_type.is_file() {
                files.push(entry.path());
            }
        }
    }

    Ok(files)
}

pub(super) fn temporary_path(path: &Path) -> Result<PathBuf> {
    let file_name = path
        .file_name()
        .context("cache path must have a file name")?
        .to_string_lossy();
    let sequence = CACHE_WRITE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    Ok(path.with_file_name(format!(
        ".{file_name}.tmp-{}-{sequence}",
        std::process::id()
    )))
}
