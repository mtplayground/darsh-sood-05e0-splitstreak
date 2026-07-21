use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth_middleware::CurrentUser;
use crate::workouts;
use crate::AppState;

pub async fn create_session(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<SessionResponse>), (StatusCode, Json<LoggingError>)> {
    let service = LoggingService::new(&state.db, &current_user.user.sub);
    let session = service
        .create_session(payload)
        .await
        .map_err(LoggingApiError::into_response)?;

    Ok((StatusCode::CREATED, Json(SessionResponse { session })))
}

pub async fn update_session(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(session_id): Path<i64>,
    Json(payload): Json<UpdateSessionRequest>,
) -> Result<Json<SessionResponse>, (StatusCode, Json<LoggingError>)> {
    let service = LoggingService::new(&state.db, &current_user.user.sub);
    let session = service
        .update_session(session_id, payload)
        .await
        .map_err(LoggingApiError::into_response)?;

    Ok(Json(SessionResponse { session }))
}

pub async fn add_strength_set(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(session_id): Path<i64>,
    Json(payload): Json<AddStrengthSetRequest>,
) -> Result<(StatusCode, Json<StrengthSetResponse>), (StatusCode, Json<LoggingError>)> {
    let service = LoggingService::new(&state.db, &current_user.user.sub);
    let strength_set = service
        .add_strength_set(session_id, payload)
        .await
        .map_err(LoggingApiError::into_response)?;

    Ok((StatusCode::CREATED, Json(StrengthSetResponse { strength_set })))
}

pub async fn add_cardio_entry(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(session_id): Path<i64>,
    Json(payload): Json<AddCardioEntryRequest>,
) -> Result<(StatusCode, Json<CardioEntryResponse>), (StatusCode, Json<LoggingError>)> {
    let service = LoggingService::new(&state.db, &current_user.user.sub);
    let cardio_entry = service
        .add_cardio_entry(session_id, payload)
        .await
        .map_err(LoggingApiError::into_response)?;

    Ok((StatusCode::CREATED, Json(CardioEntryResponse { cardio_entry })))
}

struct LoggingService<'a> {
    db: &'a PgPool,
    user_sub: &'a str,
}

impl<'a> LoggingService<'a> {
    fn new(db: &'a PgPool, user_sub: &'a str) -> Self {
        Self { db, user_sub }
    }

    async fn create_session(
        &self,
        payload: CreateSessionRequest,
    ) -> Result<workouts::WorkoutSession, LoggingApiError> {
        let session =
            workouts::NewWorkoutSession::new(self.user_sub, payload.started_at, payload.notes)?;

        workouts::create_session(self.db, &session)
            .await
            .map_err(LoggingApiError::database)
    }

    async fn update_session(
        &self,
        session_id: i64,
        payload: UpdateSessionRequest,
    ) -> Result<workouts::WorkoutSession, LoggingApiError> {
        validate_session_id(session_id)?;
        let update = workouts::WorkoutSessionUpdate::new(
            payload.started_at,
            payload.completed_at,
            payload.notes,
        )?;

        workouts::update_session_for_user(self.db, session_id, self.user_sub, &update)
            .await
            .map_err(LoggingApiError::database)?
            .ok_or(LoggingApiError::NotFound)
    }

    async fn add_strength_set(
        &self,
        session_id: i64,
        payload: AddStrengthSetRequest,
    ) -> Result<workouts::StrengthSet, LoggingApiError> {
        self.ensure_session_owner(session_id).await?;
        let strength_set = workouts::NewStrengthSet::new(
            session_id,
            payload.exercise_id,
            payload.set_number,
            payload.reps,
            payload.weight_kg,
        )?;

        workouts::add_strength_set(self.db, &strength_set)
            .await
            .map_err(LoggingApiError::database)
    }

    async fn add_cardio_entry(
        &self,
        session_id: i64,
        payload: AddCardioEntryRequest,
    ) -> Result<workouts::CardioEntry, LoggingApiError> {
        self.ensure_session_owner(session_id).await?;
        let cardio_entry = workouts::NewCardioEntry::new(
            session_id,
            payload.exercise_id,
            payload.cardio_type,
            payload.duration_seconds,
            payload.distance_meters,
            payload.intensity_level,
            payload.speed_kph,
            payload.incline_percent,
            payload.notes,
        )?;

        workouts::add_cardio_entry(self.db, &cardio_entry)
            .await
            .map_err(LoggingApiError::database)
    }

    async fn ensure_session_owner(&self, session_id: i64) -> Result<(), LoggingApiError> {
        validate_session_id(session_id)?;
        let owns_session = workouts::session_belongs_to_user(self.db, session_id, self.user_sub)
            .await
            .map_err(LoggingApiError::database)?;

        if owns_session {
            Ok(())
        } else {
            Err(LoggingApiError::NotFound)
        }
    }
}

fn validate_session_id(session_id: i64) -> Result<(), LoggingApiError> {
    if session_id <= 0 {
        return Err(LoggingApiError::Validation(
            workouts::WorkoutModelError::InvalidSessionId,
        ));
    }

    Ok(())
}

#[derive(Debug)]
enum LoggingApiError {
    Database(sqlx::Error),
    NotFound,
    Validation(workouts::WorkoutModelError),
}

impl LoggingApiError {
    fn database(error: sqlx::Error) -> Self {
        Self::Database(error)
    }

    fn into_response(self) -> (StatusCode, Json<LoggingError>) {
        match self {
            Self::Database(error) => {
                tracing::error!(%error, "logging database operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(LoggingError {
                        error: "logging_operation_failed",
                        message: "Workout changes could not be saved right now. Try again.",
                    }),
                )
            }
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                Json(LoggingError {
                    error: "workout_session_not_found",
                    message: "Workout session was not found. Refresh and try again.",
                }),
            ),
            Self::Validation(error) => {
                tracing::debug!(%error, "logging request validation failed");
                (
                    StatusCode::BAD_REQUEST,
                    Json(LoggingError {
                        error: "invalid_logging_request",
                        message: "Check the workout details and try again.",
                    }),
                )
            }
        }
    }
}

impl From<workouts::WorkoutModelError> for LoggingApiError {
    fn from(error: workouts::WorkoutModelError) -> Self {
        Self::Validation(error)
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub started_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSessionRequest {
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddStrengthSetRequest {
    pub exercise_id: i64,
    pub set_number: i32,
    pub reps: i32,
    pub weight_kg: f64,
}

#[derive(Debug, Deserialize)]
pub struct AddCardioEntryRequest {
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
pub struct SessionResponse {
    pub session: workouts::WorkoutSession,
}

#[derive(Debug, Serialize)]
pub struct StrengthSetResponse {
    pub strength_set: workouts::StrengthSet,
}

#[derive(Debug, Serialize)]
pub struct CardioEntryResponse {
    pub cardio_entry: workouts::CardioEntry,
}

#[derive(Debug, Serialize)]
pub struct LoggingError {
    pub error: &'static str,
    pub message: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_session_path_id() {
        assert!(matches!(
            validate_session_id(0),
            Err(LoggingApiError::Validation(
                workouts::WorkoutModelError::InvalidSessionId
            ))
        ));
    }

    #[test]
    fn maps_not_found_to_404() {
        let (status, Json(error)) = LoggingApiError::NotFound.into_response();

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(error.error, "workout_session_not_found");
        assert_eq!(
            error.message,
            "Workout session was not found. Refresh and try again."
        );
    }
}
