-- Your SQL goes here
CREATE TABLE IF NOT EXISTS users(
    id SERIAL PRIMARY KEY,
    discord_id VARCHAR NOT NULL
);

