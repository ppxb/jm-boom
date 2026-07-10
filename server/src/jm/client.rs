use super::{auth::JmAuth, crypto, error::JmError, JmResult};
use once_cell::sync::OnceCell;
use reqwest::{Client, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize};
use std::{sync::Mutex, time::Duration};

static HTTP_CLIENT: OnceCell<Mutex<Option<Client>>> = OnceCell::new();

const USER_AGENT: &str = "Mozilla/5.0 (Linux; Android 13; jm-boom Build/TQ1A.230305.002; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/120.0.6099.230 Mobile Safari/537.36";

/// JM API client
pub struct JmClient {
    client: Client,
}

impl JmClient {
    /// Create a new JM client
    pub fn new() -> JmResult<Self> {
        Ok(Self {
            client: get_or_create_client()?,
        })
    }

    /// GET request to JM API
    pub async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        path: &str,
        query: &[(&str, String)],
    ) -> JmResult<T> {
        let auth = JmAuth::new();
        let url = format!("{endpoint}/{path}");

        let response = self
            .client
            .get(&url)
            .apply_jm_headers(&url, &auth)?
            .query(query)
            .send()
            .await
            .map_err(|e| JmError::Network(e.to_string()))?;

        decode_response(response, &url, &auth).await
    }
}

impl Default for JmClient {
    fn default() -> Self {
        Self::new().expect("Failed to create JM client")
    }
}

// Internal helpers

fn get_or_create_client() -> JmResult<Client> {
    let client_cell = HTTP_CLIENT.get_or_init(|| Mutex::new(None));
    let mut guard = client_cell
        .lock()
        .map_err(|e| JmError::Other(format!("Lock error: {e}")))?;

    if let Some(client) = guard.as_ref() {
        return Ok(client.clone());
    }

    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| JmError::Other(format!("Failed to create HTTP client: {e}")))?;

    *guard = Some(client.clone());
    Ok(client)
}

trait RequestBuilderExt {
    fn apply_jm_headers(self, url: &str, auth: &JmAuth) -> JmResult<RequestBuilder>;
}

impl RequestBuilderExt for RequestBuilder {
    fn apply_jm_headers(self, url: &str, auth: &JmAuth) -> JmResult<RequestBuilder> {
        let mut builder = self
            .header("accept", "application/json")
            .header("token", &auth.token)
            .header("tokenparam", &auth.tokenparam)
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
    auth: &JmAuth,
) -> JmResult<T> {
    if !response.status().is_success() {
        return Err(JmError::Http(format!(
            "HTTP {}: {url}",
            response.status()
        )));
    }

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
            let decrypted = crypto::decrypt_data(&encrypted, &auth.ts)?;
            serde_json::from_str(&decrypted).map_err(|e| JmError::Payload(e.to_string()))
        }
        value => serde_json::from_value(value).map_err(|e| JmError::Payload(e.to_string())),
    }
}
