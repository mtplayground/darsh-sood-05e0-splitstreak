CREATE TABLE split_templates (
    id BIGSERIAL PRIMARY KEY,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    depth_level TEXT NOT NULL,
    schedule TEXT[] NOT NULL,
    training_days_per_cycle INTEGER NOT NULL,
    rest_days_per_cycle INTEGER NOT NULL,
    rationale TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT split_templates_slug_not_blank CHECK (btrim(slug) <> ''),
    CONSTRAINT split_templates_name_not_blank CHECK (btrim(name) <> ''),
    CONSTRAINT split_templates_depth_allowed CHECK (depth_level IN ('simple', 'advanced')),
    CONSTRAINT split_templates_schedule_length CHECK (cardinality(schedule) BETWEEN 1 AND 14),
    CONSTRAINT split_templates_training_days_range CHECK (training_days_per_cycle BETWEEN 1 AND 14),
    CONSTRAINT split_templates_rest_days_range CHECK (rest_days_per_cycle BETWEEN 0 AND 13),
    CONSTRAINT split_templates_cycle_count_matches_schedule CHECK (
        training_days_per_cycle + rest_days_per_cycle = cardinality(schedule)
    ),
    CONSTRAINT split_templates_rationale_not_blank CHECK (btrim(rationale) <> '')
);

CREATE UNIQUE INDEX split_templates_slug_unique ON split_templates (slug);
CREATE INDEX split_templates_depth_name_idx ON split_templates (depth_level, name);

CREATE TRIGGER split_templates_set_updated_at
BEFORE UPDATE ON split_templates
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

INSERT INTO split_templates (
    slug,
    name,
    depth_level,
    schedule,
    training_days_per_cycle,
    rest_days_per_cycle,
    rationale
)
VALUES
    (
        'full-body-3-day',
        'Full-body 3 day',
        'simple',
        ARRAY['Full body', 'Rest', 'Full body', 'Rest', 'Full body', 'Rest', 'Rest'],
        3,
        4,
        'Three total-body sessions give beginners frequent practice on main lifts while leaving recovery days between hard efforts.'
    ),
    (
        'upper-lower-4-day',
        'Upper/lower 4 day',
        'simple',
        ARRAY['Upper', 'Lower', 'Rest', 'Upper', 'Lower', 'Rest', 'Rest'],
        4,
        3,
        'Alternating upper and lower days raises weekly volume without making any single session too long.'
    ),
    (
        'push-pull-legs-6-day',
        'Push/pull/legs 6 day',
        'advanced',
        ARRAY['Push', 'Pull', 'Legs', 'Push', 'Pull', 'Legs', 'Rest'],
        6,
        1,
        'Push, pull, and legs sessions separate overlapping muscle groups so advanced lifters can train often with focused volume.'
    ),
    (
        'push-pull-legs-rest-repeat',
        'Push/pull/legs rest repeat',
        'advanced',
        ARRAY['Push', 'Pull', 'Legs', 'Rest'],
        3,
        1,
        'A four-day repeating cycle keeps PPL frequency high while inserting recovery before the next push session.'
    ),
    (
        'powerbuilding-4-day',
        'Powerbuilding 4 day',
        'advanced',
        ARRAY['Upper strength', 'Lower strength', 'Rest', 'Push hypertrophy', 'Pull and legs hypertrophy', 'Rest', 'Rest'],
        4,
        3,
        'Heavy strength days and higher-rep hypertrophy days balance performance practice with enough accessory work for physique goals.'
    ),
    (
        'body-part-5-day',
        'Body-part 5 day',
        'advanced',
        ARRAY['Chest', 'Back', 'Legs', 'Shoulders', 'Arms', 'Rest', 'Rest'],
        5,
        2,
        'A classic body-part split concentrates volume into focused sessions for lifters who recover well from higher per-muscle workloads.'
    )
ON CONFLICT (slug) DO UPDATE
SET
    name = EXCLUDED.name,
    depth_level = EXCLUDED.depth_level,
    schedule = EXCLUDED.schedule,
    training_days_per_cycle = EXCLUDED.training_days_per_cycle,
    rest_days_per_cycle = EXCLUDED.rest_days_per_cycle,
    rationale = EXCLUDED.rationale;
