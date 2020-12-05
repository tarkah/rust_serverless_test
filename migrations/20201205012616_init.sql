-- Add migration script here
CREATE TABLE IF NOT EXISTS test
(
    id          BIGSERIAL PRIMARY KEY,
    description TEXT    NOT NULL
);
