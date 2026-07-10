use aes::Aes256;
use base64::prelude::{Engine as _, BASE64_STANDARD};
use ecb::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyInit};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

mod auth;
mod client;
mod codec;
mod comic;
mod error;
mod home;
mod models;
mod search;
mod serde_ext;
mod setting;
mod user;

pub(crate) use auth::*;
pub(crate) use client::*;
pub(crate) use codec::*;
pub(crate) use comic::*;
pub(crate) use error::*;
pub(crate) use home::*;
pub(crate) use models::*;
pub(crate) use search::*;
pub(crate) use serde_ext::*;
pub(crate) use setting::*;
pub(crate) use user::*;

pub(crate) type ApiResult<T> = Result<T, ApiError>;

type Aes256EcbDec = ecb::Decryptor<Aes256>;

const API_VERSION: &str = "2.0.20";
const API_SECRET: &str = "185Hcomic3PAPP7R";
const DEFAULT_API_ENDPOINT: &str = FALLBACK_API_ENDPOINTS[0];
const FALLBACK_API_ENDPOINTS: [&str; 2] = ["https://www.cdnhjk.net", "https://www.cdnhth.club"];
const HOST_CONFIG_AES_SEED: &str = "diosfjckwpqpdfjkvnqQjsik";
const HOST_CONFIG_URLS: [&str; 2] = [
    "https://rup4a04-c02.tos-cn-hongkong.bytepluses.com/newsvr-2025.txt",
    "https://rup4a04-c01.tos-ap-southeast-1.bytepluses.com/newsvr-2025.txt",
];
const UNSUPPORTED_HOME_SECTION_TITLES: [&str; 4] = ["禁漫小说", "禁漫书库", "禁漫書庫", "禁漫小說"];
const HOME_SECTION_PREVIEW_LIMIT: usize = 8;
const HOME_SECTION_LIST_PAGE_SIZE: usize = 20;
const SEARCH_PAGE_SIZE: usize = 80;
const JM_PLUGIN_ID: &str = "bf99008d-010b-4f17-ac7c-61a9b57dc3d9";
static IMG_HOST_CACHE: OnceLock<Mutex<HashMap<String, ImgHostCacheEntry>>> = OnceLock::new();
static JWT_TOKEN: OnceLock<Mutex<Option<String>>> = OnceLock::new();

#[derive(Clone, Debug)]
pub(crate) struct ImgHostCacheEntry {
    pub(crate) value: String,
    pub(crate) expires_at: u64,
}
pub(crate) fn resolve_api_endpoint(endpoint: Option<String>) -> ApiResult<String> {
    let Some(endpoint) = endpoint else {
        return Ok(DEFAULT_API_ENDPOINT.to_string());
    };
    normalize_api_endpoint(&endpoint)
}

fn normalize_api_endpoint(endpoint: &str) -> ApiResult<String> {
    let endpoint = endpoint.trim().trim_end_matches('/');

    if endpoint.is_empty() {
        return Ok(DEFAULT_API_ENDPOINT.to_string());
    }

    let endpoint = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        endpoint.to_string()
    } else {
        format!("https://{endpoint}")
    };
    let url = reqwest::Url::parse(&endpoint).map_err(|error| {
        ApiError::new(
            ApiErrorKind::UnsupportedEndpoint,
            format!("Invalid API endpoint {endpoint}: {error}"),
        )
    })?;

    match url.scheme() {
        "http" | "https" if url.host_str().is_some() => {
            let host = url.host_str().ok_or_else(|| {
                ApiError::new(
                    ApiErrorKind::UnsupportedEndpoint,
                    format!("Missing host in endpoint: {endpoint}"),
                )
            })?;
            let mut normalized = format!("{}://{}", url.scheme(), host);
            if let Some(port) = url.port() {
                normalized.push_str(&format!(":{port}"));
            }
            Ok(normalized)
        }
        _ => Err(ApiError::new(
            ApiErrorKind::UnsupportedEndpoint,
            format!("Unsupported API endpoint: {endpoint}"),
        )),
    }
}

fn cover_image_url(img_host: Option<&str>, comic_id: &str) -> Option<String> {
    let img_host = img_host?.trim().trim_end_matches('/');

    if img_host.is_empty() {
        return None;
    }

    Some(format!("{img_host}/media/albums/{comic_id}_3x4.jpg"))
}

fn user_avatar_url(img_host: Option<&str>, photo: &str) -> Option<String> {
    let photo = photo.trim();

    if photo.is_empty() {
        return None;
    }

    if photo.starts_with("http://") || photo.starts_with("https://") {
        return Some(photo.to_string());
    }

    let img_host = img_host?.trim().trim_end_matches('/');

    if img_host.is_empty() {
        return None;
    }

    if photo.starts_with('/') {
        Some(format!("{img_host}{photo}"))
    } else {
        Some(format!("{img_host}/media/users/{photo}"))
    }
}

fn response_preview(value: &str) -> String {
    value
        .chars()
        .take(180)
        .collect::<String>()
        .replace('\n', "\\n")
}
