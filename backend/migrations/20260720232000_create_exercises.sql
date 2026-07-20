CREATE TABLE exercises (
    id BIGSERIAL PRIMARY KEY,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    modality TEXT NOT NULL,
    primary_muscle_group TEXT,
    equipment TEXT,
    aliases TEXT[] NOT NULL DEFAULT '{}',
    is_bodyweight BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT exercises_slug_not_blank CHECK (btrim(slug) <> ''),
    CONSTRAINT exercises_name_not_blank CHECK (btrim(name) <> ''),
    CONSTRAINT exercises_modality_allowed CHECK (modality IN ('strength', 'cardio'))
);

CREATE UNIQUE INDEX exercises_slug_unique ON exercises (slug);
CREATE INDEX exercises_modality_name_idx ON exercises (modality, name);
CREATE INDEX exercises_name_lower_idx ON exercises (lower(name));
CREATE INDEX exercises_aliases_idx ON exercises USING GIN (aliases);

CREATE TRIGGER exercises_set_updated_at
BEFORE UPDATE ON exercises
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

INSERT INTO exercises (slug, name, modality, primary_muscle_group, equipment, aliases, is_bodyweight)
VALUES
    ('barbell-back-squat', 'Barbell Back Squat', 'strength', 'legs', 'barbell', ARRAY['squat', 'back squat'], FALSE),
    ('front-squat', 'Front Squat', 'strength', 'legs', 'barbell', ARRAY['barbell front squat'], FALSE),
    ('deadlift', 'Deadlift', 'strength', 'posterior chain', 'barbell', ARRAY['barbell deadlift'], FALSE),
    ('romanian-deadlift', 'Romanian Deadlift', 'strength', 'hamstrings', 'barbell', ARRAY['rdl'], FALSE),
    ('bench-press', 'Bench Press', 'strength', 'chest', 'barbell', ARRAY['barbell bench press'], FALSE),
    ('incline-dumbbell-press', 'Incline Dumbbell Press', 'strength', 'chest', 'dumbbells', ARRAY['incline press'], FALSE),
    ('overhead-press', 'Overhead Press', 'strength', 'shoulders', 'barbell', ARRAY['shoulder press', 'strict press'], FALSE),
    ('push-up', 'Push-up', 'strength', 'chest', 'bodyweight', ARRAY['pushup', 'press-up'], TRUE),
    ('pull-up', 'Pull-up', 'strength', 'back', 'pull-up bar', ARRAY['pullup'], TRUE),
    ('chin-up', 'Chin-up', 'strength', 'back', 'pull-up bar', ARRAY['chinup'], TRUE),
    ('barbell-row', 'Barbell Row', 'strength', 'back', 'barbell', ARRAY['bent-over row'], FALSE),
    ('dumbbell-row', 'Dumbbell Row', 'strength', 'back', 'dumbbells', ARRAY['single-arm row'], FALSE),
    ('lat-pulldown', 'Lat Pulldown', 'strength', 'back', 'cable machine', ARRAY['pulldown'], FALSE),
    ('seated-cable-row', 'Seated Cable Row', 'strength', 'back', 'cable machine', ARRAY['cable row'], FALSE),
    ('leg-press', 'Leg Press', 'strength', 'legs', 'machine', ARRAY['machine leg press'], FALSE),
    ('leg-curl', 'Leg Curl', 'strength', 'hamstrings', 'machine', ARRAY['hamstring curl'], FALSE),
    ('leg-extension', 'Leg Extension', 'strength', 'quadriceps', 'machine', ARRAY['quad extension'], FALSE),
    ('walking-lunge', 'Walking Lunge', 'strength', 'legs', 'bodyweight', ARRAY['lunges'], TRUE),
    ('hip-thrust', 'Hip Thrust', 'strength', 'glutes', 'barbell', ARRAY['barbell hip thrust'], FALSE),
    ('calf-raise', 'Calf Raise', 'strength', 'calves', 'machine', ARRAY['standing calf raise'], FALSE),
    ('biceps-curl', 'Biceps Curl', 'strength', 'arms', 'dumbbells', ARRAY['curl', 'dumbbell curl'], FALSE),
    ('triceps-pushdown', 'Triceps Pushdown', 'strength', 'arms', 'cable machine', ARRAY['cable triceps pushdown'], FALSE),
    ('lateral-raise', 'Lateral Raise', 'strength', 'shoulders', 'dumbbells', ARRAY['side raise'], FALSE),
    ('plank', 'Plank', 'strength', 'core', 'bodyweight', ARRAY['front plank'], TRUE),
    ('crunch', 'Crunch', 'strength', 'core', 'bodyweight', ARRAY['ab crunch'], TRUE),
    ('treadmill-run', 'Treadmill Run', 'cardio', NULL, 'treadmill', ARRAY['run', 'running'], FALSE),
    ('outdoor-run', 'Outdoor Run', 'cardio', NULL, NULL, ARRAY['road run', 'trail run'], TRUE),
    ('walk', 'Walk', 'cardio', NULL, NULL, ARRAY['walking'], TRUE),
    ('cycling', 'Cycling', 'cardio', NULL, 'bike', ARRAY['bike', 'biking'], FALSE),
    ('stationary-bike', 'Stationary Bike', 'cardio', NULL, 'stationary bike', ARRAY['exercise bike'], FALSE),
    ('rowing-machine', 'Rowing Machine', 'cardio', NULL, 'rower', ARRAY['rower', 'erg'], FALSE),
    ('elliptical', 'Elliptical', 'cardio', NULL, 'elliptical', ARRAY['cross trainer'], FALSE),
    ('stair-climber', 'Stair Climber', 'cardio', NULL, 'stair machine', ARRAY['stairs', 'stepmill'], FALSE),
    ('swimming', 'Swimming', 'cardio', NULL, 'pool', ARRAY['swim'], TRUE),
    ('jump-rope', 'Jump Rope', 'cardio', NULL, 'jump rope', ARRAY['skipping rope'], FALSE),
    ('hiking', 'Hiking', 'cardio', NULL, NULL, ARRAY['hike'], TRUE)
ON CONFLICT (slug) DO UPDATE
SET
    name = EXCLUDED.name,
    modality = EXCLUDED.modality,
    primary_muscle_group = EXCLUDED.primary_muscle_group,
    equipment = EXCLUDED.equipment,
    aliases = EXCLUDED.aliases,
    is_bodyweight = EXCLUDED.is_bodyweight;
