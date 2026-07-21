CREATE TABLE user_active_splits (
    user_sub TEXT PRIMARY KEY REFERENCES users(sub) ON DELETE CASCADE,
    split_template_id BIGINT NOT NULL REFERENCES split_templates(id) ON DELETE RESTRICT,
    template_slug TEXT NOT NULL,
    template_name TEXT NOT NULL,
    depth_level TEXT NOT NULL,
    schedule TEXT[] NOT NULL,
    training_days_per_cycle INTEGER NOT NULL,
    rest_days_per_cycle INTEGER NOT NULL,
    selected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT user_active_splits_template_slug_not_blank CHECK (btrim(template_slug) <> ''),
    CONSTRAINT user_active_splits_template_name_not_blank CHECK (btrim(template_name) <> ''),
    CONSTRAINT user_active_splits_depth_allowed CHECK (depth_level IN ('simple', 'advanced')),
    CONSTRAINT user_active_splits_schedule_length CHECK (cardinality(schedule) BETWEEN 1 AND 14),
    CONSTRAINT user_active_splits_training_days_range CHECK (training_days_per_cycle BETWEEN 1 AND 14),
    CONSTRAINT user_active_splits_rest_days_range CHECK (rest_days_per_cycle BETWEEN 0 AND 13),
    CONSTRAINT user_active_splits_cycle_count_matches_schedule CHECK (
        training_days_per_cycle + rest_days_per_cycle = cardinality(schedule)
    )
);

CREATE INDEX user_active_splits_template_idx
ON user_active_splits (split_template_id);

CREATE TRIGGER user_active_splits_set_updated_at
BEFORE UPDATE ON user_active_splits
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
