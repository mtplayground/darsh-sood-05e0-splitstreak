CREATE TABLE workout_sessions (
    id BIGSERIAL PRIMARY KEY,
    user_sub TEXT NOT NULL REFERENCES users(sub) ON DELETE CASCADE,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT workout_sessions_completed_after_start CHECK (
        completed_at IS NULL OR completed_at >= started_at
    ),
    CONSTRAINT workout_sessions_notes_not_blank CHECK (
        notes IS NULL OR btrim(notes) <> ''
    )
);

CREATE INDEX workout_sessions_user_started_idx
ON workout_sessions (user_sub, started_at DESC);

CREATE TRIGGER workout_sessions_set_updated_at
BEFORE UPDATE ON workout_sessions
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE strength_sets (
    id BIGSERIAL PRIMARY KEY,
    session_id BIGINT NOT NULL REFERENCES workout_sessions(id) ON DELETE CASCADE,
    exercise_id BIGINT NOT NULL REFERENCES exercises(id) ON DELETE RESTRICT,
    set_number INTEGER NOT NULL,
    reps INTEGER NOT NULL,
    weight_kg DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT strength_sets_set_number_range CHECK (set_number BETWEEN 1 AND 200),
    CONSTRAINT strength_sets_reps_range CHECK (reps BETWEEN 1 AND 1000),
    CONSTRAINT strength_sets_weight_kg_range CHECK (weight_kg >= 0 AND weight_kg <= 2000),
    CONSTRAINT strength_sets_unique_set UNIQUE (session_id, exercise_id, set_number)
);

CREATE INDEX strength_sets_session_idx ON strength_sets (session_id, set_number);
CREATE INDEX strength_sets_exercise_idx ON strength_sets (exercise_id);

CREATE TRIGGER strength_sets_set_updated_at
BEFORE UPDATE ON strength_sets
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
