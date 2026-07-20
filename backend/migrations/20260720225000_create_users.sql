CREATE TABLE users (
    sub TEXT PRIMARY KEY,
    email TEXT NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    name TEXT,
    picture_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT users_sub_not_blank CHECK (btrim(sub) <> ''),
    CONSTRAINT users_email_not_blank CHECK (btrim(email) <> '')
);

CREATE UNIQUE INDEX users_email_lower_unique ON users (lower(email));
CREATE INDEX users_last_seen_at_idx ON users (last_seen_at DESC);

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_set_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
