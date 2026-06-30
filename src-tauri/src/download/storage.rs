use super::paths::task_output_dir;
use super::progress::current_timestamp;
use super::types::{DownloadTask, DownloadTaskStatus};
use crate::api::{ApiError, ApiErrorKind, ApiResult};
use crate::storage;
use sqlx::{Row, SqlitePool};
use tauri::AppHandle;

pub(crate) fn recover_interrupted_tasks(tasks: &mut [DownloadTask]) -> bool {
    let now = current_timestamp();
    let mut recovered = false;

    for task in tasks {
        if task.status != DownloadTaskStatus::Running {
            continue;
        }

        task.status = DownloadTaskStatus::Queued;
        task.started_at = None;
        task.completed_at = None;
        task.current_chapter_title.clear();
        task.total_pages = 0;
        task.completed_pages = 0;
        task.eta_seconds = None;
        task.speed_bytes_per_second = 0;
        task.error = None;
        task.updated_at = now;
        recovered = true;
    }

    recovered
}

pub(crate) fn migrate_pending_task_output_dirs(
    app: &AppHandle,
    tasks: &mut [DownloadTask],
) -> ApiResult<bool> {
    let mut migrated = false;

    for task in tasks {
        if !matches!(
            task.status,
            DownloadTaskStatus::Queued | DownloadTaskStatus::Running | DownloadTaskStatus::Paused
        ) {
            continue;
        }

        let output_dir = task_output_dir(app, &task.comic_title)?
            .to_string_lossy()
            .to_string();
        if task.output_dir == output_dir {
            continue;
        }

        task.output_dir = output_dir;
        task.updated_at = current_timestamp();
        migrated = true;
    }

    Ok(migrated)
}

pub(crate) async fn load_tasks() -> ApiResult<Vec<DownloadTask>> {
    let pool = storage::pool().map_err(|error| ApiError::new(ApiErrorKind::Cache, error))?;
    load_tasks_from_pool(pool).await
}

pub(crate) async fn persist_tasks(tasks: &[DownloadTask]) -> ApiResult<()> {
    let pool = storage::pool().map_err(|error| ApiError::new(ApiErrorKind::Cache, error))?;
    persist_tasks_to_pool(pool, tasks).await
}

async fn load_tasks_from_pool(pool: &SqlitePool) -> ApiResult<Vec<DownloadTask>> {
    let task_rows = sqlx::query(
        r#"
        SELECT task_id, album_id, comic_title, endpoint, status, current_chapter_title,
               total_pages, completed_pages, eta_seconds, speed_bytes_per_second,
               output_dir, error, created_at, started_at, updated_at, completed_at
        FROM download_tasks
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;

    let mut tasks = Vec::with_capacity(task_rows.len());
    for row in task_rows {
        let task_id: String = row.get("task_id");
        let chapter_rows = sqlx::query(
            r#"
            SELECT chapter_id, title, chapter_order
            FROM download_chapters
            WHERE task_id = ?
            ORDER BY order_index ASC
            "#,
        )
        .bind(&task_id)
        .fetch_all(pool)
        .await
        .map_err(map_sqlx_error)?;
        let chapters = chapter_rows
            .into_iter()
            .map(|row| super::types::DownloadChapterRequest {
                chapter_id: row.get("chapter_id"),
                title: row.get("title"),
                order: i64_to_u32(row.get("chapter_order")),
            })
            .collect();

        tasks.push(DownloadTask {
            task_id,
            album_id: row.get("album_id"),
            comic_title: row.get("comic_title"),
            endpoint: row.get("endpoint"),
            chapters,
            status: status_from_db(row.get::<String, _>("status"))?,
            current_chapter_title: row.get("current_chapter_title"),
            total_pages: i64_to_u32(row.get("total_pages")),
            completed_pages: i64_to_u32(row.get("completed_pages")),
            eta_seconds: row.get::<Option<i64>, _>("eta_seconds").map(i64_to_u64),
            speed_bytes_per_second: i64_to_u64(row.get("speed_bytes_per_second")),
            output_dir: row.get("output_dir"),
            error: row.get("error"),
            created_at: i64_to_u64(row.get("created_at")),
            started_at: row.get::<Option<i64>, _>("started_at").map(i64_to_u64),
            updated_at: i64_to_u64(row.get("updated_at")),
            completed_at: row.get::<Option<i64>, _>("completed_at").map(i64_to_u64),
        });
    }

    Ok(tasks)
}

async fn persist_tasks_to_pool(pool: &SqlitePool, tasks: &[DownloadTask]) -> ApiResult<()> {
    let mut transaction = pool.begin().await.map_err(map_sqlx_error)?;

    sqlx::query("DELETE FROM download_tasks")
        .execute(&mut *transaction)
        .await
        .map_err(map_sqlx_error)?;

    for task in tasks {
        sqlx::query(
            r#"
            INSERT INTO download_tasks (
                task_id, album_id, comic_title, endpoint, status, current_chapter_title,
                total_pages, completed_pages, eta_seconds, speed_bytes_per_second,
                output_dir, error, created_at, started_at, updated_at, completed_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&task.task_id)
        .bind(&task.album_id)
        .bind(&task.comic_title)
        .bind(&task.endpoint)
        .bind(status_to_db(task.status))
        .bind(&task.current_chapter_title)
        .bind(u32_to_i64(task.total_pages))
        .bind(u32_to_i64(task.completed_pages))
        .bind(task.eta_seconds.map(u64_to_i64))
        .bind(u64_to_i64(task.speed_bytes_per_second))
        .bind(&task.output_dir)
        .bind(&task.error)
        .bind(u64_to_i64(task.created_at))
        .bind(task.started_at.map(u64_to_i64))
        .bind(u64_to_i64(task.updated_at))
        .bind(task.completed_at.map(u64_to_i64))
        .execute(&mut *transaction)
        .await
        .map_err(map_sqlx_error)?;

        for (index, chapter) in task.chapters.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO download_chapters (
                    task_id, chapter_id, title, order_index, chapter_order
                )
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(&task.task_id)
            .bind(&chapter.chapter_id)
            .bind(&chapter.title)
            .bind(usize_to_i64(index))
            .bind(u32_to_i64(chapter.order))
            .execute(&mut *transaction)
            .await
            .map_err(map_sqlx_error)?;
        }
    }

    transaction.commit().await.map_err(map_sqlx_error)
}

fn status_to_db(status: DownloadTaskStatus) -> &'static str {
    match status {
        DownloadTaskStatus::Queued => "queued",
        DownloadTaskStatus::Running => "running",
        DownloadTaskStatus::Paused => "paused",
        DownloadTaskStatus::Completed => "completed",
        DownloadTaskStatus::Failed => "failed",
        DownloadTaskStatus::Cancelled => "cancelled",
    }
}

fn status_from_db(status: String) -> ApiResult<DownloadTaskStatus> {
    match status.as_str() {
        "queued" => Ok(DownloadTaskStatus::Queued),
        "running" => Ok(DownloadTaskStatus::Running),
        "paused" => Ok(DownloadTaskStatus::Paused),
        "completed" => Ok(DownloadTaskStatus::Completed),
        "failed" => Ok(DownloadTaskStatus::Failed),
        "cancelled" => Ok(DownloadTaskStatus::Cancelled),
        value => Err(ApiError::new(
            ApiErrorKind::Payload,
            format!("Unknown download task status: {value}"),
        )),
    }
}

fn map_sqlx_error(error: sqlx::Error) -> ApiError {
    ApiError::new(
        ApiErrorKind::Cache,
        format!("SQLite download storage error: {error}"),
    )
}

fn i64_to_u32(value: i64) -> u32 {
    value.max(0).min(u32::MAX as i64) as u32
}

fn i64_to_u64(value: i64) -> u64 {
    value.max(0) as u64
}

fn u32_to_i64(value: u32) -> i64 {
    i64::from(value)
}

fn u64_to_i64(value: u64) -> i64 {
    value.min(i64::MAX as u64) as i64
}

fn usize_to_i64(value: usize) -> i64 {
    value.min(i64::MAX as usize) as i64
}
