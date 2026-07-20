CREATE TABLE cardio_entries (
    id BIGSERIAL PRIMARY KEY,
    session_id BIGINT NOT NULL REFERENCES workout_sessions(id) ON DELETE CASCADE,
    exercise_id BIGINT NOT NULL REFERENCES exercises(id) ON DELETE RESTRICT,
    cardio_type TEXT NOT NULL,
    duration_seconds INTEGER NOT NULL,
    distance_meters DOUBLE PRECISION,
    intensity_level INTEGER,
    speed_kph DOUBLE PRECISION,
    incline_percent DOUBLE PRECISION,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT cardio_entries_type_not_blank CHECK (btrim(cardio_type) <> ''),
    CONSTRAINT cardio_entries_duration_range CHECK (duration_seconds BETWEEN 1 AND 86400),
    CONSTRAINT cardio_entries_distance_meters_range CHECK (
        distance_meters IS NULL OR (distance_meters >= 0 AND distance_meters <= 1000000)
    ),
    CONSTRAINT cardio_entries_intensity_level_range CHECK (
        intensity_level IS NULL OR intensity_level BETWEEN 1 AND 10
    ),
    CONSTRAINT cardio_entries_speed_kph_range CHECK (
        speed_kph IS NULL OR (speed_kph >= 0 AND speed_kph <= 80)
    ),
    CONSTRAINT cardio_entries_incline_percent_range CHECK (
        incline_percent IS NULL OR (incline_percent >= -20 AND incline_percent <= 40)
    ),
    CONSTRAINT cardio_entries_notes_not_blank CHECK (
        notes IS NULL OR btrim(notes) <> ''
    )
);

CREATE INDEX cardio_entries_session_idx ON cardio_entries (session_id, id);
CREATE INDEX cardio_entries_exercise_idx ON cardio_entries (exercise_id);
CREATE INDEX cardio_entries_type_idx ON cardio_entries (cardio_type);

CREATE TRIGGER cardio_entries_set_updated_at
BEFORE UPDATE ON cardio_entries
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
