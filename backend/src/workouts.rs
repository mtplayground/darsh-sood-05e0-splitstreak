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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkoutModelError {
    InvalidExerciseId,
    InvalidReps,
    InvalidSessionId,
    InvalidSetNumber,
    InvalidWeight,
    RequiredFieldEmpty { field: &'static str },
}

impl std::fmt::Display for WorkoutModelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidExerciseId => write!(formatter, "exercise_id must be positive"),
            Self::InvalidReps => write!(formatter, "reps must be 1 through 1000"),
            Self::InvalidSessionId => write!(formatter, "session_id must be positive"),
            Self::InvalidSetNumber => write!(formatter, "set_number must be 1 through 200"),
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
}
