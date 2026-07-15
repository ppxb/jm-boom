use crate::{
    http_error::HttpError,
    source::{AvailableSource, InstalledSource, SourceCatalogError, SourceRegistryError},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CatalogQuery {
    #[serde(default)]
    pub refresh: bool,
}

pub async fn list(State(app): State<AppState>) -> Json<Vec<InstalledSource>> {
    Json(app.sources.list().await)
}

pub async fn catalog(
    State(app): State<AppState>,
    Query(query): Query<CatalogQuery>,
) -> Result<Json<Vec<AvailableSource>>, HttpError> {
    app.source_catalog
        .list(query.refresh)
        .await
        .map(Json)
        .map_err(catalog_error)
}

pub async fn install(
    State(app): State<AppState>,
    Path(source_id): Path<String>,
) -> Result<Json<InstalledSource>, HttpError> {
    app.source_catalog
        .install(&source_id)
        .await
        .map(Json)
        .map_err(catalog_error)
}

fn catalog_error(error: SourceCatalogError) -> HttpError {
    match error {
        SourceCatalogError::SourceNotFound(source_id) => HttpError::new(
            StatusCode::NOT_FOUND,
            format!("源目录中不存在: {source_id}"),
            false,
        ),
        SourceCatalogError::Registry(SourceRegistryError::Downgrade { .. }) => {
            HttpError::new(StatusCode::CONFLICT, error.to_string(), false)
        }
        SourceCatalogError::Request(message) => {
            HttpError::new(StatusCode::BAD_GATEWAY, message, true)
        }
        SourceCatalogError::Worker(message) => HttpError::internal(message),
        SourceCatalogError::List(_)
        | SourceCatalogError::ListTooLarge
        | SourceCatalogError::DownloadUrl(_)
        | SourceCatalogError::PackageTooLarge
        | SourceCatalogError::SourceIdMismatch { .. }
        | SourceCatalogError::Package(_)
        | SourceCatalogError::Runtime(_) => {
            HttpError::new(StatusCode::BAD_GATEWAY, error.to_string(), false)
        }
        SourceCatalogError::Registry(SourceRegistryError::Package(_))
        | SourceCatalogError::Registry(SourceRegistryError::NotInstalled(_))
        | SourceCatalogError::Registry(SourceRegistryError::Storage(_)) => {
            HttpError::internal(error.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::StatusCode, response::IntoResponse};

    use super::{catalog_error, CatalogQuery};
    use crate::source::SourceCatalogError;

    #[test]
    fn parses_catalog_refresh_query() {
        let query: CatalogQuery =
            serde_json::from_str(r#"{"refresh":true}"#).expect("parse catalog query");
        assert!(query.refresh);
    }

    #[tokio::test]
    async fn maps_unknown_catalog_source_to_not_found() {
        let response =
            catalog_error(SourceCatalogError::SourceNotFound("zh.missing".into())).into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read response body");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("decode response");
        assert_eq!(value["retryable"], false);
    }
}
