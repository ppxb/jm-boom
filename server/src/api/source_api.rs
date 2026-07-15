use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    http_error::HttpError,
    source::{
        Chapter, FilterValue, Listing, Manga, MangaPageResult, Page, PageContent, PageContext,
        SourceRuntimeError, SourceServiceError,
    },
    AppState,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    pub query: Option<String>,
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default)]
    pub filters: Vec<FilterValue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRequest {
    pub manga: Manga,
    #[serde(default = "default_true")]
    pub needs_details: bool,
    #[serde(default = "default_true")]
    pub needs_chapters: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PagesRequest {
    pub manga: Manga,
    pub chapter: Chapter,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListingRequest {
    pub listing: Listing,
    #[serde(default = "default_page")]
    pub page: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SearchResponse {
    source_id: String,
    result: MangaPageResult,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DetailResponse {
    source_id: String,
    manga: Manga,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PagesResponse {
    source_id: String,
    pages: Vec<PageResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ListingResponse {
    source_id: String,
    result: MangaPageResult,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PageResponse {
    content: PageContentResponse,
    thumbnail: Option<String>,
    has_description: bool,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
enum PageContentResponse {
    Remote {
        url: String,
        context: Option<PageContext>,
    },
    Text {
        text: String,
    },
    Archive {
        url: String,
        path: String,
    },
}

pub(super) async fn search(
    State(app): State<AppState>,
    Path(source_id): Path<String>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, HttpError> {
    if request.page < 1 {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "页码必须大于 0",
            false,
        ));
    }
    let result = app
        .source_service
        .search(&source_id, request.query, request.page, request.filters)
        .await
        .map_err(source_error)?;
    Ok(Json(SearchResponse { source_id, result }))
}

pub(super) async fn update(
    State(app): State<AppState>,
    Path(source_id): Path<String>,
    Json(request): Json<UpdateRequest>,
) -> Result<Json<DetailResponse>, HttpError> {
    let manga = app
        .source_service
        .update_manga(
            &source_id,
            request.manga,
            request.needs_details,
            request.needs_chapters,
        )
        .await
        .map_err(source_error)?;
    Ok(Json(DetailResponse { source_id, manga }))
}

pub(super) async fn pages(
    State(app): State<AppState>,
    Path(source_id): Path<String>,
    Json(request): Json<PagesRequest>,
) -> Result<Json<PagesResponse>, HttpError> {
    let pages = app
        .source_service
        .get_pages(&source_id, request.manga, request.chapter)
        .await
        .map_err(source_error)?
        .into_iter()
        .map(PageResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(PagesResponse { source_id, pages }))
}

pub(super) async fn listing(
    State(app): State<AppState>,
    Path(source_id): Path<String>,
    Json(request): Json<ListingRequest>,
) -> Result<Json<ListingResponse>, HttpError> {
    if request.page < 1 {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "页码必须大于 0",
            false,
        ));
    }
    let result = app
        .source_service
        .get_listing(&source_id, request.listing, request.page)
        .await
        .map_err(source_error)?;
    Ok(Json(ListingResponse { source_id, result }))
}

impl TryFrom<Page> for PageResponse {
    type Error = HttpError;

    fn try_from(page: Page) -> Result<Self, Self::Error> {
        let content = match page.content {
            PageContent::Url(url, context) => PageContentResponse::Remote { url, context },
            PageContent::Text(text) => PageContentResponse::Text { text },
            PageContent::Zip(url, path) => PageContentResponse::Archive { url, path },
            PageContent::Image(_) => {
                return Err(HttpError::new(
                    StatusCode::NOT_IMPLEMENTED,
                    "当前漫画源返回了内存图片，服务端尚未提供图片描述符桥接",
                    false,
                ));
            }
        };
        Ok(Self {
            content,
            thumbnail: page.thumbnail,
            has_description: page.has_description,
            description: page.description,
        })
    }
}

fn source_error(error: SourceServiceError) -> HttpError {
    match error {
        SourceServiceError::NotInstalled(source_id) => HttpError::new(
            StatusCode::NOT_FOUND,
            format!("漫画源未安装: {source_id}"),
            false,
        ),
        SourceServiceError::UnsupportedInMemoryPage => HttpError::new(
            StatusCode::NOT_IMPLEMENTED,
            "当前漫画源返回了内存图片，服务端尚未提供图片描述符桥接",
            false,
        ),
        SourceServiceError::ShuttingDown => {
            HttpError::new(StatusCode::SERVICE_UNAVAILABLE, "漫画源服务正在关闭", true)
        }
        SourceServiceError::Worker(message) => HttpError::internal(message),
        SourceServiceError::Runtime(runtime) => runtime_error(runtime),
    }
}

fn runtime_error(error: SourceRuntimeError) -> HttpError {
    match error {
        SourceRuntimeError::Source(-2) => {
            HttpError::new(StatusCode::NOT_IMPLEMENTED, "漫画源未实现此能力", false)
        }
        SourceRuntimeError::Source(-3) => {
            HttpError::new(StatusCode::BAD_GATEWAY, "漫画源网络请求失败", true)
        }
        SourceRuntimeError::Source(code) => HttpError::new(
            StatusCode::BAD_GATEWAY,
            format!("漫画源返回错误码: {code}"),
            false,
        ),
        SourceRuntimeError::SourceMessage(message) => {
            HttpError::new(StatusCode::BAD_GATEWAY, message, true)
        }
        SourceRuntimeError::Execution(message) => {
            HttpError::new(StatusCode::BAD_GATEWAY, message, true)
        }
        SourceRuntimeError::Compile(message)
        | SourceRuntimeError::Instantiate(message)
        | SourceRuntimeError::Export(message)
        | SourceRuntimeError::Host(message)
        | SourceRuntimeError::InvalidResult(message) => HttpError::internal(message),
        SourceRuntimeError::Encode(error) => HttpError::internal(error.to_string()),
    }
}

fn default_page() -> i32 {
    1
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::StatusCode, response::IntoResponse};

    use super::{source_error, PageContentResponse, PageResponse, SearchRequest, SearchResponse};
    use crate::source::{MangaPageResult, SourceServiceError};

    #[test]
    fn search_request_uses_stable_defaults() {
        let request: SearchRequest =
            serde_json::from_value(serde_json::json!({})).expect("deserialize search request");
        assert_eq!(request.page, 1);
        assert!(request.query.is_none());
        assert!(request.filters.is_empty());
    }

    #[test]
    fn serializes_source_and_page_contracts_in_camel_case() {
        let search = serde_json::to_value(SearchResponse {
            source_id: "zh.example".into(),
            result: MangaPageResult {
                entries: Vec::new(),
                has_next_page: false,
            },
        })
        .expect("serialize search response");
        assert_eq!(search["sourceId"], "zh.example");
        assert_eq!(search["result"]["hasNextPage"], false);

        let page = serde_json::to_value(PageResponse {
            content: PageContentResponse::Remote {
                url: "https://example.com/1.jpg".into(),
                context: None,
            },
            thumbnail: None,
            has_description: false,
            description: None,
        })
        .expect("serialize page response");
        assert_eq!(page["content"]["type"], "remote");
        assert_eq!(page["content"]["data"]["url"], "https://example.com/1.jpg");
        assert_eq!(page["hasDescription"], false);
    }

    #[tokio::test]
    async fn maps_missing_source_to_shared_not_found_error() {
        let response =
            source_error(SourceServiceError::NotInstalled("zh.missing".into())).into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read response body");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("decode response body");
        assert_eq!(value["retryable"], false);
    }
}
