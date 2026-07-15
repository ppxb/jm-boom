use super::{SourceCapabilities, SourceInfo, SourcePackage, SourcePackageError};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledSource {
    pub info: SourceInfo,
    pub capabilities: SourceCapabilities,
    pub filter_count: usize,
    pub setting_count: usize,
}

impl From<&SourcePackage> for InstalledSource {
    fn from(package: &SourcePackage) -> Self {
        Self {
            info: package.manifest.info.clone(),
            capabilities: package.capabilities.clone(),
            filter_count: package.filters.len(),
            setting_count: package.settings.len(),
        }
    }
}

#[derive(Debug, Error)]
pub enum SourceRegistryError {
    #[error(transparent)]
    Package(#[from] SourcePackageError),
    #[error("cannot downgrade source {source_id} from v{installed} to v{requested}")]
    Downgrade {
        source_id: String,
        installed: u32,
        requested: u32,
    },
    #[error("source is not installed: {0}")]
    NotInstalled(String),
    #[error("source storage error: {0}")]
    Storage(#[from] std::io::Error),
}

pub struct SourceRegistry {
    directory: PathBuf,
    packages: RwLock<BTreeMap<String, Arc<SourcePackage>>>,
    operation_lock: Mutex<()>,
}

impl SourceRegistry {
    pub fn load(directory: impl Into<PathBuf>) -> Result<Self, SourceRegistryError> {
        let directory = directory.into();
        std::fs::create_dir_all(&directory)?;

        let mut packages = BTreeMap::new();
        for entry in std::fs::read_dir(&directory)? {
            let entry = entry?;
            let path = entry.path();
            if !is_source_package(&path) {
                continue;
            }
            match SourcePackage::from_file(&path) {
                Ok(package) => {
                    let source_id = package.manifest.info.id.clone();
                    if let Some(existing) = packages.get(&source_id) {
                        let existing: &Arc<SourcePackage> = existing;
                        if existing.manifest.info.version >= package.manifest.info.version {
                            tracing::warn!(
                                %source_id,
                                path = %path.display(),
                                "忽略重复或更旧的漫画源包"
                            );
                            continue;
                        }
                    }
                    packages.insert(source_id, Arc::new(package));
                }
                Err(error) => tracing::warn!(
                    path = %path.display(),
                    %error,
                    "跳过无效的漫画源包"
                ),
            }
        }

        Ok(Self {
            directory,
            packages: RwLock::new(packages),
            operation_lock: Mutex::new(()),
        })
    }

    pub async fn list(&self) -> Vec<InstalledSource> {
        self.packages
            .read()
            .await
            .values()
            .map(|package| InstalledSource::from(package.as_ref()))
            .collect()
    }

    pub async fn get(&self, source_id: &str) -> Option<Arc<SourcePackage>> {
        self.packages.read().await.get(source_id).cloned()
    }

    pub async fn install(&self, bytes: Vec<u8>) -> Result<InstalledSource, SourceRegistryError> {
        let _operation = self.operation_lock.lock().await;
        let (package, bytes) = tokio::task::spawn_blocking(move || {
            SourcePackage::from_bytes(&bytes).map(|package| (package, bytes))
        })
        .await
        .map_err(|error| std::io::Error::other(error.to_string()))??;
        let source_id = package.manifest.info.id.clone();
        let version = package.manifest.info.version;

        if let Some(existing) = self.packages.read().await.get(&source_id) {
            if existing.manifest.info.version > version {
                return Err(SourceRegistryError::Downgrade {
                    source_id,
                    installed: existing.manifest.info.version,
                    requested: version,
                });
            }
        }

        let destination = self.package_path(&source_id);
        let temporary = self.directory.join(format!(".{source_id}.aix.tmp"));
        tokio::fs::write(&temporary, bytes).await?;
        if destination.exists() {
            tokio::fs::remove_file(&destination).await?;
        }
        tokio::fs::rename(&temporary, &destination).await?;

        let summary = InstalledSource::from(&package);
        self.packages
            .write()
            .await
            .insert(source_id, Arc::new(package));
        Ok(summary)
    }

    pub async fn remove(&self, source_id: &str) -> Result<(), SourceRegistryError> {
        let _operation = self.operation_lock.lock().await;
        if !self.packages.read().await.contains_key(source_id) {
            return Err(SourceRegistryError::NotInstalled(source_id.into()));
        }
        let path = self.package_path(source_id);
        if path.exists() {
            tokio::fs::remove_file(path).await?;
        }
        self.packages.write().await.remove(source_id);
        Ok(())
    }

    fn package_path(&self, source_id: &str) -> PathBuf {
        self.directory.join(format!("{source_id}.aix"))
    }
}

fn is_source_package(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("aix"))
}
