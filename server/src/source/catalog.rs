use std::{sync::Arc, time::Duration};

use reqwest::Url;
use serde::Serialize;
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};

use super::{
    CompiledSource, InstalledSource, SourceList, SourceListEntry, SourcePackage, SourceRegistry,
    SourceRegistryError, SourceRuntimeError,
};

const DEFAULT_SOURCE_LIST_URL: &str = "https://aidoku-community.github.io/sources/index.min.json";
const MAX_SOURCE_LIST_BYTES: usize = 4 * 1024 * 1024;
const MAX_SOURCE_PACKAGE_BYTES: usize = 128 * 1024 * 1024;
const CATALOG_CACHE_TTL: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableSource {
    pub id: String,
    pub name: String,
    pub version: u32,
    pub icon_url: Option<String>,
    pub download_url: Option<String>,
    pub languages: Vec<String>,
    pub content_rating: u8,
    pub installed_version: Option<u32>,
}

#[derive(Debug, Error)]
pub enum SourceCatalogError {
    #[error("source catalog request failed: {0}")]
    Request(String),
    #[error("source catalog is invalid: {0}")]
    List(#[from] super::SourceListError),
    #[error("source catalog response is too large")]
    ListTooLarge,
    #[error("source is not present in the catalog: {0}")]
    SourceNotFound(String),
    #[error("source download URL is not allowed: {0}")]
    DownloadUrl(String),
    #[error("source package response is too large")]
    PackageTooLarge,
    #[error(
        "downloaded package does not match requested source: expected {expected}, got {actual}"
    )]
    SourceIdMismatch { expected: String, actual: String },
    #[error(transparent)]
    Package(#[from] super::SourcePackageError),
    #[error(transparent)]
    Runtime(#[from] SourceRuntimeError),
    #[error(transparent)]
    Registry(#[from] SourceRegistryError),
    #[error("source worker failed: {0}")]
    Worker(String),
}

struct CatalogSnapshot {
    fetched_at: tokio::time::Instant,
    list: SourceList,
}

pub struct SourceCatalogService {
    client: reqwest::Client,
    list_url: Url,
    registry: Arc<SourceRegistry>,
    snapshot: RwLock<Option<CatalogSnapshot>>,
    fetch_lock: Mutex<()>,
    install_lock: Mutex<()>,
}

impl SourceCatalogService {
    pub fn from_env(registry: Arc<SourceRegistry>) -> Result<Self, SourceCatalogError> {
        let value = std::env::var("JM_BOOM_SOURCE_LIST_URL")
            .unwrap_or_else(|_| DEFAULT_SOURCE_LIST_URL.into());
        let list_url = Url::parse(&value)
            .map_err(|error| SourceCatalogError::DownloadUrl(error.to_string()))?;
        if list_url.scheme() != "https" || list_url.host_str().is_none() {
            return Err(SourceCatalogError::DownloadUrl(
                "source list URL must use HTTPS".into(),
            ));
        }
        validate_catalog_url(&list_url)?;
        Self::new(registry, list_url)
    }

    pub fn new(registry: Arc<SourceRegistry>, list_url: Url) -> Result<Self, SourceCatalogError> {
        validate_catalog_url(&list_url)?;
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|error| SourceCatalogError::Request(error.to_string()))?;
        Ok(Self {
            client,
            list_url,
            registry,
            snapshot: RwLock::new(None),
            fetch_lock: Mutex::new(()),
            install_lock: Mutex::new(()),
        })
    }

    pub fn list_url(&self) -> &Url {
        &self.list_url
    }

    pub async fn list(
        &self,
        force_refresh: bool,
    ) -> Result<Vec<AvailableSource>, SourceCatalogError> {
        let list = self.fetch_list(force_refresh).await?;
        let installed = self.registry.list().await;
        Ok(list
            .sources
            .iter()
            .map(|entry| available_source(entry, &list.url, &installed))
            .collect())
    }

    pub async fn install(&self, source_id: &str) -> Result<InstalledSource, SourceCatalogError> {
        let _install = self.install_lock.lock().await;
        let list = self.fetch_list(false).await?;
        let entry = list
            .sources
            .iter()
            .find(|entry| entry.id == source_id)
            .ok_or_else(|| SourceCatalogError::SourceNotFound(source_id.into()))?;
        if let Some(installed) = self.registry.get(source_id).await {
            if installed.manifest.info.version >= entry.version {
                return Ok(InstalledSource::from(installed.as_ref()));
            }
        }
        let url = entry
            .resolved_download_url(&list.url)
            .ok_or_else(|| SourceCatalogError::DownloadUrl(source_id.into()))?;
        validate_download_url(&url, &list.url)?;

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|error| SourceCatalogError::Request(error.to_string()))?
            .error_for_status()
            .map_err(|error| SourceCatalogError::Request(error.to_string()))?;
        if let Some(length) = response.content_length() {
            if length > MAX_SOURCE_PACKAGE_BYTES as u64 {
                return Err(SourceCatalogError::PackageTooLarge);
            }
        }
        let mut bytes = Vec::new();
        let mut response = response;
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| SourceCatalogError::Request(error.to_string()))?
        {
            if bytes.len().saturating_add(chunk.len()) > MAX_SOURCE_PACKAGE_BYTES {
                return Err(SourceCatalogError::PackageTooLarge);
            }
            bytes.extend_from_slice(&chunk);
        }

        let expected_id = source_id.to_string();
        let (package, bytes) = tokio::task::spawn_blocking(move || {
            let package = SourcePackage::from_bytes(&bytes)?;
            CompiledSource::compile(&package)?.instantiate()?;
            Ok::<_, SourceCatalogError>((package, bytes))
        })
        .await
        .map_err(|error| SourceCatalogError::Worker(error.to_string()))??;
        let actual_id = package.manifest.info.id.clone();
        if actual_id != expected_id {
            return Err(SourceCatalogError::SourceIdMismatch {
                expected: expected_id,
                actual: actual_id,
            });
        }
        self.registry
            .install_parsed(package, bytes)
            .await
            .map_err(Into::into)
    }

    async fn fetch_list(&self, force_refresh: bool) -> Result<SourceList, SourceCatalogError> {
        if !force_refresh {
            if let Some(snapshot) = self.snapshot.read().await.as_ref() {
                if snapshot.fetched_at.elapsed() < CATALOG_CACHE_TTL {
                    return Ok(snapshot.list.clone());
                }
            }
        }
        let _fetch = self.fetch_lock.lock().await;
        if !force_refresh {
            if let Some(snapshot) = self.snapshot.read().await.as_ref() {
                if snapshot.fetched_at.elapsed() < CATALOG_CACHE_TTL {
                    return Ok(snapshot.list.clone());
                }
            }
        }
        let response = self
            .client
            .get(self.list_url.clone())
            .send()
            .await
            .map_err(|error| SourceCatalogError::Request(error.to_string()))?
            .error_for_status()
            .map_err(|error| SourceCatalogError::Request(error.to_string()))?;
        if response
            .content_length()
            .is_some_and(|length| length > MAX_SOURCE_LIST_BYTES as u64)
        {
            return Err(SourceCatalogError::ListTooLarge);
        }
        let mut bytes = Vec::new();
        let mut response = response;
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| SourceCatalogError::Request(error.to_string()))?
        {
            if bytes.len().saturating_add(chunk.len()) > MAX_SOURCE_LIST_BYTES {
                return Err(SourceCatalogError::ListTooLarge);
            }
            bytes.extend_from_slice(&chunk);
        }
        let list = SourceList::parse(self.list_url.clone(), &bytes)?;
        *self.snapshot.write().await = Some(CatalogSnapshot {
            fetched_at: tokio::time::Instant::now(),
            list: list.clone(),
        });
        Ok(list)
    }
}

fn available_source(
    entry: &SourceListEntry,
    list_url: &Url,
    installed: &[InstalledSource],
) -> AvailableSource {
    AvailableSource {
        id: entry.id.clone(),
        name: entry.name.clone(),
        version: entry.version,
        icon_url: entry.resolved_icon_url(list_url).map(|url| url.to_string()),
        download_url: entry
            .resolved_download_url(list_url)
            .filter(|url| validate_download_url(url, list_url).is_ok())
            .map(|url| url.to_string()),
        languages: entry.resolved_languages(),
        content_rating: entry.resolved_content_rating(),
        installed_version: installed
            .iter()
            .find(|source| source.info.id == entry.id)
            .map(|source| source.info.version),
    }
}

fn validate_download_url(url: &Url, list_url: &Url) -> Result<(), SourceCatalogError> {
    if url.scheme() != "https" {
        return Err(SourceCatalogError::DownloadUrl(url.to_string()));
    }
    if url.host_str() != list_url.host_str() {
        return Err(SourceCatalogError::DownloadUrl(url.to_string()));
    }
    if url.username() != "" || url.password().is_some() || url.port().is_some() {
        return Err(SourceCatalogError::DownloadUrl(url.to_string()));
    }
    Ok(())
}

fn validate_catalog_url(url: &Url) -> Result<(), SourceCatalogError> {
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
    {
        return Err(SourceCatalogError::DownloadUrl(
            "source list URL must be an HTTPS URL without credentials or a custom port".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_catalog_url, validate_download_url};
    use reqwest::Url;

    #[test]
    fn restricts_downloads_to_the_https_catalog_host() {
        let base = Url::parse("https://example.com/sources/index.min.json").unwrap();
        assert!(validate_download_url(
            &Url::parse("https://example.com/sources/aix/example.aix").unwrap(),
            &base
        )
        .is_ok());
        assert!(validate_download_url(
            &Url::parse("http://example.com/sources/example.aix").unwrap(),
            &base
        )
        .is_err());
        assert!(validate_download_url(
            &Url::parse("https://cdn.example.com/example.aix").unwrap(),
            &base
        )
        .is_err());
    }

    #[test]
    fn rejects_catalog_credentials_and_custom_ports() {
        assert!(
            validate_catalog_url(&Url::parse("http://example.com/index.json").unwrap()).is_err()
        );
        assert!(validate_catalog_url(
            &Url::parse("https://user:pass@example.com/index.json").unwrap()
        )
        .is_err());
        assert!(
            validate_catalog_url(&Url::parse("https://example.com:8443/index.json").unwrap())
                .is_err()
        );
    }
}
