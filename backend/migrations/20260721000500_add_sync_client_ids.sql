ALTER TABLE workout_sessions
ADD COLUMN client_id TEXT,
ADD CONSTRAINT workout_sessions_client_id_not_blank CHECK (
    client_id IS NULL OR btrim(client_id) <> ''
);

ALTER TABLE strength_sets
ADD COLUMN client_id TEXT,
ADD CONSTRAINT strength_sets_client_id_not_blank CHECK (
    client_id IS NULL OR btrim(client_id) <> ''
);

ALTER TABLE cardio_entries
ADD COLUMN client_id TEXT,
ADD CONSTRAINT cardio_entries_client_id_not_blank CHECK (
    client_id IS NULL OR btrim(client_id) <> ''
);

CREATE UNIQUE INDEX workout_sessions_user_client_id_unique
ON workout_sessions (user_sub, client_id)
WHERE client_id IS NOT NULL;

CREATE UNIQUE INDEX strength_sets_session_client_id_unique
ON strength_sets (session_id, client_id)
WHERE client_id IS NOT NULL;

CREATE UNIQUE INDEX cardio_entries_session_client_id_unique
ON cardio_entries (session_id, client_id)
WHERE client_id IS NOT NULL;
