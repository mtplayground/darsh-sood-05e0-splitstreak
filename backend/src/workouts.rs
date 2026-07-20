use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct WorkoutSession {
    pub id: i64,
    pub user_sub: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct StrengthSet {
    pub id: i64,
    pub session_id: i64,
    pub exercise_id: i64,
    pub set_number: i32,
    pub reps: i32,
    pub weight_kg: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct CardioEntry {
    pub id: i64,
    pub session_id: i64,
    pub exercise_id: i64,
    pub cardio_type: String,
    pub duration_seconds: i32,
    pub distance_meters: Option<f64>,
    pub intensity_level: Option<i32>,
    pub speed_kph: Option<f64>,
    pub incline_percent: Option<f64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewWorkoutSession {
    pub user_sub: String,
    pub started_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

impl NewWorkoutSession {
    pub fn new(
        user_sub: impl Into<String>,
        started_at: Option<DateTime<Utc>>,
        notes: Option<String>,
    ) -> Result<Self, WorkoutModelError> {
        let user_sub = normalize_required("user_sub", user_sub.into())?;

        Ok(Self {
            user_sub,
            started_at,
            notes: normalize_optional(notes),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkoutSessionUpdate {
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

impl WorkoutSessionUpdate {
    pub fn new(
        started_at: Option<DateTime<Utc>>,
        completed_at: Option<DateTime<Utc>>,
        notes: Option<String>,
    ) -> Result<Self, WorkoutModelError> {
        if let (Some(started_at), Some(completed_at)) = (started_at, completed_at) {
            if completed_at < started_at {
                return Err(WorkoutModelError::InvalidCompletedAt);
            }
        }

        Ok(Self {
            started_at,
            completed_at,
            notes: normalize_optional(notes),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewStrengthSet {
    pub session_id: i64,
    pub exercise_id: i64,
    pub set_number: i32,
    pub reps: i32,
    pub weight_kg: f64,
}

impl NewStrengthSet {
    pub fn new(
        session_id: i64,
        exercise_id: i64,
        set_number: i32,
        reps: i32,
        weight_kg: f64,
    ) -> Result<Self, WorkoutModelError> {
        if session_id <= 0 {
            return Err(WorkoutModelError::InvalidSessionId);
        }

        if exercise_id <= 0 {
            return Err(WorkoutModelError::InvalidExerciseId);
        }

        if !(1..=200).contains(&set_number) {
            return Err(WorkoutModelError::InvalidSetNumber);
        }

        if !(1..=1000).contains(&reps) {
            return Err(WorkoutModelError::InvalidReps);
        }

        if !weight_kg.is_finite() || !(0.0..=2000.0).contains(&weight_kg) {
            return Err(WorkoutModelError::InvalidWeight);
        }

        Ok(Self {
            session_id,
            exercise_id,
            set_number,
            reps,
            weight_kg,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewCardioEntry {
    pub session_id: i64,
    pub exercise_id: i64,
    pub cardio_type: String,
    pub duration_seconds: i32,
    pub distance_meters: Option<f64>,
    pub intensity_level: Option<i32>,
    pub speed_kph: Option<f64>,
    pub incline_percent: Option<f64>,
    pub notes: Option<String>,
}

impl NewCardioEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_id: i64,
        exercise_id: i64,
        cardio_type: impl Into<String>,
        duration_seconds: i32,
        distance_meters: Option<f64>,
        intensity_level: Option<i32>,
        speed_kph: Option<f64>,
        incline_percent: Option<f64>,
        notes: Option<String>,
    ) -> Result<Self, WorkoutModelError> {
        if session_id <= 0 {
            return Err(WorkoutModelError::InvalidSessionId);
        }

        if exercise_id <= 0 {
            return Err(WorkoutModelError::InvalidExerciseId);
        }

        let cardio_type = normalize_required("cardio_type", cardio_type.into())?;

        if !(1..=86400).contains(&duration_seconds) {
            return Err(WorkoutModelError::InvalidDuration);
        }

        validate_optional_float(
            distance_meters,
            0.0,
            1_000_000.0,
            WorkoutModelError::InvalidDistance,
        )?;
        validate_optional_float(speed_kph, 0.0, 80.0, WorkoutModelError::InvalidSpeed)?;
        validate_optional_float(
            incline_percent,
            -20.0,
            40.0,
            WorkoutModelError::InvalidIncline,
        )?;

        if let Some(intensity_level) = intensity_level {
            if !(1..=10).contains(&intensity_level) {
                return Err(WorkoutModelError::InvalidIntensity);
            }
        }

        Ok(Self {
            session_id,
            exercise_id,
            cardio_type,
            duration_seconds,
            distance_meters,
            intensity_level,
            speed_kph,
            incline_percent,
            notes: normalize_optional(notes),
        })
    }
}

pub async fn create_session(
    pool: &PgPool,
    session: &NewWorkoutSession,
) -> Result<WorkoutSession, sqlx::Error> {
    sqlx::query_as::<_, WorkoutSession>(
        r#"
        INSERT INTO workout_sessions (user_sub, started_at, notes)
        VALUES ($1, COALESCE($2, NOW()), $3)
        RETURNING
            id,
            user_sub,
            started_at,
            completed_at,
            notes,
            created_at,
            updated_at
        "#,
    )
    .bind(&session.user_sub)
    .bind(session.started_at)
    .bind(&session.notes)
    .persistent(false)
    .fetch_one(pool)
    .await
}

pub async fn update_session_for_user(
    pool: &PgPool,
    session_id: i64,
    user_sub: &str,
    update: &WorkoutSessionUpdate,
) -> Result<Option<WorkoutSession>, sqlx::Error> {
    sqlx::query_as::<_, WorkoutSession>(
        r#"
        UPDATE workout_sessions
        SET
            started_at = COALESCE($3, started_at),
            completed_at = COALESCE($4, completed_at),
            notes = COALESCE($5, notes)
        WHERE id = $1 AND user_sub = $2
        RETURNING
            id,
            user_sub,
            started_at,
            completed_at,
            notes,
            created_at,
            updated_at
        "#,
    )
    .bind(session_id)
    .bind(user_sub)
    .bind(update.started_at)
    .bind(update.completed_at)
    .bind(&update.notes)
    .persistent(false)
    .fetch_optional(pool)
    .await
}

pub async fn session_belongs_to_user(
    pool: &PgPool,
    session_id: i64,
    user_sub: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM workout_sessions
            WHERE id = $1 AND user_sub = $2
        )
        "#,
    )
    .bind(session_id)
    .bind(user_sub)
    .persistent(false)
    .fetch_one(pool)
    .await
}

pub async fn add_strength_set(
    pool: &PgPool,
    strength_set: &NewStrengthSet,
) -> Result<StrengthSet, sqlx::Error> {
    sqlx::query_as::<_, StrengthSet>(
        r#"
        INSERT INTO strength_sets (session_id, exercise_id, set_number, reps, weight_kg)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id,
            session_id,
            exercise_id,
            set_number,
            reps,
            weight_kg,
            created_at,
            updated_at
        "#,
    )
    .bind(strength_set.session_id)
    .bind(strength_set.exercise_id)
    .bind(strength_set.set_number)
    .bind(strength_set.reps)
    .bind(strength_set.weight_kg)
    .persistent(false)
    .fetch_one(pool)
    .await
}

pub async fn list_strength_sets(
    pool: &PgPool,
    session_id: i64,
) -> Result<Vec<StrengthSet>, sqlx::Error> {
    sqlx::query_as::<_, StrengthSet>(
        r#"
        SELECT
            id,
            session_id,
            exercise_id,
            set_number,
            reps,
            weight_kg,
            created_at,
            updated_at
        FROM strength_sets
        WHERE session_id = $1
        ORDER BY set_number ASC, id ASC
        "#,
    )
    .bind(session_id)
    .persistent(false)
    .fetch_all(pool)
    .await
}

pub async fn add_cardio_entry(
    pool: &PgPool,
    cardio_entry: &NewCardioEntry,
) -> Result<CardioEntry, sqlx::Error> {
    sqlx::query_as::<_, CardioEntry>(
        r#"
        INSERT INTO cardio_entries (
            session_id,
            exercise_id,
            cardio_type,
            duration_seconds,
            distance_meters,
            intensity_level,
            speed_kph,
            incline_percent,
            notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING
            id,
            session_id,
            exercise_id,
            cardio_type,
            duration_seconds,
            distance_meters,
            intensity_level,
            speed_kph,
            incline_percent,
            notes,
            created_at,
            updated_at
        "#,
    )
    .bind(cardio_entry.session_id)
    .bind(cardio_entry.exercise_id)
    .bind(&cardio_entry.cardio_type)
    .bind(cardio_entry.duration_seconds)
    .bind(cardio_entry.distance_meters)
    .bind(cardio_entry.intensity_level)
    .bind(cardio_entry.speed_kph)
    .bind(cardio_entry.incline_percent)
    .bind(&cardio_entry.notes)
    .persistent(false)
    .fetch_one(pool)
    .await
}

pub async fn list_cardio_entries(
    pool: &PgPool,
    session_id: i64,
) -> Result<Vec<CardioEntry>, sqlx::Error> {
    sqlx::query_as::<_, CardioEntry>(
        r#"
        SELECT
            id,
            session_id,
            exercise_id,
            cardio_type,
            duration_seconds,
            distance_meters,
            intensity_level,
            speed_kph,
            incline_percent,
            notes,
            created_at,
            updated_at
        FROM cardio_entries
        WHERE session_id = $1
        ORDER BY id ASC
        "#,
    )
    .bind(session_id)
    .persistent(false)
    .fetch_all(pool)
    .await
}

fn normalize_required(
    field: &'static str,
    value: String,
) -> Result<String, WorkoutModelError> {
    let normalized = value.trim().to_owned();
    if normalized.is_empty() {
        return Err(WorkoutModelError::RequiredFieldEmpty { field });
    }

    Ok(normalized)
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let normalized = item.trim().to_owned();
        if normalized.is_empty() {
            None
        } else {
            Some(normalized)
        }
    })
}

fn validate_optional_float(
    value: Option<f64>,
    min: f64,
    max: f64,
    error: WorkoutModelError,
) -> Result<(), WorkoutModelError> {
    if let Some(value) = value {
        if !value.is_finite() || value < min || value > max {
            return Err(error);
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkoutModelError {
    InvalidCompletedAt,
    InvalidDistance,
    InvalidDuration,
    InvalidExerciseId,
    InvalidIncline,
    InvalidIntensity,
    InvalidReps,
    InvalidSessionId,
    InvalidSetNumber,
    InvalidSpeed,
    InvalidWeight,
    RequiredFieldEmpty { field: &'static str },
}

impl std::fmt::Display for WorkoutModelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCompletedAt => write!(formatter, "completed_at must be after started_at"),
            Self::InvalidDistance => write!(formatter, "distance_meters must be 0 through 1000000"),
            Self::InvalidDuration => write!(formatter, "duration_seconds must be 1 through 86400"),
            Self::InvalidExerciseId => write!(formatter, "exercise_id must be positive"),
            Self::InvalidIncline => write!(formatter, "incline_percent must be -20 through 40"),
            Self::InvalidIntensity => write!(formatter, "intensity_level must be 1 through 10"),
            Self::InvalidReps => write!(formatter, "reps must be 1 through 1000"),
            Self::InvalidSessionId => write!(formatter, "session_id must be positive"),
            Self::InvalidSetNumber => write!(formatter, "set_number must be 1 through 200"),
            Self::InvalidSpeed => write!(formatter, "speed_kph must be 0 through 80"),
            Self::InvalidWeight => write!(formatter, "weight_kg must be 0 through 2000"),
            Self::RequiredFieldEmpty { field } => write!(formatter, "{field} must not be empty"),
        }
    }
}

impl std::error::Error for WorkoutModelError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_workout_session_trims_user_and_notes() {
        let session = NewWorkoutSession::new(" user-sub ", None, Some(" Push day ".to_owned()));

        assert_eq!(
            session,
            Ok(NewWorkoutSession {
                user_sub: "user-sub".to_owned(),
                started_at: None,
                notes: Some("Push day".to_owned()),
            })
        );
    }

    #[test]
    fn new_workout_session_rejects_empty_user() {
        assert_eq!(
            NewWorkoutSession::new(" ", None, None),
            Err(WorkoutModelError::RequiredFieldEmpty { field: "user_sub" })
        );
    }

    #[test]
    fn workout_session_update_rejects_completion_before_start() {
        let started_at = match DateTime::parse_from_rfc3339("2026-07-20T12:00:00Z") {
            Ok(value) => value.with_timezone(&Utc),
            Err(error) => panic!("test timestamp should parse: {error}"),
        };
        let completed_at = match DateTime::parse_from_rfc3339("2026-07-20T11:59:00Z") {
            Ok(value) => value.with_timezone(&Utc),
            Err(error) => panic!("test timestamp should parse: {error}"),
        };

        assert_eq!(
            WorkoutSessionUpdate::new(Some(started_at), Some(completed_at), None),
            Err(WorkoutModelError::InvalidCompletedAt)
        );
    }

    #[test]
    fn new_strength_set_validates_ranges() {
        assert!(NewStrengthSet::new(1, 2, 1, 5, 100.0).is_ok());
        assert_eq!(
            NewStrengthSet::new(0, 2, 1, 5, 100.0),
            Err(WorkoutModelError::InvalidSessionId)
        );
        assert_eq!(
            NewStrengthSet::new(1, 0, 1, 5, 100.0),
            Err(WorkoutModelError::InvalidExerciseId)
        );
        assert_eq!(
            NewStrengthSet::new(1, 2, 0, 5, 100.0),
            Err(WorkoutModelError::InvalidSetNumber)
        );
        assert_eq!(
            NewStrengthSet::new(1, 2, 1, 0, 100.0),
            Err(WorkoutModelError::InvalidReps)
        );
        assert_eq!(
            NewStrengthSet::new(1, 2, 1, 5, f64::NAN),
            Err(WorkoutModelError::InvalidWeight)
        );
    }

    #[test]
    fn new_cardio_entry_trims_type_and_notes() {
        let entry = NewCardioEntry::new(
            1,
            2,
            " treadmill ",
            1800,
            Some(5000.0),
            Some(7),
            Some(10.5),
            Some(2.5),
            Some(" tempo run ".to_owned()),
        );

        assert_eq!(
            entry,
            Ok(NewCardioEntry {
                session_id: 1,
                exercise_id: 2,
                cardio_type: "treadmill".to_owned(),
                duration_seconds: 1800,
                distance_meters: Some(5000.0),
                intensity_level: Some(7),
                speed_kph: Some(10.5),
                incline_percent: Some(2.5),
                notes: Some("tempo run".to_owned()),
            })
        );
    }

    #[test]
    fn new_cardio_entry_validates_ranges() {
        assert_eq!(
            NewCardioEntry::new(0, 2, "run", 1800, None, None, None, None, None),
            Err(WorkoutModelError::InvalidSessionId)
        );
        assert_eq!(
            NewCardioEntry::new(1, 0, "run", 1800, None, None, None, None, None),
            Err(WorkoutModelError::InvalidExerciseId)
        );
        assert_eq!(
            NewCardioEntry::new(1, 2, " ", 1800, None, None, None, None, None),
            Err(WorkoutModelError::RequiredFieldEmpty {
                field: "cardio_type"
            })
        );
        assert_eq!(
            NewCardioEntry::new(1, 2, "run", 0, None, None, None, None, None),
            Err(WorkoutModelError::InvalidDuration)
        );
        assert_eq!(
            NewCardioEntry::new(1, 2, "run", 1800, Some(-1.0), None, None, None, None),
            Err(WorkoutModelError::InvalidDistance)
        );
        assert_eq!(
            NewCardioEntry::new(1, 2, "run", 1800, None, Some(11), None, None, None),
            Err(WorkoutModelError::InvalidIntensity)
        );
        assert_eq!(
            NewCardioEntry::new(1, 2, "run", 1800, None, None, Some(f64::NAN), None, None),
            Err(WorkoutModelError::InvalidSpeed)
        );
        assert_eq!(
            NewCardioEntry::new(1, 2, "run", 1800, None, None, None, Some(45.0), None),
            Err(WorkoutModelError::InvalidIncline)
        );
    }
}
