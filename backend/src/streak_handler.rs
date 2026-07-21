use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use crate::auth_middleware::CurrentUser;
use crate::streaks;
use crate::AppState;

const DEFAULT_CALENDAR_DAYS: i64 = 35;

pub async fn current(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<StreakQuery>,
) -> Result<Json<streaks::StreakCalendar>, (StatusCode, Json<StreakError>)> {
    let days = query.calendar_days().map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(StreakError {
                error: error.to_string(),
            }),
        )
    })?;

    let calendar = streaks::compute_streak_calendar(&state.db, &current_user.user.sub, days)
        .await
        .map_err(|error| {
            tracing::error!(%error, "streak endpoint database operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(StreakError {
                    error: "streak could not be loaded".to_owned(),
                }),
            )
        })?;

    Ok(Json(calendar))
}

#[derive(Debug, Deserialize)]
pub struct StreakQuery {
    #[serde(default)]
    days: Option<i64>,
}

impl StreakQuery {
    fn calendar_days(&self) -> Result<i64, StreakQueryError> {
        let days = self.days.unwrap_or(DEFAULT_CALENDAR_DAYS);
        if !(7..=90).contains(&days) {
            return Err(StreakQueryError::InvalidDays);
        }

        Ok(days)
    }
}

#[derive(Debug, Serialize)]
pub struct StreakError {
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StreakQueryError {
    InvalidDays,
}

impl std::fmt::Display for StreakQueryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDays => write!(formatter, "days must be 7 through 90"),
        }
    }
}

impl std::error::Error for StreakQueryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_default_and_valid_calendar_days() {
        assert_eq!(StreakQuery { days: None }.calendar_days(), Ok(35));
        assert_eq!(StreakQuery { days: Some(14) }.calendar_days(), Ok(14));
    }

    #[test]
    fn rejects_calendar_days_outside_bounds() {
        assert_eq!(
            StreakQuery { days: Some(6) }.calendar_days(),
            Err(StreakQueryError::InvalidDays)
        );
        assert_eq!(
            StreakQuery { days: Some(91) }.calendar_days(),
            Err(StreakQueryError::InvalidDays)
        );
    }
}
