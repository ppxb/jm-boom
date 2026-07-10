-- 缓存索引表
CREATE TABLE IF NOT EXISTS cache_index (
    key TEXT PRIMARY KEY,
    path TEXT NOT NULL,
    size INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    accessed_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cache_accessed ON cache_index(accessed_at);
