-- Your SQL goes here
CREATE TABLE IF NOT EXISTS roles(
    id SERIAL PRIMARY KEY ,
    role_id VARCHAR NOT NULL,
    guild VARCHAR NOT NULL
);
CREATE TABLE IF NOT EXISTS crossroles(
    id SERIAL PRIMARY KEY,
    roles INTEGER NOT NULL,
    color VARCHAR NOT NULL,
    mentionable BOOLEAN NOT NULL DEFAULT 'f',
    guild VARCHAR NOT NULL,
    users INTEGER NOT NULL,
    FOREIGN KEY (users) REFERENCES users (id),
    FOREIGN KEY (roles) REFERENCES roles (id)
);
CREATE TABLE IF NOT EXISTS ban(
    id SERIAL PRIMARY KEY NOT NULL,
    users INTEGER NOT NULL,
    guild VARCHAR,
    end_epoch VARCHAR,
    FOREIGN KEY (users) REFERENCES users (id)
)