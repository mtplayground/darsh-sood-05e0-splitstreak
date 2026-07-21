use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;

use crate::auth_middleware::CurrentUser;
use crate::AppState;

const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 50;
const MAX_OFFSET: i64 = 10_000;

pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>, (StatusCode, Json<HistoryError>)> {
    let page = HistoryPageRequest::try_from(query).map_err(HistoryApiError::into_response)?;
    let history = load_history(&state.db, &current_user.user.sub, page)
        .await
        .map_err(HistoryApiError::database)
        .map_err(HistoryApiError::into_response)?;

    Ok(Json(history))
}

async fn load_history(
    pool: &PgPool,
    user_sub: &str,
    page: HistoryPageRequest,
) -> Result<HistoryResponse, sqlx::Error> {
    let requested_limit = page.limit + 1;
    let mut session_rows = sqlx::query_as::<_, SessionHistoryRow>(
        r#"
        SELECT
            id,
            user_sub,
            started_at,
            completed_at,
            notes,
            created_at,
            updated_at
        FROM workout_sessions
        WHERE user_sub = $1
        ORDER BY started_at DESC, id DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_sub)
    .bind(requested_limit)
    .bind(page.offset)
    .persistent(false)
    .fetch_all(pool)
    .await?;

    let has_more = session_rows.len() as i64 > page.limit;
    if has_more {
        session_rows.truncate(page.limit as usize);
    }

    let session_ids = session_rows
        .iter()
        .map(|session| session.id)
        .collect::<Vec<_>>();
    let strength_sets = load_strength_sets(pool, &session_ids).await?;
    let cardio_entries = load_cardio_entries(pool, &session_ids).await?;
    let mut strength_by_session = group_strength_sets(strength_sets);
    let mut cardio_by_session = group_cardio_entries(cardio_entries);

    let sessions = session_rows
        .into_iter()
        .map(|row| HistorySession {
            id: row.id,
            user_sub: row.user_sub,
            date: row.started_at.date_naive(),
            started_at: row.started_at,
            completed_at: row.completed_at,
            notes: row.notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
            strength_sets: strength_by_session.remove(&row.id).unwrap_or_default(),
            cardio_entries: cardio_by_session.remove(&row.id).unwrap_or_default(),
        })
        .collect::<Vec<_>>();

    Ok(HistoryResponse {
        page: HistoryPage {
            limit: page.limit,
            offset: page.offset,
            has_more,
            next_offset: has_more.then_some(page.offset + page.limit),
        },
        sessions,
    })
}

async fn load_strength_sets(
    pool: &PgPool,
    session_ids: &[i64],
) -> Result<Vec<StrengthSetHistoryRow>, sqlx::Error> {
    if session_ids.is_empty() {
        return Ok(Vec::new());
    }

    sqlx::query_as::<_, StrengthSetHistoryRow>(
        r#"
        SELECT
            ss.id,
            ss.session_id,
            ss.exercise_id,
            e.slug AS exercise_slug,
            e.name AS exercise_name,
            ss.set_number,
            ss.reps,
            ss.weight_kg,
            ss.created_at,
            ss.updated_at
        FROM strength_sets ss
        INNER JOIN exercises e ON e.id = ss.exercise_id
        WHERE ss.session_id = ANY($1)
        ORDER BY
            array_position($1, ss.session_id),
            e.name ASC,
            ss.set_number ASC,
            ss.id ASC
        "#,
    )
    .bind(session_ids)
    .persistent(false)
    .fetch_all(pool)
    .await
}

async fn load_cardio_entries(
    pool: &PgPool,
    session_ids: &[i64],
) -> Result<Vec<CardioEntryHistoryRow>, sqlx::Error> {
    if session_ids.is_empty() {
        return Ok(Vec::new());
    }

    sqlx::query_as::<_, CardioEntryHistoryRow>(
        r#"
        SELECT
            ce.id,
            ce.session_id,
            ce.exercise_id,
            e.slug AS exercise_slug,
            e.name AS exercise_name,
            ce.cardio_type,
            ce.duration_seconds,
            ce.distance_meters,
            ce.intensity_level,
            ce.speed_kph,
            ce.incline_percent,
            ce.notes,
            ce.created_at,
            ce.updated_at
        FROM cardio_entries ce
        INNER JOIN exercises e ON e.id = ce.exercise_id
        WHERE ce.session_id = ANY($1)
        ORDER BY array_position($1, ce.session_id), ce.id ASC
        "#,
    )
    .bind(session_ids)
    .persistent(false)
    .fetch_all(pool)
    .await
}

fn group_strength_sets(
    rows: Vec<StrengthSetHistoryRow>,
) -> HashMap<i64, Vec<StrengthSetHistoryItem>> {
    let mut grouped: HashMap<i64, Vec<StrengthSetHistoryItem>> = HashMap::new();
    for row in rows {
        grouped
            .entry(row.session_id)
            .or_default()
            .push(StrengthSetHistoryItem {
                id: row.id,
                exercise_id: row.exercise_id,
                exercise_slug: row.exercise_slug,
                exercise_name: row.exercise_name,
                set_number: row.set_number,
                reps: row.reps,
                weight_kg: row.weight_kg,
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
    }

    grouped
}

fn group_cardio_entries(
    rows: Vec<CardioEntryHistoryRow>,
) -> HashMap<i64, Vec<CardioEntryHistoryItem>> {
    let mut grouped: HashMap<i64, Vec<CardioEntryHistoryItem>> = HashMap::new();
    for row in rows {
        grouped
            .entry(row.session_id)
            .or_default()
            .push(CardioEntryHistoryItem {
                id: row.id,
                exercise_id: row.exercise_id,
                exercise_slug: row.exercise_slug,
                exercise_name: row.exercise_name,
                cardio_type: row.cardio_type,
                duration_seconds: row.duration_seconds,
                distance_meters: row.distance_meters,
                intensity_level: row.intensity_level,
                speed_kph: row.speed_kph,
                incline_percent: row.incline_percent,
                notes: row.notes,
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
    }

    grouped
}

#[derive(Debug, Deserialize, Default)]
pub struct HistoryQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HistoryPageRequest {
    limit: i64,
    offset: i64,
}

impl TryFrom<HistoryQuery> for HistoryPageRequest {
    type Error = HistoryApiError;

    fn try_from(query: HistoryQuery) -> Result<Self, Self::Error> {
        let limit = query.limit.unwrap_or(DEFAULT_LIMIT);
        let offset = query.offset.unwrap_or(0);

        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(HistoryApiError::Validation);
        }

        if !(0..=MAX_OFFSET).contains(&offset) {
            return Err(HistoryApiError::Validation);
        }

        Ok(Self { limit, offset })
    }
}

#[derive(Debug, FromRow)]
struct SessionHistoryRow {
    id: i64,
    user_sub: String,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct StrengthSetHistoryRow {
    id: i64,
    session_id: i64,
    exercise_id: i64,
    exercise_slug: String,
    exercise_name: String,
    set_number: i32,
    reps: i32,
    weight_kg: f64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct CardioEntryHistoryRow {
    id: i64,
    session_id: i64,
    exercise_id: i64,
    exercise_slug: String,
    exercise_name: String,
    cardio_type: String,
    duration_seconds: i32,
    distance_meters: Option<f64>,
    intensity_level: Option<i32>,
    speed_kph: Option<f64>,
    incline_percent: Option<f64>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub page: HistoryPage,
    pub sessions: Vec<HistorySession>,
}

#[derive(Debug, Serialize)]
pub struct HistoryPage {
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
    pub next_offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct HistorySession {
    pub id: i64,
    pub user_sub: String,
    pub date: NaiveDate,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub strength_sets: Vec<StrengthSetHistoryItem>,
    pub cardio_entries: Vec<CardioEntryHistoryItem>,
}

#[derive(Debug, Serialize)]
pub struct StrengthSetHistoryItem {
    pub id: i64,
    pub exercise_id: i64,
    pub exercise_slug: String,
    pub exercise_name: String,
    pub set_number: i32,
    pub reps: i32,
    pub weight_kg: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CardioEntryHistoryItem {
    pub id: i64,
    pub exercise_id: i64,
    pub exercise_slug: String,
    pub exercise_name: String,
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

#[derive(Debug)]
enum HistoryApiError {
    Database(sqlx::Error),
    Validation,
}

impl HistoryApiError {
    fn database(error: sqlx::Error) -> Self {
        Self::Database(error)
    }

    fn into_response(self) -> (StatusCode, Json<HistoryError>) {
        match self {
            Self::Database(error) => {
                tracing::error!(%error, "history endpoint database operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(HistoryError {
                        error: "history could not be loaded",
                    }),
                )
            }
            Self::Validation => (
                StatusCode::BAD_REQUEST,
                Json(HistoryError {
                    error: "invalid history pagination",
                }),
            ),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HistoryError {
    pub error: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_default_pagination() {
        let page = HistoryPageRequest::try_from(HistoryQuery::default()).unwrap();

        assert_eq!(
            page,
            HistoryPageRequest {
                limit: DEFAULT_LIMIT,
                offset: 0
            }
        );
    }

    #[test]
    fn accepts_explicit_pagination_bounds() {
        let page = HistoryPageRequest::try_from(HistoryQuery {
            limit: Some(MAX_LIMIT),
            offset: Some(MAX_OFFSET),
        })
        .unwrap();

        assert_eq!(
            page,
            HistoryPageRequest {
                limit: MAX_LIMIT,
                offset: MAX_OFFSET
            }
        );
    }

    #[test]
    fn rejects_invalid_pagination() {
        for query in [
            HistoryQuery {
                limit: Some(0),
                offset: None,
            },
            HistoryQuery {
                limit: Some(MAX_LIMIT + 1),
                offset: None,
            },
            HistoryQuery {
                limit: None,
                offset: Some(-1),
            },
            HistoryQuery {
                limit: None,
                offset: Some(MAX_OFFSET + 1),
            },
        ] {
            assert!(matches!(
                HistoryPageRequest::try_from(query),
                Err(HistoryApiError::Validation)
            ));
        }
    }
}
