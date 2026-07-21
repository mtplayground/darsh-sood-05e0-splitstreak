use std::collections::HashMap;

use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};

use crate::auth_middleware::CurrentUser;
use crate::workouts;
use crate::AppState;

pub async fn reconcile(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<SyncBatchRequest>,
) -> Result<Json<SyncBatchResponse>, (StatusCode, Json<SyncErrorResponse>)> {
    let service = SyncService::new(&state.db, &current_user.user.sub);
    let response = service
        .reconcile(payload)
        .await
        .map_err(SyncApiError::into_response)?;

    Ok(Json(response))
}

struct SyncService<'a> {
    db: &'a PgPool,
    user_sub: &'a str,
}

impl<'a> SyncService<'a> {
    fn new(db: &'a PgPool, user_sub: &'a str) -> Self {
        Self { db, user_sub }
    }

    async fn reconcile(
        &self,
        payload: SyncBatchRequest,
    ) -> Result<SyncBatchResponse, SyncApiError> {
        if payload.sessions.len() > 50
            || payload.strength_sets.len() > 250
            || payload.cardio_entries.len() > 250
        {
            return Err(SyncApiError::Validation("sync batch is too large"));
        }

        let mut transaction = self.db.begin().await.map_err(SyncApiError::database)?;
        let mut session_ids = HashMap::new();
        let mut session_results = Vec::with_capacity(payload.sessions.len());

        for session in payload.sessions {
            validate_client_id(&session.client_id)?;
            let new_session =
                workouts::NewWorkoutSession::new(self.user_sub, session.started_at, session.notes)
                    .map_err(SyncApiError::ValidationModel)?;
            let update = workouts::WorkoutSessionUpdate::new(
                Some(new_session.started_at.unwrap_or_else(Utc::now)),
                session.completed_at,
                new_session.notes.clone(),
            )
            .map_err(SyncApiError::ValidationModel)?;
            let synced = upsert_session(
                &mut transaction,
                self.user_sub,
                &session.client_id,
                &update,
            )
            .await
            .map_err(SyncApiError::database)?;

            session_ids.insert(session.client_id.clone(), synced.id);
            session_results.push(SyncSessionResult {
                client_id: session.client_id,
                server_id: synced.id,
                session: synced,
                status: "synced",
            });
        }

        let mut strength_results = Vec::with_capacity(payload.strength_sets.len());
        for strength_set in payload.strength_sets {
            validate_client_id(&strength_set.client_id)?;
            let session_id = resolve_session_id(
                &mut transaction,
                self.user_sub,
                &mut session_ids,
                &strength_set.client_session_id,
            )
            .await?;
            let new_strength_set = workouts::NewStrengthSet::new(
                session_id,
                strength_set.exercise_id,
                strength_set.set_number,
                strength_set.reps,
                strength_set.weight_kg,
            )
            .map_err(SyncApiError::ValidationModel)?;
            let synced = upsert_strength_set(
                &mut transaction,
                &strength_set.client_id,
                &new_strength_set,
            )
            .await
            .map_err(SyncApiError::database)?;

            strength_results.push(SyncStrengthSetResult {
                client_id: strength_set.client_id,
                server_id: synced.id,
                strength_set: synced,
                status: "synced",
            });
        }

        let mut cardio_results = Vec::with_capacity(payload.cardio_entries.len());
        for cardio_entry in payload.cardio_entries {
            validate_client_id(&cardio_entry.client_id)?;
            let session_id = resolve_session_id(
                &mut transaction,
                self.user_sub,
                &mut session_ids,
                &cardio_entry.client_session_id,
            )
            .await?;
            let new_cardio_entry = workouts::NewCardioEntry::new(
                session_id,
                cardio_entry.exercise_id,
                cardio_entry.cardio_type,
                cardio_entry.duration_seconds,
                cardio_entry.distance_meters,
                cardio_entry.intensity_level,
                cardio_entry.speed_kph,
                cardio_entry.incline_percent,
                cardio_entry.notes,
            )
            .map_err(SyncApiError::ValidationModel)?;
            let synced =
                upsert_cardio_entry(&mut transaction, &cardio_entry.client_id, &new_cardio_entry)
                    .await
                    .map_err(SyncApiError::database)?;

            cardio_results.push(SyncCardioEntryResult {
                client_id: cardio_entry.client_id,
                server_id: synced.id,
                cardio_entry: synced,
                status: "synced",
            });
        }

        transaction.commit().await.map_err(SyncApiError::database)?;

        Ok(SyncBatchResponse {
            status: "synced",
            sessions: session_results,
            strength_sets: strength_results,
            cardio_entries: cardio_results,
        })
    }
}

async fn resolve_session_id(
    transaction: &mut Transaction<'_, Postgres>,
    user_sub: &str,
    session_ids: &mut HashMap<String, i64>,
    client_session_id: &str,
) -> Result<i64, SyncApiError> {
    validate_client_id(client_session_id)?;
    if let Some(session_id) = session_ids.get(client_session_id) {
        return Ok(*session_id);
    }

    let session_id = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM workout_sessions
        WHERE user_sub = $1 AND client_id = $2
        "#,
    )
    .bind(user_sub)
    .bind(client_session_id)
    .persistent(false)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(SyncApiError::database)?
    .ok_or(SyncApiError::Validation("client session was not found"))?;

    session_ids.insert(client_session_id.to_owned(), session_id);
    Ok(session_id)
}

async fn upsert_session(
    transaction: &mut Transaction<'_, Postgres>,
    user_sub: &str,
    client_id: &str,
    update: &workouts::WorkoutSessionUpdate,
) -> Result<workouts::WorkoutSession, sqlx::Error> {
    sqlx::query_as::<_, workouts::WorkoutSession>(
        r#"
        INSERT INTO workout_sessions (user_sub, client_id, started_at, completed_at, notes)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (user_sub, client_id) WHERE client_id IS NOT NULL
        DO UPDATE SET
            started_at = EXCLUDED.started_at,
            completed_at = COALESCE(EXCLUDED.completed_at, workout_sessions.completed_at),
            notes = COALESCE(EXCLUDED.notes, workout_sessions.notes)
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
    .bind(user_sub)
    .bind(client_id)
    .bind(update.started_at)
    .bind(update.completed_at)
    .bind(&update.notes)
    .persistent(false)
    .fetch_one(&mut **transaction)
    .await
}

async fn upsert_strength_set(
    transaction: &mut Transaction<'_, Postgres>,
    client_id: &str,
    strength_set: &workouts::NewStrengthSet,
) -> Result<workouts::StrengthSet, sqlx::Error> {
    sqlx::query_as::<_, workouts::StrengthSet>(
        r#"
        INSERT INTO strength_sets (
            session_id,
            exercise_id,
            set_number,
            reps,
            weight_kg,
            client_id
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (session_id, client_id) WHERE client_id IS NOT NULL
        DO UPDATE SET
            exercise_id = EXCLUDED.exercise_id,
            set_number = EXCLUDED.set_number,
            reps = EXCLUDED.reps,
            weight_kg = EXCLUDED.weight_kg
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
    .bind(client_id)
    .persistent(false)
    .fetch_one(&mut **transaction)
    .await
}

async fn upsert_cardio_entry(
    transaction: &mut Transaction<'_, Postgres>,
    client_id: &str,
    cardio_entry: &workouts::NewCardioEntry,
) -> Result<workouts::CardioEntry, sqlx::Error> {
    sqlx::query_as::<_, workouts::CardioEntry>(
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
            notes,
            client_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (session_id, client_id) WHERE client_id IS NOT NULL
        DO UPDATE SET
            exercise_id = EXCLUDED.exercise_id,
            cardio_type = EXCLUDED.cardio_type,
            duration_seconds = EXCLUDED.duration_seconds,
            distance_meters = EXCLUDED.distance_meters,
            intensity_level = EXCLUDED.intensity_level,
            speed_kph = EXCLUDED.speed_kph,
            incline_percent = EXCLUDED.incline_percent,
            notes = COALESCE(EXCLUDED.notes, cardio_entries.notes)
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
    .bind(client_id)
    .persistent(false)
    .fetch_one(&mut **transaction)
    .await
}

fn validate_client_id(client_id: &str) -> Result<(), SyncApiError> {
    if client_id.trim().is_empty() || client_id.len() > 120 {
        return Err(SyncApiError::Validation("client_id is invalid"));
    }

    Ok(())
}

#[derive(Debug)]
enum SyncApiError {
    Database(sqlx::Error),
    Validation(&'static str),
    ValidationModel(workouts::WorkoutModelError),
}

impl SyncApiError {
    fn database(error: sqlx::Error) -> Self {
        Self::Database(error)
    }

    fn into_response(self) -> (StatusCode, Json<SyncErrorResponse>) {
        match self {
            Self::Database(error) => {
                tracing::error!(%error, "sync reconciliation database operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(SyncErrorResponse {
                        error: "sync failed",
                    }),
                )
            }
            Self::Validation(message) => (
                StatusCode::BAD_REQUEST,
                Json(SyncErrorResponse { error: message }),
            ),
            Self::ValidationModel(error) => {
                tracing::debug!(%error, "sync reconciliation validation failed");
                (
                    StatusCode::BAD_REQUEST,
                    Json(SyncErrorResponse {
                        error: "invalid sync payload",
                    }),
                )
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SyncBatchRequest {
    #[serde(default)]
    pub sessions: Vec<SyncSessionRequest>,
    #[serde(default)]
    pub strength_sets: Vec<SyncStrengthSetRequest>,
    #[serde(default)]
    pub cardio_entries: Vec<SyncCardioEntryRequest>,
}

#[derive(Debug, Deserialize)]
pub struct SyncSessionRequest {
    pub client_id: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SyncStrengthSetRequest {
    pub client_id: String,
    pub client_session_id: String,
    pub exercise_id: i64,
    pub set_number: i32,
    pub reps: i32,
    pub weight_kg: f64,
}

#[derive(Debug, Deserialize)]
pub struct SyncCardioEntryRequest {
    pub client_id: String,
    pub client_session_id: String,
    pub exercise_id: i64,
    pub cardio_type: String,
    pub duration_seconds: i32,
    pub distance_meters: Option<f64>,
    pub intensity_level: Option<i32>,
    pub speed_kph: Option<f64>,
    pub incline_percent: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SyncBatchResponse {
    pub status: &'static str,
    pub sessions: Vec<SyncSessionResult>,
    pub strength_sets: Vec<SyncStrengthSetResult>,
    pub cardio_entries: Vec<SyncCardioEntryResult>,
}

#[derive(Debug, Serialize)]
pub struct SyncSessionResult {
    pub client_id: String,
    pub server_id: i64,
    pub session: workouts::WorkoutSession,
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct SyncStrengthSetResult {
    pub client_id: String,
    pub server_id: i64,
    pub strength_set: workouts::StrengthSet,
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct SyncCardioEntryResult {
    pub client_id: String,
    pub server_id: i64,
    pub cardio_entry: workouts::CardioEntry,
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct SyncErrorResponse {
    pub error: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_blank_client_id() {
        assert!(matches!(
            validate_client_id(" "),
            Err(SyncApiError::Validation("client_id is invalid"))
        ));
    }
}
