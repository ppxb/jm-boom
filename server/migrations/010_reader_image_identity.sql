-- Reader caches can be rebuilt. Download files are migrated separately before workers start.
DELETE FROM cache_index WHERE key NOT LIKE 'covers:%';
DELETE FROM reader_page_dimensions;

ALTER TABLE download_manifests ADD COLUMN storage_version INTEGER NOT NULL DEFAULT 0;
