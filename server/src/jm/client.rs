use super::{
    crypto,
    error::JmError,
    models::{
        Comic, ComicDetailPayload, HomeSection, HomeSectionPayload, SearchPayload, SearchResult,
    },
    signature::JmRequestSignature,
    JmResult,
};
use crate::domain::comic::{ComicChapter, ComicDetail, RelatedComic};
use once_cell::sync::OnceCell;
use reqwest::{Client, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;

static HTTP_CLIENT: OnceCell<Client> = OnceCell::new();

const USER_AGENT: &str = "Mozilla/5.0 (Linux; Android 13; jm-boom Build/TQ1A.230305.002; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/120.0.6099.230 Mobile Safari/537.36";

/// JM API client
pub struct JmClient {
    pub(crate) client: Client,
    jwt_token: Arc<RwLock<Option<String>>>,
}

impl JmClient {
    /// Create a new JM client
    pub fn new() -> JmResult<Self> {
        Ok(Self {
            client: get_or_create_client()?,
            jwt_token: Arc::new(RwLock::new(None)),
        })
    }

    /// GET request to JM API
    pub async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        path: &str,
        query: &[(&str, String)],
    ) -> JmResult<T> {
        let signature = JmRequestSignature::new();
        let url = format!("{endpoint}/{path}");

        let mut request = self
            .client
            .get(&url)
            .apply_request_signature(&url, &signature)?
            .query(query);
        if let Some(jwt) = self.jwt_token.read().await.as_ref() {
            request = request.header("Authorization", format!("Bearer {jwt}"));
        }
        let response = request
            .send()
            .await
            .map_err(|e| JmError::Network(e.to_string()))?;

        decode_response(response, &url, &signature).await
    }

    pub async fn post_form<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        path: &str,
        fields: &[(String, String)],
        authenticated: bool,
    ) -> JmResult<T> {
        let signature = JmRequestSignature::new();
        let url = format!("{endpoint}/{path}");
        let mut request = self
            .client
            .post(&url)
            .apply_request_signature(&url, &signature)?
            .form(fields);
        if authenticated {
            if let Some(jwt) = self.jwt_token.read().await.as_ref() {
                request = request.header("Authorization", format!("Bearer {jwt}"));
            }
        }
        let response = request
            .send()
            .await
            .map_err(|error| JmError::Network(error.to_string()))?;

        decode_response(response, &url, &signature).await
    }

    pub async fn set_jwt_token(&self, token: Option<String>) {
        *self.jwt_token.write().await = token;
    }

    /// Search comics
    pub async fn search(
        &self,
        endpoint: &str,
        keyword: &str,
        page: u32,
        order: &str,
    ) -> JmResult<SearchResult> {
        let payload: SearchPayload = self
            .get(
                endpoint,
                "search",
                &[
                    ("search_query", keyword.to_string()),
                    ("page", page.to_string()),
                    ("o", order.to_string()),
                ],
            )
            .await?;

        Ok(SearchResult {
            total: payload.total,
            content: payload.content.into_iter().map(Comic::from).collect(),
            redirect_aid: payload.redirect_aid,
        })
    }

    /// Get comic detail
    pub async fn get_comic_detail(&self, endpoint: &str, comic_id: &str) -> JmResult<ComicDetail> {
        let payload: ComicDetailPayload = self
            .get(endpoint, "album", &[("id", comic_id.to_string())])
            .await?;

        Ok(ComicDetail {
            id: payload.id,
            title: payload.name,
            description: payload.description,
            image: payload.image,
            authors: payload.author,
            tags: payload.tags,
            actors: payload.actors,
            works: payload.works,
            total_views: payload.total_views,
            likes: payload.likes,
            comment_count: payload.comment_total,
            related_comics: payload
                .related_list
                .into_iter()
                .map(|related| RelatedComic {
                    id: related.id,
                    title: related.name,
                    author: related.author,
                    image: related.image,
                })
                .collect(),
            chapters: payload
                .series
                .into_iter()
                .map(|chapter| ComicChapter {
                    id: chapter.id,
                    title: chapter.name,
                    sort: chapter.sort,
                })
                .collect(),
        })
    }

    /// Get home feed sections
    pub async fn get_home_feed(&self, endpoint: &str) -> JmResult<Vec<HomeSection>> {
        let sections: Vec<HomeSectionPayload> = self.get(endpoint, "promote", &[]).await?;

        Ok(sections
            .into_iter()
            .filter(|s| !is_unsupported_section(&s.title))
            .map(|s| HomeSection {
                id: s.id,
                title: s.title,
                slug: s.slug,
                section_type: s.section_type,
                filter_val: s.filter_val,
                content: s
                    .content
                    .into_iter()
                    .take(20) // limit preview items
                    .map(Comic::from)
                    .collect(),
            })
            .collect())
    }
}

impl Default for JmClient {
    fn default() -> Self {
        Self::new().expect("Failed to create JM client")
    }
}

// Helper to filter unsupported sections
fn is_unsupported_section(title: &str) -> bool {
    let title = title.trim();
    matches!(
        title,
        "禁漫小说"
            | "禁漫书库"
            | "禁漫書庫"
            | "禁漫小說"
            | "最新成人APP"
            | "大人気エロ同人"
            | "人気のエロ動画"
            | "オススメ動画サイト"
    )
}

#[cfg(test)]
mod tests {
    use super::is_unsupported_section;

    #[test]
    fn filters_unsupported_home_sections() {
        for title in ["禁漫小说", "禁漫书库", "禁漫書庫", "禁漫小說"] {
            assert!(is_unsupported_section(title));
        }

        assert!(!is_unsupported_section("每周推荐"));
    }
}

// Internal helpers

fn get_or_create_client() -> JmResult<Client> {
    let client = HTTP_CLIENT.get_or_try_init(|| {
        Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| JmError::Other(format!("Failed to create HTTP client: {e}")))
    })?;
    Ok(client.clone())
}

trait RequestBuilderExt {
    fn apply_request_signature(
        self,
        url: &str,
        signature: &JmRequestSignature,
    ) -> JmResult<RequestBuilder>;
}

impl RequestBuilderExt for RequestBuilder {
    fn apply_request_signature(
        self,
        url: &str,
        signature: &JmRequestSignature,
    ) -> JmResult<RequestBuilder> {
        let mut builder = self
            .header("accept", "application/json")
            .header("token", &signature.token)
            .header("tokenparam", &signature.tokenparam)
            .header("user-agent", USER_AGENT);

        if let Some(host) = extract_host(url) {
            builder = builder.header("Host", host);
        }

        Ok(builder)
    }
}

fn extract_host(url: &str) -> Option<String> {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_string))
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    code: i32,
    data: Option<serde_json::Value>,
    #[serde(rename = "errorMsg")]
    error_msg: Option<String>,
    #[serde(skip)]
    _phantom: std::marker::PhantomData<T>,
}

async fn decode_response<T: DeserializeOwned>(
    response: reqwest::Response,
    url: &str,
    signature: &JmRequestSignature,
) -> JmResult<T> {
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| JmError::Network(e.to_string()))?;

    let body = body.trim();
    if body.is_empty() {
        return Err(JmError::Empty);
    }

    let envelope: ApiResponse<T> =
        serde_json::from_str(body).map_err(|e| JmError::Payload(e.to_string()))?;

    if !status.is_success() {
        return Err(JmError::Api(
            envelope
                .error_msg
                .unwrap_or_else(|| format!("HTTP {}: {url}", status)),
        ));
    }

    if envelope.code != 200 {
        return Err(JmError::Api(
            envelope
                .error_msg
                .unwrap_or_else(|| format!("API error code: {}", envelope.code)),
        ));
    }

    let data = envelope.data.ok_or(JmError::MissingData)?;

    match data {
        serde_json::Value::String(encrypted) => {
            let decrypted = crypto::decrypt_data(&encrypted, &signature.ts)?;
            serde_json::from_str(&decrypted).map_err(|e| JmError::Payload(e.to_string()))
        }
        value => serde_json::from_value(value).map_err(|e| JmError::Payload(e.to_string())),
    }
}
