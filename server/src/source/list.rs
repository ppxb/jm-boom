use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceListEntry {
    pub id: String,
    pub name: String,
    pub version: u32,
    #[serde(default, alias = "iconURL")]
    pub icon_url: Option<String>,
    #[serde(default, alias = "downloadURL")]
    pub download_url: Option<String>,
    #[serde(default)]
    pub languages: Option<Vec<String>>,
    #[serde(default, alias = "contentRating")]
    pub content_rating: Option<u8>,
    #[serde(default, alias = "altNames")]
    pub alt_names: Option<Vec<String>>,
    #[serde(default, alias = "baseURL")]
    pub base_url: Option<String>,
    #[serde(default, alias = "minAppVersion")]
    pub min_app_version: Option<String>,
    #[serde(default, alias = "maxAppVersion")]
    pub max_app_version: Option<String>,
    #[serde(default)]
    pub lang: Option<String>,
    #[serde(default)]
    pub nsfw: Option<u8>,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

impl SourceListEntry {
    pub fn resolved_download_url(&self, base_url: &Url) -> Option<Url> {
        let base_url = directory_url(base_url);
        if let Some(url) = self.download_url.as_deref() {
            return base_url.join(url).ok();
        }
        self.file
            .as_deref()
            .and_then(|file| base_url.join(&format!("sources/{file}")).ok())
    }

    pub fn resolved_icon_url(&self, base_url: &Url) -> Option<Url> {
        let base_url = directory_url(base_url);
        if let Some(url) = self.icon_url.as_deref() {
            return base_url.join(url).ok();
        }
        self.icon
            .as_deref()
            .and_then(|icon| base_url.join(&format!("icons/{icon}")).ok())
    }

    pub fn resolved_languages(&self) -> Vec<String> {
        self.languages
            .clone()
            .or_else(|| self.lang.clone().map(|lang| vec![lang]))
            .unwrap_or_default()
    }

    pub fn resolved_content_rating(&self) -> u8 {
        self.content_rating.or(self.nsfw).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceListDocument {
    pub name: String,
    #[serde(default, alias = "feedbackURL")]
    pub feedback_url: Option<String>,
    pub sources: Vec<SourceListEntry>,
}

#[derive(Debug, Clone)]
pub struct SourceList {
    pub url: Url,
    pub name: String,
    pub feedback_url: Option<Url>,
    pub sources: Vec<SourceListEntry>,
    pub legacy_format: bool,
}

#[derive(Debug, Error)]
pub enum SourceListError {
    #[error("invalid source list JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("source list URL must be absolute: {0}")]
    RelativeUrl(Url),
}

impl SourceList {
    pub fn parse(url: Url, bytes: &[u8]) -> Result<Self, SourceListError> {
        if url.scheme().is_empty() || url.host_str().is_none() {
            return Err(SourceListError::RelativeUrl(url));
        }

        if let Ok(document) = serde_json::from_slice::<SourceListDocument>(bytes) {
            let base_url = directory_url(&url);
            return Ok(Self {
                feedback_url: document
                    .feedback_url
                    .as_deref()
                    .and_then(|value| base_url.join(value).ok()),
                url,
                name: document.name,
                sources: document.sources,
                legacy_format: false,
            });
        }

        let sources = serde_json::from_slice::<Vec<SourceListEntry>>(bytes)?;
        Ok(Self {
            url,
            name: "External Sources".into(),
            feedback_url: None,
            sources,
            legacy_format: true,
        })
    }
}

fn directory_url(url: &Url) -> Url {
    if url.path().ends_with('/') {
        return url.clone();
    }

    let mut directory = url.clone();
    let path = url.path();
    let directory_path = path
        .rsplit_once('/')
        .map(|(parent, _)| format!("{parent}/"))
        .unwrap_or_else(|| "/".into());
    directory.set_path(&directory_path);
    directory
}

#[cfg(test)]
mod tests {
    use super::SourceList;
    use reqwest::Url;

    #[test]
    fn parses_official_array_format_and_resolves_relative_urls() {
        let url = Url::parse("https://example.com/sources/index.min.json").unwrap();
        let list = SourceList::parse(
            url.clone(),
            br#"[{"id":"zh.example","name":"Example","version":2,"downloadURL":"sources/example.aix","iconURL":"icons/example.png"}]"#,
        )
        .unwrap();
        assert!(list.legacy_format);
        let source = &list.sources[0];
        assert_eq!(
            source.resolved_download_url(&url).unwrap().as_str(),
            "https://example.com/sources/sources/example.aix"
        );
        assert_eq!(
            source.resolved_icon_url(&url).unwrap().as_str(),
            "https://example.com/sources/icons/example.png"
        );
    }

    #[test]
    fn parses_document_format() {
        let url = Url::parse("https://example.com/list.json").unwrap();
        let list = SourceList::parse(
            url,
            br#"{"name":"Community","feedbackURL":"feedback","sources":[]}"#,
        )
        .unwrap();
        assert!(!list.legacy_format);
        assert_eq!(list.name, "Community");
        assert_eq!(
            list.feedback_url.unwrap().as_str(),
            "https://example.com/feedback"
        );
    }
}
