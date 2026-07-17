CREATE TABLE IF NOT EXISTS reading_history (
    comic_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    image TEXT NOT NULL,
    chapter_id TEXT NOT NULL,
    chapter_title TEXT NOT NULL,
    page_index INTEGER NOT NULL,
    page_count INTEGER NOT NULL,
    last_read_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_reading_history_last_read_at
    ON reading_history(last_read_at DESC);
