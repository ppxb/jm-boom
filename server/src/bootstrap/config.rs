use axum::http::{header, HeaderName, HeaderValue, Method};
use std::path::PathBuf;
use tower_http::cors::{AllowOrigin, CorsLayer};

pub struct AppConfig {
    pub data_dir: PathBuf,
    pub static_dir: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            static_dir: std::env::var("JM_BOOM_STATIC_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("./static")),
        }
    }

    pub fn cors_layer(&self) -> anyhow::Result<Option<CorsLayer>> {
        let configured = match std::env::var("JM_BOOM_CORS_ORIGINS") {
            Ok(value) => value,
            Err(std::env::VarError::NotPresent) => return Ok(None),
            Err(error) => return Err(anyhow::anyhow!("invalid JM_BOOM_CORS_ORIGINS: {error}")),
        };
        let origins = parse_cors_origins(&configured)?;

        if origins.is_empty() {
            return Ok(None);
        }

        tracing::info!(origins = ?origins, "cross-origin access enabled");
        Ok(Some(
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(origins))
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([
                    header::CONTENT_TYPE,
                    HeaderName::from_static("x-jm-boom-image-priority"),
                ]),
        ))
    }
}

fn parse_cors_origins(configured: &str) -> anyhow::Result<Vec<HeaderValue>> {
    let mut origins = Vec::new();

    for value in configured
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let url = reqwest::Url::parse(value)
            .map_err(|error| anyhow::anyhow!("invalid CORS origin {value:?}: {error}"))?;
        let is_http = matches!(url.scheme(), "http" | "https");
        let is_origin_only = url.path() == "/"
            && url.query().is_none()
            && url.fragment().is_none()
            && url.username().is_empty()
            && url.password().is_none();

        if !is_http || url.host_str().is_none() || !is_origin_only {
            return Err(anyhow::anyhow!(
                "CORS origin must be an http(s) origin without path, query, credentials, or fragment: {value:?}"
            ));
        }

        let origin = HeaderValue::from_str(&url.origin().ascii_serialization())?;
        if !origins.contains(&origin) {
            origins.push(origin);
        }
    }

    Ok(origins)
}

#[cfg(test)]
mod tests {
    use super::parse_cors_origins;

    #[test]
    fn parses_and_deduplicates_http_origins() {
        let origins =
            parse_cors_origins("http://localhost:5173, https://example.com, http://localhost:5173")
                .expect("parse CORS origins");
        assert_eq!(origins.len(), 2);
        assert_eq!(origins[0], "http://localhost:5173");
        assert_eq!(origins[1], "https://example.com");
    }

    #[test]
    fn rejects_non_origin_values() {
        assert!(parse_cors_origins("https://example.com/path").is_err());
        assert!(parse_cors_origins("file:///tmp/app").is_err());
    }
}
