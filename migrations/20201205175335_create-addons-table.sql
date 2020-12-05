-- Add migration script here
CREATE TABLE IF NOT EXISTS addons
(
    id          BIGSERIAL PRIMARY KEY,
    repository  TEXT    NOT NULL    UNIQUE,
    repository_name TEXT    NOT NULL,
    external_id TEXT    NOT NULL,
    source  TEXT    NOT NULL,
    homepage    TEXT,
    description TEXT,
    image_url   TEXT,
    owner_name  TEXT,
    owner_image_url TEXT,
    total_download_count    INTEGER DEFAULT 0,
    addon_state TEXT    NOT NULL,
    addon_state_changed_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_available    BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION trigger_set_timestamp()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_timestamp
BEFORE UPDATE ON addons
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();