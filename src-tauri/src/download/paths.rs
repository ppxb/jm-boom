use super::types::{DownloadChapterRequest, DownloadTask};
use crate::api::{ApiError, ApiErrorKind, ApiResult};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub(crate) fn download_files_root(app: &AppHandle) -> ApiResult<PathBuf> {
    app.path()
        .app_data_dir()
        .map(|path| path.join("downloads").join("file"))
        .map_err(|error| ApiError::new(ApiErrorKind::Cache, error.to_string()))
}

pub(crate) fn task_output_dir(app: &AppHandle, comic_title: &str) -> ApiResult<PathBuf> {
    Ok(download_files_root(app)?.join(safe_path_segment(comic_title)))
}

pub(crate) fn download_chapter_dir(output_dir: &str, chapter: &DownloadChapterRequest) -> PathBuf {
    PathBuf::from(output_dir).join(safe_path_segment(&chapter.title))
}

pub(crate) fn remove_task_files(app: &AppHandle, task: &DownloadTask) -> ApiResult<()> {
    let root = download_files_root(app)?;
    let root = match fs::canonicalize(root) {
        Ok(path) => path,
        Err(_) => return Ok(()),
    };
    let output_dir = PathBuf::from(&task.output_dir);

    for chapter in &task.chapters {
        let chapter_dir = download_chapter_dir(&task.output_dir, chapter);
        if !chapter_dir.exists() {
            continue;
        }
        let chapter_dir = fs::canonicalize(chapter_dir).map_err(map_download_error)?;
        if !chapter_dir.starts_with(&root) {
            continue;
        }

        if chapter_dir.is_dir() {
            fs::remove_dir_all(&chapter_dir).map_err(map_download_error)?;
        } else {
            fs::remove_file(&chapter_dir).map_err(map_download_error)?;
        }
    }

    if output_dir.exists() && output_dir.is_dir() {
        let output_dir = fs::canonicalize(output_dir).map_err(map_download_error)?;
        if output_dir.starts_with(&root) && is_directory_empty(&output_dir)? {
            fs::remove_dir(&output_dir).map_err(map_download_error)?;
        }
    }

    Ok(())
}

fn is_directory_empty(path: &PathBuf) -> ApiResult<bool> {
    let mut entries = fs::read_dir(path).map_err(map_download_error)?;

    Ok(entries.next().is_none())
}

pub(crate) fn file_size_bytes(path: &PathBuf) -> Option<u64> {
    fs::metadata(path).map(|metadata| metadata.len()).ok()
}

fn safe_path_segment(value: &str) -> String {
    let segment = value
        .trim()
        .chars()
        .map(|character| {
            if character.is_control()
                || matches!(
                    character,
                    '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
                )
            {
                '_'
            } else {
                character
            }
        })
        .collect::<String>()
        .trim_matches(['.', ' '])
        .to_string();

    if segment.is_empty() {
        "unknown".to_string()
    } else {
        segment
    }
}

pub(crate) fn map_download_error(error: std::io::Error) -> ApiError {
    ApiError::new(ApiErrorKind::Cache, error.to_string())
}
