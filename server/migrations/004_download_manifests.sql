CREATE TABLE IF NOT EXISTS download_manifests (
    task_id TEXT NOT NULL,
    album_id TEXT NOT NULL,
    chapter_id TEXT NOT NULL,
    title TEXT NOT NULL,
    payload TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (task_id, chapter_id)
);

CREATE INDEX IF NOT EXISTS idx_download_manifests_chapter
    ON download_manifests(chapter_id, updated_at DESC);
