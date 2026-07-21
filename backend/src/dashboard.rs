use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Serialize;

use crate::auth_middleware::CurrentUser;
use crate::streaks;
use crate::workouts;
use crate::AppState;

pub async fn today(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<TodayDashboardResponse>, (StatusCode, Json<DashboardError>)> {
    let workout = workouts::find_today_session_summary(&state.db, &current_user.user.sub)
        .await
        .map_err(|error| {
            tracing::error!(%error, "today dashboard database lookup failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DashboardError {
                    error: "dashboard could not be loaded",
                }),
            )
        })?;

    let streak = streaks::compute_current_streak(&state.db, &current_user.user.sub)
        .await
        .map_err(|error| {
            tracing::error!(%error, "today streak computation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DashboardError {
                    error: "dashboard could not be loaded",
                }),
            )
        })?;

    Ok(Json(TodayDashboardResponse { workout, streak }))
}

#[derive(Debug, Serialize)]
pub struct TodayDashboardResponse {
    pub workout: Option<workouts::WorkoutSessionSummary>,
    pub streak: streaks::StreakSummary,
}

#[derive(Debug, Serialize)]
pub struct DashboardError {
    pub error: &'static str,
}
