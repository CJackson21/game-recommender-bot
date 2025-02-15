CREATE TABLE IF NOT EXISTS games (
    id SERIAL PRIMARY KEY,
    steam_id TEXT NOT NULL,
    name TEXT NOT NULL,
    playtime INT NOT NULL DEFAULT 0,
    last_updated TIMESTAMP DEFAULT NOW()
)-- Add migration script here
