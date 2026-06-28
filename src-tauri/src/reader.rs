use crate::api::{build_http_client, resolve_api_endpoint, ApiError, ApiErrorKind, ApiResult};
use crate::plugin_codec::decode_setting_payload;
use aes::Aes256;
use base64::prelude::{Engine as _, BASE64_STANDARD};
use ecb::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyInit};
use image::{DynamicImage, RgbImage};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex as AsyncMutex;
use tokio::task::JoinSet;

const DEFAULT_READER_CACHE_LIMIT_BYTES: u64 = 512 * 1024 * 1024;
const MIN_READER_CACHE_LIMIT_BYTES: u64 = 128 * 1024 * 1024;
const MAX_READER_CACHE_LIMIT_BYTES: u64 = 2048 * 1024 * 1024;
const JM_API_VERSION: &str = "2.0.20";
const JM_API_SECRET: &str = "185Hcomic3PAPP7R";
const JM_SCRAMBLE_ID: u32 = 220_980;
const SEED_MAP: [u32; 10] = [2, 4, 6, 8, 10, 12, 14, 16, 18, 20];
const SCRAMBLED_WEBP_QUALITY: f32 = 75.0;

static MANIFEST_CACHE: OnceLock<Mutex<HashMap<String, ReaderManifest>>> = OnceLock::new();
static PAGE_MATERIALIZE_LOCKS: OnceLock<Mutex<HashMap<PathBuf, Arc<AsyncMutex<()>>>>> =
    OnceLock::new();

#[derive(Debug, Clone)]
struct ReaderManifest {
    endpoint: String,
    read_id: String,
    read_id_number: u32,
    pages: Vec<ReaderPage>,
}

#[derive(Debug, Clone)]
struct ReaderPage {
    index: u32,
    page_name: String,
    source_url: String,
}

#[derive(Debug, Clone, Copy)]
enum ReaderPageMaterializeOrigin {
    Visible,
    ChapterCache,
}

impl ReaderPageMaterializeOrigin {
    fn as_str(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::ChapterCache => "chapter_cache",
        }
    }
}

#[derive(Debug, Deserialize)]
struct ReaderChapterPayload {
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    id: String,
    #[serde(default)]
    images: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ReaderChapterEnvelope {
    images: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ReaderApiEnvelope {
    #[serde(default)]
    code: i32,
    data: Option<serde_json::Value>,
    #[serde(default, rename = "errorMsg")]
    error_msg: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ComicReadManifestResult {
    pub endpoint: String,
    #[serde(rename = "readId")]
    pub read_id: String,
    #[serde(rename = "pageCount")]
    pub page_count: u32,
    #[serde(rename = "cacheLimitBytes")]
    pub cache_limit_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct ComicReadPageResult {
    #[serde(rename = "readId")]
    pub read_id: String,
    pub index: u32,
    pub path: String,
    pub width: u32,
    pub height: u32,
    #[serde(rename = "aspectRatio")]
    pub aspect_ratio: f32,
    #[serde(rename = "isCached")]
    pub is_cached: bool,
}

#[derive(Debug, Serialize)]
pub struct ComicReadChapterCacheResult {
    pub requested: u32,
    pub completed: u32,
}

#[derive(Debug, Serialize)]
pub struct ReaderCacheStatsResult {
    #[serde(rename = "cacheDir")]
    pub cache_dir: String,
    #[serde(rename = "totalBytes")]
    pub total_bytes: u64,
    #[serde(rename = "fileCount")]
    pub file_count: u32,
    #[serde(rename = "cacheLimitBytes")]
    pub cache_limit_bytes: u64,
    #[serde(rename = "cacheTrimBytes")]
    pub cache_trim_bytes: u64,
}

pub async fn get_comic_read_manifest(
    read_id: String,
    endpoint: Option<String>,
) -> ApiResult<ComicReadManifestResult> {
    let manifest = get_or_load_manifest(read_id, endpoint).await?;

    Ok(manifest.to_result())
}

pub async fn get_comic_read_page(
    app: &AppHandle,
    read_id: String,
    index: u32,
    endpoint: Option<String>,
    request_origin: Option<String>,
    cache_limit_bytes: Option<u64>,
) -> ApiResult<ComicReadPageResult> {
    let manifest = get_or_load_manifest(read_id, endpoint).await?;

    materialize_reader_page(
        app,
        &manifest,
        index,
        normalize_cache_limit(cache_limit_bytes),
        request_origin,
    )
    .await
}

pub async fn cache_comic_read_chapter(
    app: &AppHandle,
    read_id: String,
    endpoint: Option<String>,
    request_origin: Option<String>,
    cache_limit_bytes: Option<u64>,
) -> ApiResult<ComicReadChapterCacheResult> {
    let manifest = get_or_load_manifest(read_id, endpoint).await?;
    let cache_limit_bytes = normalize_cache_limit(cache_limit_bytes);
    let page_count = manifest.pages.len() as u32;

    if page_count == 0 {
        return Ok(ComicReadChapterCacheResult {
            requested: 0,
            completed: 0,
        });
    }

    let indexes = (0..page_count).collect::<Vec<_>>();
    let requested = indexes.len() as u32;
    let mut completed = 0;
    let concurrency = 5;
    let mut pending = VecDeque::from(indexes);
    let mut tasks = JoinSet::new();

    for _ in 0..concurrency {
        spawn_chapter_cache_task(
            &mut tasks,
            app,
            &manifest,
            &mut pending,
            cache_limit_bytes,
            request_origin.clone(),
        );
    }

    while let Some(result) = tasks.join_next().await {
        if matches!(result, Ok(true)) {
            completed += 1;
        }

        spawn_chapter_cache_task(
            &mut tasks,
            app,
            &manifest,
            &mut pending,
            cache_limit_bytes,
            request_origin.clone(),
        );
    }

    Ok(ComicReadChapterCacheResult {
        requested,
        completed,
    })
}

pub fn get_reader_cache_stats(
    app: &AppHandle,
    cache_limit_bytes: Option<u64>,
) -> ApiResult<ReaderCacheStatsResult> {
    let cache_root = reader_cache_root(app)?;
    let cache_limit_bytes = normalize_cache_limit(cache_limit_bytes);
    reader_cache_stats(cache_root, cache_limit_bytes)
}

pub fn clear_reader_cache(
    app: &AppHandle,
    cache_limit_bytes: Option<u64>,
) -> ApiResult<ReaderCacheStatsResult> {
    let cache_root = reader_cache_root(app)?;
    let cache_limit_bytes = normalize_cache_limit(cache_limit_bytes);

    if cache_root.exists() {
        fs::remove_dir_all(&cache_root).map_err(map_cache_error)?;
    }

    clear_manifest_cache();
    reader_cache_stats(cache_root, cache_limit_bytes)
}

pub fn open_reader_cache_dir(app: &AppHandle) -> ApiResult<()> {
    let cache_root = reader_cache_root(app)?;

    fs::create_dir_all(&cache_root).map_err(map_cache_error)?;
    tauri_plugin_opener::open_path(&cache_root, None::<&str>)
        .map_err(|error| ApiError::new(ApiErrorKind::Cache, error.to_string()))
}

impl ReaderManifest {
    fn to_result(&self) -> ComicReadManifestResult {
        ComicReadManifestResult {
            endpoint: self.endpoint.clone(),
            read_id: self.read_id.clone(),
            page_count: self.pages.len() as u32,
            cache_limit_bytes: DEFAULT_READER_CACHE_LIMIT_BYTES,
        }
    }
}

async fn get_or_load_manifest(
    read_id: String,
    endpoint: Option<String>,
) -> ApiResult<ReaderManifest> {
    let read_id = normalize_read_id(read_id)?;
    let endpoint = resolve_api_endpoint(endpoint)?;
    let cache_key = manifest_cache_key(&endpoint, &read_id);

    if let Some(manifest) = cached_manifest(&cache_key) {
        return Ok(manifest);
    }

    let client = build_http_client()?;
    let img_host =
        request_reader_img_host(&client, &endpoint, &ReaderSettingRequest::current()).await?;
    let chapter =
        request_reader_chapter(&client, &endpoint, &read_id, &ReaderApiRequest::current()).await?;
    let manifest = build_reader_manifest(&endpoint, &read_id, &img_host, chapter)?;
    cache_manifest(cache_key, manifest.clone());

    Ok(manifest)
}

async fn request_reader_chapter(
    client: &reqwest::Client,
    endpoint: &str,
    read_id: &str,
    api_request: &ReaderApiRequest,
) -> ApiResult<ReaderChapterPayload> {
    let request_name = format!("{endpoint}/chapter");
    let response = client
        .get(&request_name)
        .header("accept", "application/json")
        .header("token", &api_request.token)
        .header("tokenparam", &api_request.tokenparam)
        .header("user-agent", android_user_agent())
        .query(&[("skip", ""), ("id", read_id)])
        .send()
        .await
        .map_err(|error| {
            ApiError::new(ApiErrorKind::Network, format!("{request_name}: {error}"))
        })?;

    if !response.status().is_success() {
        return Err(ApiError::new(
            ApiErrorKind::Http,
            format!("{request_name}: API returned HTTP {}", response.status()),
        ));
    }

    let body = response.text().await.map_err(|error| {
        ApiError::new(ApiErrorKind::Network, format!("{request_name}: {error}"))
    })?;

    decode_plugin_payload::<ReaderChapterPayload>(body.trim(), &api_request.ts)
        .or_else(|_| {
            serde_json::from_str::<ReaderChapterEnvelope>(body.trim()).map(|payload| {
                ReaderChapterPayload {
                    id: read_id.to_string(),
                    images: payload.images,
                }
            })
        })
        .map_err(|error| {
            ApiError::new(
                ApiErrorKind::Payload,
                format!(
                    "{request_name}: Invalid chapter payload: {error}. Body starts with: {}",
                    reader_response_preview(&body)
                ),
            )
        })
}

async fn request_reader_img_host(
    client: &reqwest::Client,
    endpoint: &str,
    setting_request: &ReaderSettingRequest,
) -> ApiResult<String> {
    let request_name = format!("{endpoint}/setting");
    let response = client
        .get(&request_name)
        .header("Tokenparam", &setting_request.tokenparam)
        .header("Token", &setting_request.token)
        .query(&[("app_img_shunt", "1"), ("t", setting_request.ts.as_str())])
        .send()
        .await
        .map_err(|error| {
            ApiError::new(ApiErrorKind::Network, format!("{request_name}: {error}"))
        })?;

    if !response.status().is_success() {
        return Err(ApiError::new(
            ApiErrorKind::Http,
            format!("{request_name}: API returned HTTP {}", response.status()),
        ));
    }

    let body = response.text().await.map_err(|error| {
        ApiError::new(ApiErrorKind::Network, format!("{request_name}: {error}"))
    })?;
    let payload =
        decode_setting_payload::<ReaderRemoteSettingPayload>(body.trim(), &setting_request.ts)
            .map_err(|error| {
                ApiError::new(
                    ApiErrorKind::Payload,
                    format!(
                        "{request_name}: Invalid setting payload: {error}. Body starts with: {}",
                        reader_response_preview(&body)
                    ),
                )
            })?;
    let img_host = payload.img_host.trim().trim_end_matches('/').to_string();

    if img_host.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "setting did not include image host",
        ));
    }

    Ok(img_host)
}

fn build_reader_manifest(
    endpoint: &str,
    read_id: &str,
    img_host: &str,
    chapter: ReaderChapterPayload,
) -> ApiResult<ReaderManifest> {
    if chapter.images.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "chapter did not include page images",
        ));
    }

    let img_host = img_host.trim().trim_end_matches('/');
    if img_host.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "setting did not include image host",
        ));
    }

    let chapter_id = if chapter.id.trim().is_empty() {
        read_id
    } else {
        chapter.id.trim()
    };
    let pages = chapter
        .images
        .into_iter()
        .enumerate()
        .filter_map(|(index, image)| {
            let image = image.trim().to_string();

            if image.is_empty() {
                return None;
            }

            Some(ReaderPage {
                index: index as u32,
                page_name: page_name_from_image(&image),
                source_url: format!("{img_host}/media/photos/{chapter_id}/{image}"),
            })
        })
        .collect::<Vec<_>>();

    if pages.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "chapter page image list is empty",
        ));
    }

    Ok(ReaderManifest {
        endpoint: endpoint.to_string(),
        read_id: read_id.to_string(),
        read_id_number: read_id.parse::<u32>().unwrap_or_default(),
        pages,
    })
}

#[derive(Debug, Deserialize)]
struct ReaderRemoteSettingPayload {
    #[serde(default, deserialize_with = "deserialize_string_from_any")]
    img_host: String,
}

#[derive(Debug, Clone)]
struct ReaderApiRequest {
    ts: String,
    token: String,
    tokenparam: String,
}

impl ReaderApiRequest {
    fn current() -> Self {
        let ts = current_millis_timestamp();
        Self {
            token: md5_hex(&format!("{ts}{JM_API_VERSION}")),
            tokenparam: format!("{ts},{JM_API_VERSION}"),
            ts,
        }
    }
}

#[derive(Debug, Clone)]
struct ReaderSettingRequest {
    ts: String,
    token: String,
    tokenparam: String,
}

impl ReaderSettingRequest {
    fn current() -> Self {
        let ts = current_seconds_timestamp();
        Self {
            token: md5_hex(&format!("{ts}{JM_API_SECRET}")),
            tokenparam: format!("{ts},{JM_API_VERSION}"),
            ts,
        }
    }
}

async fn materialize_reader_page(
    app: &AppHandle,
    manifest: &ReaderManifest,
    index: u32,
    cache_limit_bytes: u64,
    request_origin: Option<String>,
) -> ApiResult<ComicReadPageResult> {
    materialize_reader_page_inner(app, manifest, index, cache_limit_bytes, request_origin).await
}

async fn materialize_reader_page_inner(
    app: &AppHandle,
    manifest: &ReaderManifest,
    index: u32,
    cache_limit_bytes: u64,
    request_origin: Option<String>,
) -> ApiResult<ComicReadPageResult> {
    let materialize_started_at = Instant::now();
    let origin = resolve_reader_materialize_origin(request_origin);
    let page = manifest
        .pages
        .get(index as usize)
        .ok_or_else(|| ApiError::new(ApiErrorKind::MissingData, "Reader page is out of range"))?
        .clone();
    let cache_root = reader_cache_root(app)?;
    let cache_path = reader_page_cache_path(&cache_root, manifest, &page)?;
    let materialize_lock = reader_page_materialize_lock(&cache_path);
    let lock_started_at = Instant::now();
    let _materialize_guard = materialize_lock.lock().await;
    let lock_wait_elapsed = lock_started_at.elapsed();

    if cache_path.exists() {
        let path = cache_path.clone();
        match tokio::task::spawn_blocking(move || image_dimensions_from_path(&path)).await {
            Ok(Ok((width, height))) => {
                eprintln!(
                    "Reader page cache hit read_id={} page={} origin={} lock_wait_ms={:.1} total_ms={:.1}",
                    manifest.read_id,
                    index + 1,
                    origin.as_str(),
                    elapsed_ms(lock_wait_elapsed),
                    elapsed_ms(materialize_started_at.elapsed()),
                );

                return Ok(page_result(
                    manifest, index, cache_path, width, height, true,
                ));
            }
            Ok(Err(error)) => {
                eprintln!("Failed to read cached reader page, refreshing it: {error}");
                let _ = fs::remove_file(&cache_path);
            }
            Err(error) => {
                return Err(ApiError::new(
                    ApiErrorKind::Decode,
                    format!("Failed to read cached reader page: {error}"),
                ));
            }
        }
    }

    let client = build_http_client()?;
    let download_started_at = Instant::now();
    let bytes = download_image_bytes(&client, &page.source_url, &manifest.endpoint).await?;
    let download_elapsed = download_started_at.elapsed();

    let page_for_decode = page.clone();
    let manifest_for_decode = manifest.clone();
    let cache_path_for_decode = cache_path.clone();
    let cache_root_for_cleanup = cache_root.clone();

    let (width, height) = tokio::task::spawn_blocking(move || {
        write_reader_page_cache(
            &cache_root_for_cleanup,
            &cache_path_for_decode,
            &manifest_for_decode,
            &page_for_decode,
            &bytes,
            cache_limit_bytes,
            origin,
        )
    })
    .await
    .map_err(|error| {
        ApiError::new(
            ApiErrorKind::Decode,
            format!("Failed to decode reader page: {error}"),
        )
    })??;
    let cache_elapsed = download_started_at
        .elapsed()
        .saturating_sub(download_elapsed);

    eprintln!(
        "Reader page materialized read_id={} page={} origin={} lock_wait_ms={:.1} download_ms={:.1} cache_ms={:.1} total_ms={:.1}",
        manifest.read_id,
        index + 1,
        origin.as_str(),
        elapsed_ms(lock_wait_elapsed),
        elapsed_ms(download_elapsed),
        elapsed_ms(cache_elapsed),
        elapsed_ms(materialize_started_at.elapsed())
    );

    Ok(page_result(
        manifest, index, cache_path, width, height, false,
    ))
}

fn spawn_chapter_cache_task(
    tasks: &mut JoinSet<bool>,
    app: &AppHandle,
    manifest: &ReaderManifest,
    pending: &mut VecDeque<u32>,
    cache_limit_bytes: u64,
    request_origin: Option<String>,
) {
    let Some(index) = pending.pop_front() else {
        return;
    };
    let app = app.clone();
    let manifest = manifest.clone();

    tasks.spawn(async move {
        materialize_reader_page_inner(&app, &manifest, index, cache_limit_bytes, request_origin)
            .await
            .is_ok()
    });
}

fn resolve_reader_materialize_origin(
    request_origin: Option<String>,
) -> ReaderPageMaterializeOrigin {
    match request_origin.as_deref() {
        Some("chapter_cache") => ReaderPageMaterializeOrigin::ChapterCache,
        _ => ReaderPageMaterializeOrigin::Visible,
    }
}

async fn download_image_bytes(
    client: &reqwest::Client,
    source_url: &str,
    _endpoint: &str,
) -> ApiResult<Vec<u8>> {
    let host = url_host(source_url);
    let response = client
        .get(source_url)
        .header("host", host)
        .send()
        .await
        .map_err(|error| ApiError::new(ApiErrorKind::Network, error.to_string()))?;

    if !response.status().is_success() {
        return Err(ApiError::new(
            ApiErrorKind::Http,
            format!("Image CDN returned HTTP {}", response.status()),
        ));
    }

    response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(|error| ApiError::new(ApiErrorKind::Network, error.to_string()))
}

fn write_reader_page_cache(
    cache_root: &Path,
    cache_path: &Path,
    manifest: &ReaderManifest,
    page: &ReaderPage,
    bytes: &[u8],
    cache_limit_bytes: u64,
    origin: ReaderPageMaterializeOrigin,
) -> ApiResult<(u32, u32)> {
    let total_started_at = Instant::now();

    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(map_cache_error)?;
    }

    if !should_decode_image(manifest, page) {
        let write_started_at = Instant::now();
        write_temp_reader_cache_file(cache_path, |temp_path| {
            fs::write(temp_path, bytes).map_err(map_cache_error)
        })?;
        let output_bytes = file_size_bytes(cache_path);
        let write_elapsed = write_started_at.elapsed();
        let cleanup_started_at = Instant::now();
        cleanup_reader_cache(cache_root, cache_limit_bytes)?;
        let cleanup_elapsed = cleanup_started_at.elapsed();
        log_reader_cache_timing(
            manifest,
            page,
            origin,
            "direct",
            bytes.len(),
            output_bytes,
            &[
                ("write_ms", write_elapsed),
                ("cleanup_ms", cleanup_elapsed),
                ("total_ms", total_started_at.elapsed()),
            ],
        );

        return Ok((0, 0));
    }

    let load_started_at = Instant::now();
    let original = image::load_from_memory(bytes).map_err(map_image_error)?;
    let load_elapsed = load_started_at.elapsed();
    let decode_started_at = Instant::now();
    let decoded = decode_scrambled_image(original, manifest.read_id_number, &page.page_name)?;
    let decode_elapsed = decode_started_at.elapsed();
    let (decoded_width, decoded_height) = decoded.dimensions();

    let encode_started_at = Instant::now();
    let webp_bytes = encode_scrambled_webp_cache(&decoded);
    let encode_elapsed = encode_started_at.elapsed();
    let write_started_at = Instant::now();
    write_temp_reader_cache_file(cache_path, |temp_path| {
        fs::write(temp_path, &webp_bytes).map_err(map_cache_error)
    })?;
    let write_elapsed = write_started_at.elapsed();
    let output_bytes = file_size_bytes(cache_path);
    let cleanup_started_at = Instant::now();
    cleanup_reader_cache(cache_root, cache_limit_bytes)?;
    let cleanup_elapsed = cleanup_started_at.elapsed();
    log_reader_cache_timing(
        manifest,
        page,
        origin,
        "scrambled_webp_q75",
        bytes.len(),
        output_bytes,
        &[
            ("load_ms", load_elapsed),
            ("reorder_ms", decode_elapsed),
            ("encode_ms", encode_elapsed),
            ("write_ms", write_elapsed),
            ("cleanup_ms", cleanup_elapsed),
            ("total_ms", total_started_at.elapsed()),
        ],
    );

    Ok((decoded_width, decoded_height))
}

fn encode_scrambled_webp_cache(decoded: &RgbImage) -> Vec<u8> {
    let (width, height) = decoded.dimensions();
    let encoder = webp::Encoder::from_rgb(decoded, width, height);

    encoder.encode(SCRAMBLED_WEBP_QUALITY).to_vec()
}

fn log_reader_cache_timing(
    manifest: &ReaderManifest,
    page: &ReaderPage,
    origin: ReaderPageMaterializeOrigin,
    mode: &str,
    source_bytes: usize,
    output_bytes: Option<u64>,
    timings: &[(&str, Duration)],
) {
    let size_info = reader_cache_size_log(source_bytes as u64, output_bytes);
    let timings = timings
        .iter()
        .map(|(name, duration)| format!("{name}={:.1}", elapsed_ms(*duration)))
        .collect::<Vec<_>>()
        .join(" ");

    eprintln!(
        "Reader cache write read_id={} page={} origin={} mode={} {} {}",
        manifest.read_id,
        page.index + 1,
        origin.as_str(),
        mode,
        size_info,
        timings
    );
}

fn reader_cache_size_log(source_bytes: u64, output_bytes: Option<u64>) -> String {
    let source_kb = bytes_to_kb(source_bytes);

    if let Some(output_bytes) = output_bytes {
        let output_kb = bytes_to_kb(output_bytes);
        let ratio = if source_bytes == 0 {
            0.0
        } else {
            output_bytes as f64 / source_bytes as f64
        };

        format!(
            "source_kb={source_kb:.1} output_kb={output_kb:.1} ratio={ratio:.2} delta_kb={:+.1}",
            output_kb - source_kb
        )
    } else {
        format!("source_kb={source_kb:.1} output_kb=? ratio=? delta_kb=?")
    }
}

fn bytes_to_kb(bytes: u64) -> f64 {
    bytes as f64 / 1024.0
}

fn file_size_bytes(path: &Path) -> Option<u64> {
    fs::metadata(path).map(|metadata| metadata.len()).ok()
}

fn elapsed_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

fn reader_page_materialize_lock(cache_path: &Path) -> Arc<AsyncMutex<()>> {
    let mut locks = PAGE_MATERIALIZE_LOCKS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(|error| error.into_inner());

    locks
        .entry(cache_path.to_path_buf())
        .or_insert_with(|| Arc::new(AsyncMutex::new(())))
        .clone()
}

fn write_temp_reader_cache_file<F>(cache_path: &Path, write: F) -> ApiResult<()>
where
    F: FnOnce(&Path) -> ApiResult<()>,
{
    let temp_path = reader_page_temp_cache_path(cache_path);

    if temp_path.exists() {
        let _ = fs::remove_file(&temp_path);
    }

    if let Err(error) = write(&temp_path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }

    persist_reader_cache_file(&temp_path, cache_path)
}

fn persist_reader_cache_file(temp_path: &Path, cache_path: &Path) -> ApiResult<()> {
    match fs::rename(temp_path, cache_path) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(temp_path);

            if cache_path.exists() {
                Ok(())
            } else {
                Err(map_cache_error(error))
            }
        }
    }
}

fn decode_scrambled_image(
    original: DynamicImage,
    read_id: u32,
    page_name: &str,
) -> ApiResult<RgbImage> {
    let rgb = original.to_rgb8();
    let seed = calculate_seed(read_id, page_name);

    Ok(reorder_scrambled_rgb_rows(&rgb, seed))
}

fn reorder_scrambled_rgb_rows(source: &RgbImage, seed: u32) -> RgbImage {
    let (natural_width, natural_height) = source.dimensions();
    let row_bytes = natural_width as usize * 3;
    let source_bytes = source.as_raw();
    let mut decoded = RgbImage::new(natural_width, natural_height);
    let decoded_bytes = decoded.as_mut();
    let remainder = natural_height % seed;

    for index in 0..seed {
        let mut height = natural_height / seed;
        let mut dy = height * index;
        let sy = natural_height - height * (index + 1) - remainder;

        if index == 0 {
            height += remainder;
        } else {
            dy += remainder;
        }

        for row in 0..height {
            let source_offset = (sy + row) as usize * row_bytes;
            let target_offset = (dy + row) as usize * row_bytes;
            let source_row = &source_bytes[source_offset..source_offset + row_bytes];
            let target_row = &mut decoded_bytes[target_offset..target_offset + row_bytes];

            target_row.copy_from_slice(source_row);
        }
    }

    decoded
}

fn should_decode_image(manifest: &ReaderManifest, page: &ReaderPage) -> bool {
    manifest.read_id_number > 0
        && !is_gif_source(&page.source_url)
        && manifest.read_id_number >= JM_SCRAMBLE_ID
}

fn calculate_seed(read_id: u32, page_name: &str) -> u32 {
    let key = format!("{read_id}{page_name}");
    let key_md5 = format!("{:x}", md5::compute(key));
    let mut char_code = key_md5
        .as_bytes()
        .last()
        .copied()
        .map(usize::from)
        .unwrap_or_default();
    let left = 268850;
    let right = 421925;

    if (left..=right).contains(&read_id) {
        char_code %= 10;
    } else if read_id >= right + 1 {
        char_code %= 8;
    }

    SEED_MAP.get(char_code).copied().unwrap_or(10)
}

fn page_result(
    manifest: &ReaderManifest,
    index: u32,
    path: PathBuf,
    width: u32,
    height: u32,
    is_cached: bool,
) -> ComicReadPageResult {
    ComicReadPageResult {
        read_id: manifest.read_id.clone(),
        index,
        path: path.to_string_lossy().to_string(),
        width,
        height,
        aspect_ratio: if height == 0 {
            1.0
        } else {
            width as f32 / height as f32
        },
        is_cached,
    }
}

fn image_dimensions_from_path(path: &Path) -> ApiResult<(u32, u32)> {
    image::image_dimensions(path).map_err(map_image_error)
}

fn reader_cache_root(app: &AppHandle) -> ApiResult<PathBuf> {
    app.path()
        .app_cache_dir()
        .map(|path| path.join("reader"))
        .map_err(|error| ApiError::new(ApiErrorKind::Cache, error.to_string()))
}

fn reader_page_cache_path(
    cache_root: &Path,
    manifest: &ReaderManifest,
    page: &ReaderPage,
) -> ApiResult<PathBuf> {
    let extension = if should_decode_image(manifest, page) {
        "webp"
    } else {
        source_extension(&page.source_url)
    };
    let read_dir = cache_root.join(safe_path_segment(&manifest.read_id));

    Ok(read_dir.join(format!("{:04}.{extension}", page.index + 1)))
}

fn reader_page_temp_cache_path(cache_path: &Path) -> PathBuf {
    let file_name = cache_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("page");
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();

    cache_path.with_file_name(format!(
        "{}.{}.{}.tmp",
        file_name,
        std::process::id(),
        nonce
    ))
}

fn is_reader_cache_temp_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("tmp"))
}

fn cleanup_reader_cache(cache_root: &Path, cache_limit_bytes: u64) -> ApiResult<()> {
    let files = collect_cache_files(cache_root)?;
    let total_size = files.iter().map(|file| file.size).sum::<u64>();

    if total_size <= cache_limit_bytes {
        return Ok(());
    }

    let cache_trim_bytes = cache_trim_bytes(cache_limit_bytes);
    let mut files = files;
    files.sort_by_key(|file| file.modified);
    let mut current_size = total_size;

    for file in files {
        if current_size <= cache_trim_bytes {
            break;
        }

        match fs::remove_file(&file.path) {
            Ok(()) => {
                current_size = current_size.saturating_sub(file.size);
            }
            Err(error) => {
                eprintln!(
                    "Failed to remove reader cache file {:?}: {error}",
                    file.path
                );
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct CacheFile {
    path: PathBuf,
    size: u64,
    modified: SystemTime,
}

fn collect_cache_files(cache_root: &Path) -> ApiResult<Vec<CacheFile>> {
    let mut files = Vec::new();

    if !cache_root.exists() {
        return Ok(files);
    }

    collect_cache_files_in(cache_root, &mut files)?;

    Ok(files)
}

fn collect_cache_files_in(dir: &Path, files: &mut Vec<CacheFile>) -> ApiResult<()> {
    for entry in fs::read_dir(dir).map_err(map_cache_error)? {
        let entry = entry.map_err(map_cache_error)?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(map_cache_error)?;

        if metadata.is_dir() {
            collect_cache_files_in(&path, files)?;
        } else if metadata.is_file() {
            if is_reader_cache_temp_path(&path) {
                continue;
            }

            files.push(CacheFile {
                path,
                size: metadata.len(),
                modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            });
        }
    }

    Ok(())
}

fn reader_cache_stats(
    cache_root: PathBuf,
    cache_limit_bytes: u64,
) -> ApiResult<ReaderCacheStatsResult> {
    let files = collect_cache_files(&cache_root)?;
    let total_bytes = files.iter().map(|file| file.size).sum::<u64>();

    Ok(ReaderCacheStatsResult {
        cache_dir: cache_root.to_string_lossy().to_string(),
        total_bytes,
        file_count: files.len() as u32,
        cache_limit_bytes,
        cache_trim_bytes: cache_trim_bytes(cache_limit_bytes),
    })
}

fn cached_manifest(cache_key: &str) -> Option<ReaderManifest> {
    MANIFEST_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .ok()
        .and_then(|cache| cache.get(cache_key).cloned())
}

fn cache_manifest(cache_key: String, manifest: ReaderManifest) {
    if let Ok(mut cache) = MANIFEST_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        cache.insert(cache_key, manifest);
    }
}

fn clear_manifest_cache() {
    if let Ok(mut cache) = MANIFEST_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        cache.clear();
    }
}

fn manifest_cache_key(endpoint: &str, read_id: &str) -> String {
    format!("{endpoint}|{read_id}")
}

fn normalize_read_id(read_id: String) -> ApiResult<String> {
    let read_id = read_id.trim().to_string();

    if read_id.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "Reader needs a read_id",
        ));
    }

    Ok(read_id)
}

fn normalize_cache_limit(cache_limit_bytes: Option<u64>) -> u64 {
    cache_limit_bytes
        .unwrap_or(DEFAULT_READER_CACHE_LIMIT_BYTES)
        .clamp(MIN_READER_CACHE_LIMIT_BYTES, MAX_READER_CACHE_LIMIT_BYTES)
}

fn cache_trim_bytes(cache_limit_bytes: u64) -> u64 {
    cache_limit_bytes.saturating_mul(82) / 100
}

fn page_name_from_image(image: &str) -> String {
    let image = image.split('?').next().unwrap_or(image);
    image
        .rsplit('/')
        .next()
        .unwrap_or(image)
        .rsplit_once('.')
        .map(|(name, _)| name.to_string())
        .unwrap_or_else(|| image.to_string())
}

fn source_extension(source_url: &str) -> &'static str {
    let source = source_url.split('?').next().unwrap_or(source_url);
    let extension = source
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "gif" => "gif",
        "png" => "png",
        "webp" => "webp",
        "jpeg" => "jpg",
        _ => "jpg",
    }
}

fn is_gif_source(source_url: &str) -> bool {
    source_extension(source_url) == "gif"
}

fn safe_path_segment(value: &str) -> String {
    let segment = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .collect::<String>();

    if segment.is_empty() {
        "unknown".to_string()
    } else {
        segment
    }
}

fn map_image_error(error: image::ImageError) -> ApiError {
    ApiError::new(ApiErrorKind::Decode, error.to_string())
}

fn map_cache_error(error: std::io::Error) -> ApiError {
    ApiError::new(ApiErrorKind::Cache, error.to_string())
}

fn reader_response_preview(value: &str) -> String {
    value
        .chars()
        .take(180)
        .collect::<String>()
        .replace('\n', "\\n")
}

fn current_millis_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn current_seconds_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn android_user_agent() -> &'static str {
    "Mozilla/5.0 (Linux; Android 13; jm-boom Build/TQ1A.230305.002; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/120.0.6099.230 Mobile Safari/537.36"
}

fn url_host(url: &str) -> String {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_string))
        .unwrap_or_default()
}

fn decode_plugin_payload<T>(body: &str, ts: &str) -> ApiResult<T>
where
    T: DeserializeOwned,
{
    let envelope: ReaderApiEnvelope = serde_json::from_str(body).map_err(|error| {
        ApiError::new(
            ApiErrorKind::Payload,
            format!(
                "Invalid plugin envelope: {error}. Body starts with: {}",
                reader_response_preview(body)
            ),
        )
    })?;

    if envelope.code != 200 {
        return Err(ApiError::new(
            ApiErrorKind::Api,
            envelope
                .error_msg
                .unwrap_or_else(|| format!("API returned code {}", envelope.code)),
        ));
    }

    let data = envelope.data.ok_or_else(|| {
        ApiError::new(
            ApiErrorKind::MissingData,
            "API response did not include data",
        )
    })?;

    match data {
        serde_json::Value::String(encrypted) => {
            let decrypted = decrypt_plugin_data(&encrypted, ts)
                .map_err(|error| ApiError::new(ApiErrorKind::Decrypt, error))?;
            serde_json::from_str(&decrypted).map_err(|error| {
                ApiError::new(
                    ApiErrorKind::Payload,
                    format!(
                        "Invalid decrypted payload: {error}. Payload starts with: {}",
                        reader_response_preview(&decrypted)
                    ),
                )
            })
        }
        value => serde_json::from_value(value).map_err(|error| {
            ApiError::new(ApiErrorKind::Payload, format!("Invalid payload: {error}"))
        }),
    }
}

fn decrypt_plugin_data(data: &str, ts: &str) -> Result<String, String> {
    let key = md5_hex(&format!("{ts}{JM_API_SECRET}"));
    decrypt_base64_with_key(data, &key)
}

fn decrypt_base64_with_key(data: &str, key: &str) -> Result<String, String> {
    let encrypted = BASE64_STANDARD
        .decode(data)
        .map_err(|error| format!("Invalid encrypted data: {error}"))?;
    let decrypted = ecb::Decryptor::<Aes256>::new_from_slice(key.as_bytes())
        .map_err(|error| format!("Invalid AES key: {error}"))?
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted)
        .map_err(|error| format!("Failed to decrypt response: {error}"))?;

    String::from_utf8(decrypted).map_err(|error| format!("Invalid decrypted text: {error}"))
}

fn md5_hex(input: &str) -> String {
    format!("{:x}", md5::compute(input))
}

fn deserialize_string_from_any<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;

    match value {
        serde_json::Value::String(value) => Ok(value),
        serde_json::Value::Number(value) => Ok(value.to_string()),
        serde_json::Value::Bool(value) => Ok(value.to_string()),
        serde_json::Value::Null => Ok(String::new()),
        _ => Err(serde::de::Error::custom("expected a scalar value")),
    }
}
