use super::{cache_index_key, storage::temporary_path, CacheConfig, ImageCache};
use crate::reader::PageImageFormat;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{
    future::{poll_fn, Future},
    io::Cursor,
    path::PathBuf,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    task::Poll,
    time::{Duration, Instant},
};
use tokio::{fs, task::JoinSet};

static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[tokio::test]
async fn dimensions_survive_eviction_and_restart_without_loading_image_files() {
    let db = test_db().await;
    let root = test_cache_dir("persist-dimensions");
    let cache =
        ImageCache::new_with_cache_dir(db.clone(), CacheConfig { max_size_bytes: 1 }, root.clone())
            .await
            .unwrap();
    let key = super::reader_page_cache_key("1001", 0);
    let mut data = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(12, 37)
        .write_to(&mut data, image::ImageFormat::Png)
        .unwrap();
    cache
        .put_reader_page(&key, PageImageFormat::Png, data.get_ref())
        .await
        .unwrap();
    assert!(cache.get_reader_page(&key).await.unwrap().is_none());
    drop(cache);

    let restored =
        ImageCache::new_with_cache_dir(db, CacheConfig { max_size_bytes: 1 }, root.clone())
            .await
            .unwrap();
    assert_eq!(
        restored.reader_dimensions("1001").await.unwrap()[&key],
        crate::reader::ImageDimensions {
            width: 12,
            height: 37
        }
    );
    assert!(restored.reader_dimensions("100").await.unwrap().is_empty());
    restored.clear().await.unwrap();
    assert!(restored.reader_dimensions("1001").await.unwrap().is_empty());
    fs::remove_dir_all(root).await.unwrap();
}

#[tokio::test]
async fn legacy_cache_and_offline_sizes_are_visible_immediately_and_flushed_in_a_batch() {
    let (cache, root) = test_cache("backfill-dimensions", 1_000_000).await;
    let key = super::reader_page_cache_key("1002", 0);
    let mut data = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(7, 19)
        .write_to(&mut data, image::ImageFormat::Png)
        .unwrap();
    cache
        .put_with_extension(&key, "png", data.get_ref())
        .await
        .unwrap();
    assert!(cache
        .repository
        .reader_dimensions("1002:")
        .await
        .unwrap()
        .is_empty());
    cache.get_reader_page(&key).await.unwrap().unwrap();
    assert_eq!(
        cache.reader_dimensions("1002").await.unwrap()[&key].height,
        19
    );
    let offline_key = super::reader_page_cache_key("1002", 1);
    cache
        .remember_reader_dimensions(&offline_key, PageImageFormat::Png, data.get_ref())
        .await;
    cache.flush_accesses().await.unwrap();
    assert_eq!(
        cache
            .repository
            .reader_dimensions("1002:")
            .await
            .unwrap()
            .len(),
        2
    );
    assert!(cache.pending_dimensions.lock().await.is_empty());
    let replacement = crate::reader::ImageDimensions {
        width: 12,
        height: 27,
    };
    cache
        .remember_reader_dimensions(&key, PageImageFormat::Png, data.get_ref())
        .await;
    cache
        .repository
        .upsert_dimensions(&key, replacement)
        .await
        .unwrap();
    assert_eq!(
        cache.reader_dimensions("1002").await.unwrap()[&key],
        replacement
    );
    cache.flush_accesses().await.unwrap();
    assert_eq!(
        cache.repository.reader_dimensions("1002:").await.unwrap()[&key],
        replacement
    );
    fs::remove_dir_all(root).await.unwrap();
}

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
async fn eviction_allows_queued_reads_before_reaching_half_capacity() {
    let (cache, root, _) = seeded_cache("evict-reader", 257, 1, 256).await;
    let gate = cache.operation_lock.read().await;
    let mut eviction = Box::pin(cache.evict_if_needed());
    assert_pending(eviction.as_mut()).await;
    let mut read = Box::pin(cache.get_cover("old-0100"));
    assert_pending(read.as_mut()).await;
    drop(gate);

    let (evicted, read) = tokio::join!(eviction, read);
    assert_eq!(read.unwrap(), Some(vec![1]));
    assert_eq!(evicted.unwrap(), 129);
    assert_eq!(cache.get_cover("old-0100").await.unwrap(), Some(vec![1]));
    assert_eq!(cache.stats().await.unwrap().size_bytes, 128);
    assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 128);
    fs::remove_dir_all(root).await.unwrap();
}

#[tokio::test]
async fn eviction_preserves_queued_replacements_and_accounts_for_their_size() {
    let (cache, root, _) = seeded_cache("evict-writer", 257, 1, 256).await;
    let gate = cache.operation_lock.read().await;
    let mut eviction = Box::pin(cache.evict_if_needed());
    assert_pending(eviction.as_mut()).await;
    let mut replacement = Box::pin(async {
        cache.put_cover("old-0100", &[9; 80]).await.unwrap();
        cache.current_size_bytes.load(Ordering::Relaxed)
    });
    assert_pending(replacement.as_mut()).await;
    drop(gate);

    let (evicted, size_at_write_completion) = tokio::join!(eviction, replacement);
    assert!(size_at_write_completion > 256);
    assert_eq!(evicted.unwrap(), 208);
    assert_eq!(
        cache.get_cover("old-0100").await.unwrap(),
        Some(vec![9; 80])
    );
    assert_eq!(cache.stats().await.unwrap().size_bytes, 128);
    assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 128);
    fs::remove_dir_all(root).await.unwrap();
}

#[tokio::test]
async fn eviction_yields_to_clear_and_preserves_subsequent_writes() {
    let (cache, root, _) = seeded_cache("evict-clear", 257, 1, 256).await;
    let gate = cache.operation_lock.read().await;
    let mut eviction = Box::pin(cache.evict_if_needed());
    assert_pending(eviction.as_mut()).await;
    let mut clear = Box::pin(async {
        cache.clear().await.unwrap();
        cache.put_cover("after-clear", &[7; 4]).await.unwrap();
    });
    assert_pending(clear.as_mut()).await;
    drop(gate);

    let (evicted, ()) = tokio::join!(eviction, clear);
    assert!((1..=64).contains(&evicted.unwrap()));
    assert_eq!(
        cache.get_cover("after-clear").await.unwrap(),
        Some(vec![7; 4])
    );
    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.size_bytes, 4);
    assert_eq!(stats.entry_count, 1);
    assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 4);
    fs::remove_dir_all(root).await.unwrap();
}

#[tokio::test]
async fn eviction_keeps_accounting_for_deletions_before_a_file_error() {
    let (cache, root, _) = seeded_cache("evict-file-error", 5, 4, 10).await;
    let invalid_path = cache.storage.path_for("old-0001", "img");
    fs::remove_file(&invalid_path).await.unwrap();
    fs::create_dir(&invalid_path).await.unwrap();

    assert!(cache.evict_if_needed().await.is_err());
    assert_eq!(
        cache.repository.entry_size("old-0000:img").await.unwrap(),
        0
    );
    assert_eq!(cache.stats().await.unwrap().size_bytes, 16);
    assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 16);

    fs::remove_dir(&invalid_path).await.unwrap();
    fs::write(&invalid_path, [1; 4]).await.unwrap();
    assert_eq!(cache.evict_if_needed().await.unwrap(), 3);
    assert_eq!(cache.stats().await.unwrap().size_bytes, 4);
    assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 4);
    fs::remove_dir_all(root).await.unwrap();
}

#[tokio::test]
async fn eviction_recovers_after_an_index_batch_fails() {
    let (cache, root, db) = seeded_cache("evict-sql-error", 5, 4, 10).await;
    sqlx::query(
        "CREATE TRIGGER fail_eviction BEFORE DELETE ON cache_index \
         WHEN OLD.key = 'old-0001:img' BEGIN SELECT RAISE(ABORT, 'test deletion failure'); END",
    )
    .execute(&db)
    .await
    .unwrap();

    assert!(cache.evict_if_needed().await.is_err());
    assert!(!cache.storage.path_for("old-0001", "img").exists());
    assert_eq!(
        cache.repository.entry_size("old-0001:img").await.unwrap(),
        4
    );
    assert_eq!(
        cache.current_size_bytes.load(Ordering::Relaxed),
        cache.stats().await.unwrap().size_bytes
    );

    sqlx::query("DROP TRIGGER fail_eviction")
        .execute(&db)
        .await
        .unwrap();
    cache.evict_if_needed().await.unwrap();
    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.size_bytes, 4);
    assert_eq!(stats.entry_count, 1);
    assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 4);
    assert_eq!(cache.storage.scan_files().await.unwrap().len(), 1);
    fs::remove_dir_all(root).await.unwrap();
}

#[tokio::test]
#[ignore = "disk-backed near-capacity eviction latency measurement"]
async fn measures_cache_read_latency_during_near_capacity_eviction() {
    let root = test_cache_dir("evict-latency");
    fs::create_dir_all(&root).await.unwrap();
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(root.join("cache.sqlite"))
                .create_if_missing(true),
        )
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&db).await.unwrap();
    let cache = ImageCache::new_with_cache_dir(
        db.clone(),
        CacheConfig {
            max_size_bytes: 64 * 1024 * 1024,
        },
        root.join("images"),
    )
    .await
    .unwrap();
    seed_entries(&cache, &db, 4097, 16 * 1024).await;
    let gate = cache.operation_lock.read().await;
    let done = AtomicBool::new(false);
    let mut eviction = Box::pin(async {
        let started = Instant::now();
        let count = cache.evict_if_needed().await.unwrap();
        done.store(true, Ordering::Relaxed);
        (count, started.elapsed())
    });
    assert_pending(eviction.as_mut()).await;
    let mut reads = Box::pin(async {
        let mut waits = Vec::new();
        while !done.load(Ordering::Relaxed) || waits.is_empty() {
            let started = Instant::now();
            assert!(cache.get_cover("old-4000").await.unwrap().is_some());
            waits.push(started.elapsed().as_secs_f64() * 1000.0);
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        waits
    });
    assert_pending(reads.as_mut()).await;
    drop(gate);
    let ((evicted, elapsed), mut waits) = tokio::join!(eviction, reads);
    let first = waits[0];
    waits.sort_by(f64::total_cmp);
    eprintln!(
        "64 MiB cache, 4097 x 16 KiB files: evicted={evicted}, total_ms={:.2}, reads={}, first_ms={first:.2}, p95_ms={:.2}, max_ms={:.2}",
        elapsed.as_secs_f64() * 1000.0,
        waits.len(),
        waits[(waits.len() - 1) * 95 / 100],
        waits.last().unwrap(),
    );
    assert_eq!(evicted, 2049);
    assert_eq!(cache.stats().await.unwrap().size_bytes, 32 * 1024 * 1024);
    drop(cache);
    db.close().await;
    fs::remove_dir_all(root).await.unwrap();
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

#[tokio::test]
async fn roundtrips_complete_reader_images_without_changing_bytes() {
    let (cache, cache_dir) = test_cache("reader-formats", 10_000).await;
    let mut total_size = 0;
    for (index, format) in PageImageFormat::supported().into_iter().enumerate() {
        let key = format!("reader:{index}");
        let data = page_image(format);
        assert!(format.is_complete(&data));
        cache.put_reader_page(&key, format, &data).await.unwrap();
        let cached = cache.get_reader_page(&key).await.unwrap().unwrap();
        assert_eq!(cached.format, format);
        assert_eq!(cached.data, data);
        total_size += data.len() as i64;
    }
    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.entry_count, 4);
    assert_eq!(stats.size_bytes, total_size);
    assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), total_size);
    fs::remove_dir_all(cache_dir).await.unwrap();
}

#[tokio::test]
async fn rejects_truncated_or_mismatched_reader_images_before_cache_writes() {
    let (cache, cache_dir) = test_cache("reader-invalid-write", 10_000).await;
    for format in PageImageFormat::supported() {
        let mut data = page_image(format);
        data.pop();
        assert_eq!(PageImageFormat::detect(&data).unwrap(), format);
        assert!(cache
            .put_reader_page("reader:0", format, &data)
            .await
            .is_err());
        assert!(!cache
            .storage
            .path_for("reader:0", format.extension())
            .exists());
    }
    assert!(cache
        .put_reader_page(
            "reader:0",
            PageImageFormat::WebP,
            &page_image(PageImageFormat::Png)
        )
        .await
        .is_err());
    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.entry_count, 0);
    assert_eq!(stats.size_bytes, 0);
    assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 0);
    assert!(cache.storage.scan_files().await.unwrap().is_empty());
    fs::remove_dir_all(cache_dir).await.unwrap();
}

#[tokio::test]
async fn removes_truncated_reader_files_indexes_and_pending_accesses() {
    let (cache, cache_dir) = test_cache("reader-invalid-read", 10_000).await;
    cache.put_cover("unrelated-cover", &[7; 17]).await.unwrap();
    for (index, format) in PageImageFormat::supported().into_iter().enumerate() {
        let key = format!("legacy:{index}");
        let mut data = page_image(format);
        data.pop();
        // Seed a legacy entry through the raw storage path, bypassing new write validation.
        cache
            .put_with_extension(&key, format.extension(), &data)
            .await
            .unwrap();
        let index_key = cache_index_key(&key, format.extension());
        cache.record_access(&index_key).await;
        assert!(cache.get_reader_page(&key).await.unwrap().is_none());
        assert!(!cache.storage.path_for(&key, format.extension()).exists());
        assert_eq!(cache.repository.entry_size(&index_key).await.unwrap(), 0);
        assert!(!cache.pending_accesses.lock().await.contains_key(&index_key));
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.entry_count, 1);
        assert_eq!(stats.size_bytes, 17);
        assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 17);
    }
    assert_eq!(cache.flush_accesses().await.unwrap(), 0);
    assert_eq!(
        cache.get_cover("unrelated-cover").await.unwrap(),
        Some(vec![7; 17])
    );
    fs::remove_dir_all(cache_dir).await.unwrap();
}

#[tokio::test]
async fn removes_a_mismatched_format_and_reads_a_valid_alternative() {
    let (cache, cache_dir) = test_cache("reader-format-mismatch", 10_000).await;
    let data = page_image(PageImageFormat::Png);
    cache
        .put_with_extension("reader:0", "webp", &data)
        .await
        .unwrap();
    cache
        .put_reader_page("reader:0", PageImageFormat::Png, &data)
        .await
        .unwrap();
    let page = cache.get_reader_page("reader:0").await.unwrap().unwrap();
    assert_eq!(page.format, PageImageFormat::Png);
    assert_eq!(page.data, data);
    assert!(!cache.storage.path_for("reader:0", "webp").exists());
    assert_eq!(cache.repository.entry_size("reader:0").await.unwrap(), 0);
    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.entry_count, 1);
    assert_eq!(stats.size_bytes, data.len() as i64);
    assert_eq!(
        cache.current_size_bytes.load(Ordering::Relaxed),
        data.len() as i64
    );
    fs::remove_dir_all(cache_dir).await.unwrap();
}

#[tokio::test]
async fn an_invalid_replacement_preserves_the_existing_reader_image() {
    let (cache, cache_dir) = test_cache("reader-invalid-replacement", 10_000).await;
    let data = page_image(PageImageFormat::Png);
    cache
        .put_reader_page("reader:0", PageImageFormat::Png, &data)
        .await
        .unwrap();
    assert!(cache
        .put_reader_page("reader:0", PageImageFormat::Png, &data[..data.len() - 1])
        .await
        .is_err());
    let page = cache.get_reader_page("reader:0").await.unwrap().unwrap();
    assert_eq!(page.data, data);
    assert_eq!(cache.stats().await.unwrap().size_bytes, data.len() as i64);
    assert_eq!(
        cache.current_size_bytes.load(Ordering::Relaxed),
        data.len() as i64
    );
    fs::remove_dir_all(cache_dir).await.unwrap();
}

#[tokio::test]
async fn concurrent_invalid_reader_reads_remove_the_index_size_only_once() {
    let (cache, cache_dir) = test_cache("reader-concurrent-repair", 10_000).await;
    cache
        .put_cover("unrelated-cover", &[7; 1000])
        .await
        .unwrap();
    let mut data = page_image(PageImageFormat::Png);
    data.pop();
    cache
        .put_with_extension("reader:0", "png", &data)
        .await
        .unwrap();
    let cache = Arc::new(cache);
    let mut readers = JoinSet::new();
    for _ in 0..8 {
        let cache = cache.clone();
        readers.spawn(async move { cache.get_reader_page("reader:0").await });
    }
    while let Some(result) = readers.join_next().await {
        assert!(result.unwrap().unwrap().is_none());
    }
    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.entry_count, 1);
    assert_eq!(stats.size_bytes, 1000);
    assert_eq!(cache.current_size_bytes.load(Ordering::Relaxed), 1000);
    fs::remove_dir_all(cache_dir).await.unwrap();
}

#[tokio::test]
async fn rechecking_a_stale_invalid_read_preserves_a_complete_replacement() {
    let (cache, cache_dir) = test_cache("reader-recheck-replacement", 10_000).await;
    let data = page_image(PageImageFormat::Png);
    cache
        .put_with_extension("reader:0", "png", &data[..data.len() - 1])
        .await
        .unwrap();
    let stale = cache
        .get_with_extension("reader:0", "png")
        .await
        .unwrap()
        .unwrap();
    assert!(!PageImageFormat::Png.is_complete(&stale));
    cache
        .put_reader_page("reader:0", PageImageFormat::Png, &data)
        .await
        .unwrap();
    let page = cache
        .recheck_reader_page("reader:0", PageImageFormat::Png)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(page.data, data);
    assert!(cache.storage.path_for("reader:0", "png").exists());
    assert_eq!(cache.stats().await.unwrap().entry_count, 1);
    assert_eq!(
        cache.current_size_bytes.load(Ordering::Relaxed),
        data.len() as i64
    );
    fs::remove_dir_all(cache_dir).await.unwrap();
}

fn page_image(format: PageImageFormat) -> Vec<u8> {
    let image =
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(2, 2, image::Rgb([1, 2, 3])));
    let format = match format {
        PageImageFormat::Gif => image::ImageFormat::Gif,
        PageImageFormat::Jpeg => image::ImageFormat::Jpeg,
        PageImageFormat::Png => image::ImageFormat::Png,
        PageImageFormat::WebP => image::ImageFormat::WebP,
    };
    let mut bytes = Cursor::new(Vec::new());
    image
        .write_to(&mut bytes, format)
        .expect("encode test image");
    bytes.into_inner()
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

async fn assert_pending<F: Future>(mut future: Pin<&mut F>) {
    poll_fn(|cx| {
        assert!(future.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
}

async fn seeded_cache(
    name: &str,
    count: usize,
    entry_size: usize,
    max_size_bytes: i64,
) -> (ImageCache, PathBuf, sqlx::SqlitePool) {
    let db = test_db().await;
    let root = test_cache_dir(name);
    let cache =
        ImageCache::new_with_cache_dir(db.clone(), CacheConfig { max_size_bytes }, root.clone())
            .await
            .unwrap();
    seed_entries(&cache, &db, count, entry_size).await;
    (cache, root, db)
}

async fn seed_entries(cache: &ImageCache, db: &sqlx::SqlitePool, count: usize, entry_size: usize) {
    let data = vec![1; entry_size];
    let mut transaction = db.begin().await.unwrap();
    for index in 0..count {
        let key = format!("old-{index:04}");
        let path = cache.storage.path_for(&key, "img");
        fs::write(&path, &data).await.unwrap();
        sqlx::query(
            "INSERT INTO cache_index (key, path, size, created_at, accessed_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(cache_index_key(&key, "img"))
        .bind(path.to_string_lossy().as_ref())
        .bind(entry_size as i64)
        .bind(index as i64)
        .bind(index as i64)
        .execute(&mut *transaction)
        .await
        .unwrap();
    }
    transaction.commit().await.unwrap();
    cache
        .current_size_bytes
        .store((count * entry_size) as i64, Ordering::Relaxed);
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
