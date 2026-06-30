use sqlx::SqlitePool;

const SCHEMA_VERSION: i64 = 2;

pub(crate) async fn run(pool: &SqlitePool) -> Result<(), String> {
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await
        .map_err(map_sqlx_error)?;

    let (mut current_version,): (i64,) = sqlx::query_as("PRAGMA user_version")
        .fetch_one(pool)
        .await
        .map_err(map_sqlx_error)?;

    if current_version > SCHEMA_VERSION {
        return Err(format!(
            "SQLite schema version {current_version} is newer than supported version {SCHEMA_VERSION}"
        ));
    }

    if current_version < 1 {
        create_schema_v1(pool).await?;
        current_version = 1;
    }

    if current_version < 2 {
        migrate_schema_v2(pool).await?;
    }

    set_schema_version(pool, SCHEMA_VERSION).await?;

    Ok(())
}

async fn create_schema_v1(pool: &SqlitePool) -> Result<(), String> {
    execute(
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS download_tasks (
            task_id TEXT PRIMARY KEY NOT NULL,
            album_id TEXT NOT NULL,
            comic_title TEXT NOT NULL,
            endpoint TEXT NOT NULL,
            status TEXT NOT NULL,
            current_chapter_title TEXT NOT NULL DEFAULT '',
            total_pages INTEGER NOT NULL DEFAULT 0,
            completed_pages INTEGER NOT NULL DEFAULT 0,
            eta_seconds INTEGER,
            speed_bytes_per_second INTEGER NOT NULL DEFAULT 0,
            output_dir TEXT NOT NULL,
            error TEXT,
            created_at INTEGER NOT NULL,
            started_at INTEGER,
            updated_at INTEGER NOT NULL,
            completed_at INTEGER
        )
        "#,
    )
    .await?;
    execute(
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS download_chapters (
            task_id TEXT NOT NULL,
            chapter_id TEXT NOT NULL,
            title TEXT NOT NULL,
            order_index INTEGER NOT NULL,
            PRIMARY KEY (task_id, order_index),
            FOREIGN KEY (task_id) REFERENCES download_tasks(task_id) ON DELETE CASCADE
        )
        "#,
    )
    .await?;
    execute(
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS reader_cache_entries (
            endpoint TEXT NOT NULL,
            read_id TEXT NOT NULL,
            page_index INTEGER NOT NULL,
            path TEXT NOT NULL UNIQUE,
            size_bytes INTEGER NOT NULL,
            width INTEGER NOT NULL DEFAULT 0,
            height INTEGER NOT NULL DEFAULT 0,
            extension TEXT NOT NULL,
            is_scrambled INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (endpoint, read_id, page_index)
        )
        "#,
    )
    .await?;
    execute(
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS runtime_cache_entries (
            cache_key TEXT PRIMARY KEY NOT NULL,
            cache_kind TEXT NOT NULL,
            value_json TEXT NOT NULL,
            expires_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
    )
    .await?;

    execute(
        pool,
        "CREATE INDEX IF NOT EXISTS idx_download_tasks_status ON download_tasks(status, updated_at)",
    )
    .await?;
    execute(
        pool,
        "CREATE INDEX IF NOT EXISTS idx_reader_cache_updated_at ON reader_cache_entries(updated_at)",
    )
    .await?;
    execute(
        pool,
        "CREATE INDEX IF NOT EXISTS idx_runtime_cache_kind_expires ON runtime_cache_entries(cache_kind, expires_at)",
    )
    .await?;

    Ok(())
}

async fn migrate_schema_v2(pool: &SqlitePool) -> Result<(), String> {
    let has_chapter_order = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM pragma_table_info('download_chapters')
        WHERE name = 'chapter_order'
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(map_sqlx_error)?
        > 0;

    if !has_chapter_order {
        execute(
            pool,
            "ALTER TABLE download_chapters ADD COLUMN chapter_order INTEGER NOT NULL DEFAULT 0",
        )
        .await?;
    }

    Ok(())
}

async fn execute(pool: &SqlitePool, statement: &str) -> Result<(), String> {
    sqlx::query(statement)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(map_sqlx_error)
}

async fn set_schema_version(pool: &SqlitePool, version: i64) -> Result<(), String> {
    let statement = format!("PRAGMA user_version = {version}");

    execute(pool, &statement).await
}

fn map_sqlx_error(error: sqlx::Error) -> String {
    format!("SQLite storage error: {error}")
}
