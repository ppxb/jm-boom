use crate::api::{
    build_http_client, current_timestamp, resolve_api_endpoint, ApiAuth, ApiError, ApiErrorKind,
    ApiResult,
};
use image::imageops::{crop_imm, replace};
use image::{DynamicImage, GenericImageView, ImageFormat, RgbaImage};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;
use tauri::{AppHandle, Manager};
use tokio::task::JoinSet;

const DEFAULT_SHUNT: &str = "1";
const DEFAULT_PREFETCH_RADIUS: u32 = 3;
const DEFAULT_READER_CACHE_LIMIT_BYTES: u64 = 512 * 1024 * 1024;
const MIN_READER_CACHE_LIMIT_BYTES: u64 = 128 * 1024 * 1024;
const MAX_READER_CACHE_LIMIT_BYTES: u64 = 2048 * 1024 * 1024;
const SEED_MAP: [u32; 10] = [2, 4, 6, 8, 10, 12, 14, 16, 18, 20];

static MANIFEST_CACHE: OnceLock<Mutex<HashMap<String, ReaderManifest>>> = OnceLock::new();
static RESULT_RE: OnceLock<Regex> = OnceLock::new();
static CONFIG_RE: OnceLock<Regex> = OnceLock::new();
static AID_RE: OnceLock<Regex> = OnceLock::new();
static SCRAMBLE_RE: OnceLock<Regex> = OnceLock::new();
static SPEED_RE: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Clone)]
struct ReaderManifest {
    endpoint: String,
    read_id: String,
    read_id_number: u32,
    shunt: String,
    scramble_id: u32,
    speed: String,
    pages: Vec<ReaderPage>,
}

#[derive(Debug, Clone)]
struct ReaderPage {
    index: u32,
    page_name: String,
    source_url: String,
}

#[derive(Debug, Deserialize)]
struct ReaderResultScript {
    images: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ReaderConfigScript {
    imghost: String,
    jmid: String,
    cache: String,
}

#[derive(Debug, Serialize)]
pub struct ComicReadManifestResult {
    pub endpoint: String,
    #[serde(rename = "readId")]
    pub read_id: String,
    pub shunt: String,
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
pub struct ComicReadPrefetchResult {
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
    shunt: Option<String>,
    endpoint: Option<String>,
) -> ApiResult<ComicReadManifestResult> {
    let manifest = get_or_load_manifest(read_id, shunt, endpoint).await?;

    Ok(manifest.to_result())
}

pub async fn get_comic_read_page(
    app: &AppHandle,
    read_id: String,
    index: u32,
    shunt: Option<String>,
    endpoint: Option<String>,
    cache_limit_bytes: Option<u64>,
) -> ApiResult<ComicReadPageResult> {
    let manifest = get_or_load_manifest(read_id, shunt, endpoint).await?;

    materialize_reader_page(app, &manifest, index, normalize_cache_limit(cache_limit_bytes)).await
}

pub async fn prefetch_comic_read_pages(
    app: &AppHandle,
    read_id: String,
    center_index: u32,
    radius: Option<u32>,
    shunt: Option<String>,
    endpoint: Option<String>,
    cache_limit_bytes: Option<u64>,
) -> ApiResult<ComicReadPrefetchResult> {
    let manifest = get_or_load_manifest(read_id, shunt, endpoint).await?;
    let cache_limit_bytes = normalize_cache_limit(cache_limit_bytes);
    let radius = radius.unwrap_or(DEFAULT_PREFETCH_RADIUS).max(1);
    let page_count = manifest.pages.len() as u32;

    if page_count == 0 {
        return Ok(ComicReadPrefetchResult {
            requested: 0,
            completed: 0,
        });
    }

    let indexes = prefetch_indexes(center_index, radius, page_count);
    let requested = indexes.len() as u32;
    let mut completed = 0;
    let concurrency = radius as usize;
    let mut pending = VecDeque::from(indexes);
    let mut tasks = JoinSet::new();

    for _ in 0..concurrency {
        spawn_prefetch_task(&mut tasks, app, &manifest, &mut pending, cache_limit_bytes);
    }

    while let Some(result) = tasks.join_next().await {
        if matches!(result, Ok(true)) {
            completed += 1;
        }

        spawn_prefetch_task(&mut tasks, app, &manifest, &mut pending, cache_limit_bytes);
    }

    Ok(ComicReadPrefetchResult {
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
            shunt: self.shunt.clone(),
            page_count: self.pages.len() as u32,
            cache_limit_bytes: DEFAULT_READER_CACHE_LIMIT_BYTES,
        }
    }
}

async fn get_or_load_manifest(
    read_id: String,
    shunt: Option<String>,
    endpoint: Option<String>,
) -> ApiResult<ReaderManifest> {
    let read_id = normalize_read_id(read_id)?;
    let shunt = normalize_shunt(shunt);
    let endpoint = resolve_api_endpoint(endpoint)?;
    let cache_key = manifest_cache_key(&endpoint, &read_id, &shunt);

    if let Some(manifest) = cached_manifest(&cache_key) {
        return Ok(manifest);
    }

    let client = build_http_client()?;
    let auth = ApiAuth::current();
    let html = request_reader_html(&client, &endpoint, &read_id, &shunt, &auth).await?;
    let manifest = parse_reader_manifest(&endpoint, &read_id, &shunt, &html)?;
    cache_manifest(cache_key, manifest.clone());

    Ok(manifest)
}

async fn request_reader_html(
    client: &reqwest::Client,
    endpoint: &str,
    read_id: &str,
    shunt: &str,
    auth: &ApiAuth,
) -> ApiResult<String> {
    let request_name = format!("{endpoint}/chapter_view_template");
    let response = client
        .get(&request_name)
        .header("accept", "text/html,application/xhtml+xml")
        .header("token", &auth.token)
        .header("tokenparam", &auth.tokenparam)
        .query(&[
            ("id", read_id.to_string()),
            ("app_img_shunt", shunt.to_string()),
            ("mode", "vertical".to_string()),
            ("page", "0".to_string()),
            ("express", "off".to_string()),
            ("v", current_timestamp().to_string()),
        ])
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

    response.text().await.map_err(|error| {
        ApiError::new(ApiErrorKind::Network, format!("{request_name}: {error}"))
    })
}

fn parse_reader_manifest(
    endpoint: &str,
    read_id: &str,
    shunt: &str,
    html: &str,
) -> ApiResult<ReaderManifest> {
    let result_script = capture_script_object(result_regex(), html, "result")?;
    let config_script = capture_script_object(config_regex(), html, "config")?;
    let result = json5::from_str::<ReaderResultScript>(result_script).map_err(|error| {
        ApiError::new(
            ApiErrorKind::Payload,
            format!(
                "chapter_view_template result is invalid: {error}. Script starts with: {}",
                reader_response_preview(result_script)
            ),
        )
    })?;
    let config = json5::from_str::<ReaderConfigScript>(config_script).map_err(|error| {
        ApiError::new(
            ApiErrorKind::Payload,
            format!(
                "chapter_view_template config is invalid: {error}. Script starts with: {}",
                reader_response_preview(config_script)
            ),
        )
    })?;

    if result.images.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "chapter_view_template did not include page images",
        ));
    }

    let img_host = config.imghost.trim().trim_end_matches('/');
    let jmid = config.jmid.trim();
    let cache = config.cache;

    if img_host.is_empty() || jmid.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "chapter_view_template did not include image host or jmid",
        ));
    }

    let pages = result
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
                source_url: format!("{img_host}/media/photos/{jmid}/{image}{cache}"),
            })
        })
        .collect::<Vec<_>>();

    if pages.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "chapter_view_template page image list is empty",
        ));
    }

    let aid = capture_u32(aid_regex(), html).unwrap_or_default();

    Ok(ReaderManifest {
        endpoint: endpoint.to_string(),
        read_id: read_id.to_string(),
        read_id_number: read_id.parse::<u32>().unwrap_or(aid),
        shunt: shunt.to_string(),
        scramble_id: capture_u32(scramble_regex(), html).unwrap_or_default(),
        speed: capture_string(speed_regex(), html).unwrap_or_default(),
        pages,
    })
}

async fn materialize_reader_page(
    app: &AppHandle,
    manifest: &ReaderManifest,
    index: u32,
    cache_limit_bytes: u64,
) -> ApiResult<ComicReadPageResult> {
    let page = manifest
        .pages
        .get(index as usize)
        .ok_or_else(|| ApiError::new(ApiErrorKind::MissingData, "Reader page is out of range"))?
        .clone();
    let cache_root = reader_cache_root(app)?;
    let cache_path = reader_page_cache_path(&cache_root, manifest, &page)?;

    if cache_path.exists() {
        let path = cache_path.clone();
        match tokio::task::spawn_blocking(move || image_dimensions_from_path(&path)).await {
            Ok(Ok((width, height))) => {
                return Ok(page_result(manifest, index, cache_path, width, height, true));
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
    let bytes = download_image_bytes(&client, &page.source_url, &manifest.endpoint).await?;
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
        )
    })
    .await
    .map_err(|error| {
        ApiError::new(
            ApiErrorKind::Decode,
            format!("Failed to decode reader page: {error}"),
        )
    })??;

    Ok(page_result(manifest, index, cache_path, width, height, false))
}

fn spawn_prefetch_task(
    tasks: &mut JoinSet<bool>,
    app: &AppHandle,
    manifest: &ReaderManifest,
    pending: &mut VecDeque<u32>,
    cache_limit_bytes: u64,
) {
    let Some(index) = pending.pop_front() else {
        return;
    };
    let app = app.clone();
    let manifest = manifest.clone();

    tasks.spawn(async move {
        materialize_reader_page(&app, &manifest, index, cache_limit_bytes)
            .await
            .is_ok()
    });
}

fn prefetch_indexes(center_index: u32, radius: u32, page_count: u32) -> Vec<u32> {
    if page_count == 0 {
        return Vec::new();
    }

    let mut indexes = Vec::new();

    for offset in 1..=radius {
        if let Some(next_index) = center_index.checked_add(offset) {
            if next_index < page_count {
                indexes.push(next_index);
            }
        }

        if let Some(previous_index) = center_index.checked_sub(offset) {
            indexes.push(previous_index);
        }
    }

    indexes
}

async fn download_image_bytes(
    client: &reqwest::Client,
    source_url: &str,
    endpoint: &str,
) -> ApiResult<Vec<u8>> {
    let response = client
        .get(source_url)
        .header("accept", "image/avif,image/webp,image/apng,image/*,*/*;q=0.8")
        .header("referer", endpoint)
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
) -> ApiResult<(u32, u32)> {
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(map_cache_error)?;
    }

    let original = image::load_from_memory(bytes).map_err(map_image_error)?;
    let (width, height) = original.dimensions();

    if should_decode_image(manifest, page) {
        let decoded = decode_scrambled_image(original, manifest.read_id_number, &page.page_name)?;
        let (decoded_width, decoded_height) = decoded.dimensions();

        DynamicImage::ImageRgba8(decoded)
            .save_with_format(cache_path, ImageFormat::WebP)
            .map_err(map_image_error)?;
        cleanup_reader_cache(cache_root, cache_limit_bytes)?;

        return Ok((decoded_width, decoded_height));
    }

    fs::write(cache_path, bytes).map_err(map_cache_error)?;
    cleanup_reader_cache(cache_root, cache_limit_bytes)?;

    Ok((width, height))
}

fn decode_scrambled_image(
    original: DynamicImage,
    read_id: u32,
    page_name: &str,
) -> ApiResult<RgbaImage> {
    let rgba = original.to_rgba8();
    let (natural_width, natural_height) = rgba.dimensions();
    let seed = calculate_seed(read_id, page_name);
    let remainder = natural_height % seed;
    let mut decoded = RgbaImage::new(natural_width, natural_height);

    for index in 0..seed {
        let mut height = natural_height / seed;
        let mut dy = height * index;
        let sy = natural_height - height * (index + 1) - remainder;

        if index == 0 {
            height += remainder;
        } else {
            dy += remainder;
        }

        let segment = crop_imm(&rgba, 0, sy, natural_width, height).to_image();
        replace(&mut decoded, &segment, 0, i64::from(dy));
    }

    Ok(decoded)
}

fn should_decode_image(manifest: &ReaderManifest, page: &ReaderPage) -> bool {
    manifest.read_id_number > 0
        && !is_gif_source(&page.source_url)
        && manifest.read_id_number > manifest.scramble_id
        && manifest.speed != "1"
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
                eprintln!("Failed to remove reader cache file {:?}: {error}", file.path);
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

fn manifest_cache_key(endpoint: &str, read_id: &str, shunt: &str) -> String {
    format!("{endpoint}|{read_id}|{shunt}")
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

fn normalize_shunt(shunt: Option<String>) -> String {
    let shunt = shunt.unwrap_or_else(|| DEFAULT_SHUNT.to_string());
    let shunt = shunt.trim();

    if shunt.is_empty() {
        DEFAULT_SHUNT.to_string()
    } else {
        shunt.to_string()
    }
}

fn normalize_cache_limit(cache_limit_bytes: Option<u64>) -> u64 {
    cache_limit_bytes
        .unwrap_or(DEFAULT_READER_CACHE_LIMIT_BYTES)
        .clamp(MIN_READER_CACHE_LIMIT_BYTES, MAX_READER_CACHE_LIMIT_BYTES)
}

fn cache_trim_bytes(cache_limit_bytes: u64) -> u64 {
    cache_limit_bytes.saturating_mul(82) / 100
}

fn capture_script_object<'a>(regex: &Regex, html: &'a str, name: &str) -> ApiResult<&'a str> {
    regex
        .captures(html)
        .and_then(|captures| captures.get(1))
        .map(|matched| matched.as_str())
        .ok_or_else(|| {
            ApiError::new(
                ApiErrorKind::MissingData,
                format!(
                    "chapter_view_template did not include {name} script. Body starts with: {}",
                    reader_response_preview(html)
                ),
            )
        })
}

fn capture_u32(regex: &Regex, html: &str) -> Option<u32> {
    regex
        .captures(html)
        .and_then(|captures| captures.get(1))
        .and_then(|matched| matched.as_str().parse::<u32>().ok())
}

fn capture_string(regex: &Regex, html: &str) -> Option<String> {
    regex
        .captures(html)
        .and_then(|captures| captures.get(1))
        .map(|matched| matched.as_str().to_string())
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

fn result_regex() -> &'static Regex {
    RESULT_RE.get_or_init(|| Regex::new(r#"(?s)const\s+result\s*=\s*(\{.*?\});"#).unwrap())
}

fn config_regex() -> &'static Regex {
    CONFIG_RE.get_or_init(|| Regex::new(r#"(?s)const\s+config\s*=\s*(\{.*?\});"#).unwrap())
}

fn aid_regex() -> &'static Regex {
    AID_RE.get_or_init(|| Regex::new(r#"var\s+aid\s*=\s*(\d+);"#).unwrap())
}

fn scramble_regex() -> &'static Regex {
    SCRAMBLE_RE.get_or_init(|| Regex::new(r#"var\s+scramble_id\s*=\s*(\d+);"#).unwrap())
}

fn speed_regex() -> &'static Regex {
    SPEED_RE.get_or_init(|| Regex::new(r#"var\s+speed\s*=\s*'([^']*)';"#).unwrap())
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
