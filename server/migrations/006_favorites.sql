CREATE TABLE IF NOT EXISTS favorites (
    comic_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    description TEXT NOT NULL,
    image TEXT NOT NULL,
    tags TEXT NOT NULL,
    favorited_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_favorites_favorited_at
    ON favorites(favorited_at DESC);
