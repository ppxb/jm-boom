use super::{storage::temporary_path, CacheConfig, ImageCache};
use sqlx::sqlite::SqlitePoolOptions;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use tokio::fs;

static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[tokio::test]
async fn tracks_replacement_size_delta_and_resets_after_clear() {
    let (cache, cache_dir) = test_cache("replace", 100).await;

    cache
        .put_cover("cover", &[1; 20])
        .await
        .expect("write initial cache entry");
    assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 20);

    cache
        .put_cover("cover", &[2; 8])
        .await
        .expect("replace cache entry");
    assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 8);
    assert_eq!(
        cache.get_cover("cover").await.expect("read replacement"),
        Some(vec![2; 8])
    );
    let stats = cache.stats().await.expect("load cache stats");
    assert_eq!(stats.size_bytes, 8);
    assert_eq!(stats.entry_count, 1);

    cache.clear().await.expect("clear cache");
    assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 0);
    fs::remove_dir_all(cache_dir)
        .await
        .expect("remove test cache");
}

#[tokio::test]
async fn evicts_to_half_capacity_only_after_limit_is_exceeded() {
    let (cache, cache_dir) = test_cache("evict", 10).await;

    cache.put_cover("one", &[1; 4]).await.expect("write one");
    cache.put_cover("two", &[2; 4]).await.expect("write two");
    let before = cache.stats().await.expect("load pre-eviction stats");
    assert_eq!(before.size_bytes, 8);
    assert_eq!(before.entry_count, 2);

    cache
        .put_cover("three", &[3; 4])
        .await
        .expect("write over capacity");
    let after = cache.stats().await.expect("load post-eviction stats");
    assert_eq!(after.size_bytes, 4);
    assert_eq!(after.entry_count, 1);
    assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 4);

    fs::remove_dir_all(cache_dir)
        .await
        .expect("remove test cache");
}

#[tokio::test]
async fn calibrates_size_from_repaired_index_on_startup() {
    let db = test_db().await;
    let cache_dir = test_cache_dir("calibrate");
    fs::create_dir_all(&cache_dir)
        .await
        .expect("create test cache directory");
    let path = cache_dir.join("seed.img");
    fs::write(&path, [1; 7])
        .await
        .expect("write seeded cache file");
    sqlx::query(
        "INSERT INTO cache_index (key, path, size, created_at, accessed_at) VALUES (?, ?, ?, 1, 1)",
    )
    .bind("seed:img")
    .bind(path.to_string_lossy().as_ref())
    .bind(99_i64)
    .execute(&db)
    .await
    .expect("seed cache index");

    let cache = ImageCache::new_with_cache_dir(
        db,
        CacheConfig {
            max_size_bytes: 100,
        },
        cache_dir.clone(),
    )
    .await
    .expect("create calibrated cache");

    assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 7);
    assert_eq!(cache.stats().await.expect("load cache stats").size_bytes, 7);
    fs::remove_dir_all(cache_dir)
        .await
        .expect("remove test cache");
}

#[tokio::test]
async fn ignores_and_repairs_an_uncommitted_cache_file() {
    let db = test_db().await;
    let cache_dir = test_cache_dir("uncommitted");
    let cache = ImageCache::new_with_cache_dir(
        db.clone(),
        CacheConfig {
            max_size_bytes: 100,
        },
        cache_dir.clone(),
    )
    .await
    .expect("create initial cache");
    let final_path = cache.storage.path_for("cover", "img");
    let temporary_path = temporary_path(&final_path).expect("create temporary cache path");
    fs::write(&temporary_path, [1; 7])
        .await
        .expect("write interrupted cache file");

    assert!(cache
        .get_cover("cover")
        .await
        .expect("read cache before repair")
        .is_none());
    drop(cache);

    let repaired = ImageCache::new_with_cache_dir(
        db,
        CacheConfig {
            max_size_bytes: 100,
        },
        cache_dir.clone(),
    )
    .await
    .expect("repair cache");
    assert!(!temporary_path.exists());
    assert!(repaired
        .get_cover("cover")
        .await
        .expect("read repaired cache")
        .is_none());

    drop(repaired);
    fs::remove_dir_all(cache_dir)
        .await
        .expect("remove test cache");
}

async fn test_cache(name: &str, max_size_bytes: i64) -> (ImageCache, PathBuf) {
    let cache_dir = test_cache_dir(name);
    let cache = ImageCache::new_with_cache_dir(
        test_db().await,
        CacheConfig { max_size_bytes },
        cache_dir.clone(),
    )
    .await
    .expect("create test cache");
    (cache, cache_dir)
}

async fn test_db() -> sqlx::SqlitePool {
    let db = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect in-memory sqlite");
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("run migrations");
    db
}

fn test_cache_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "jm-boom-image-cache-{name}-{}-{}",
        std::process::id(),
        TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ))
}
