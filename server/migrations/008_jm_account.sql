CREATE TABLE IF NOT EXISTS jm_account (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    username TEXT NOT NULL,
    password_cipher TEXT NOT NULL,
    auto_login INTEGER NOT NULL,
    auto_sign_in INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
