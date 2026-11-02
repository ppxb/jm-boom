CREATE TABLE reader_page_dimensions (
    cache_key TEXT PRIMARY KEY NOT NULL,
    width INTEGER NOT NULL CHECK (width > 0),
    height INTEGER NOT NULL CHECK (height > 0)
);
