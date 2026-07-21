use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::active_splits;
use crate::auth_middleware::CurrentUser;
use crate::AppState;

pub async fn get_active_split(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<ActiveSplitResponse>, (StatusCode, Json<ActiveSplitError>)> {
    let service = ActiveSplitService::new(&state.db, &current_user.user.sub);
    let active_split = service
        .get_active_split()
        .await
        .map_err(ActiveSplitApiError::into_response)?;

    Ok(Json(ActiveSplitResponse { active_split }))
}

pub async fn select_active_split(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<SelectActiveSplitRequest>,
) -> Result<Json<ActiveSplitResponse>, (StatusCode, Json<ActiveSplitError>)> {
    let service = ActiveSplitService::new(&state.db, &current_user.user.sub);
    let active_split = service
        .select_active_split(payload)
        .await
        .map_err(ActiveSplitApiError::into_response)?;

    Ok(Json(ActiveSplitResponse {
        active_split: Some(active_split),
    }))
}

struct ActiveSplitService<'a> {
    db: &'a PgPool,
    user_sub: &'a str,
}

impl<'a> ActiveSplitService<'a> {
    fn new(db: &'a PgPool, user_sub: &'a str) -> Self {
        Self { db, user_sub }
    }

    async fn get_active_split(
        &self,
    ) -> Result<Option<active_splits::ActiveSplit>, ActiveSplitApiError> {
        active_splits::find_active_split(self.db, self.user_sub)
            .await
            .map_err(ActiveSplitApiError::database)
    }

    async fn select_active_split(
        &self,
        payload: SelectActiveSplitRequest,
    ) -> Result<active_splits::ActiveSplit, ActiveSplitApiError> {
        let selection = payload.into_selection()?;
        active_splits::select_active_split(self.db, self.user_sub, &selection)
            .await
            .map_err(ActiveSplitApiError::database)?
            .ok_or(ActiveSplitApiError::NotFound)
    }
}

#[derive(Debug)]
enum ActiveSplitApiError {
    Database(sqlx::Error),
    NotFound,
    Validation(active_splits::ActiveSplitModelError),
}

impl ActiveSplitApiError {
    fn database(error: sqlx::Error) -> Self {
        Self::Database(error)
    }

    fn into_response(self) -> (StatusCode, Json<ActiveSplitError>) {
        match self {
            Self::Database(error) => {
                tracing::error!(%error, "active split database operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ActiveSplitError {
                        error: "active split operation failed",
                    }),
                )
            }
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ActiveSplitError {
                    error: "split template not found",
                }),
            ),
            Self::Validation(error) => {
                tracing::debug!(%error, "active split request validation failed");
                (
                    StatusCode::BAD_REQUEST,
                    Json(ActiveSplitError {
                        error: "invalid active split request",
                    }),
                )
            }
        }
    }
}

impl From<active_splits::ActiveSplitModelError> for ActiveSplitApiError {
    fn from(error: active_splits::ActiveSplitModelError) -> Self {
        Self::Validation(error)
    }
}

#[derive(Debug, Deserialize)]
pub struct SelectActiveSplitRequest {
    pub split_template_id: Option<i64>,
    pub split_template_slug: Option<String>,
}

impl SelectActiveSplitRequest {
    fn into_selection(
        self,
    ) -> Result<active_splits::ActiveSplitSelection, active_splits::ActiveSplitModelError> {
        match (self.split_template_id, self.split_template_slug) {
            (Some(_), Some(_)) => Err(active_splits::ActiveSplitModelError::AmbiguousSelection),
            (Some(template_id), None) => {
                active_splits::ActiveSplitSelection::by_template_id(template_id)
            }
            (None, Some(slug)) => active_splits::ActiveSplitSelection::by_template_slug(slug),
            (None, None) => Err(active_splits::ActiveSplitModelError::MissingSelection),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ActiveSplitResponse {
    pub active_split: Option<active_splits::ActiveSplit>,
}

#[derive(Debug, Serialize)]
pub struct ActiveSplitError {
    pub error: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_accepts_slug_selection() {
        let selection = SelectActiveSplitRequest {
            split_template_id: None,
            split_template_slug: Some(" full-body-3-day ".to_owned()),
        }
        .into_selection();

        assert_eq!(
            selection,
            Ok(active_splits::ActiveSplitSelection::TemplateSlug(
                "full-body-3-day".to_owned()
            ))
        );
    }

    #[test]
    fn request_rejects_missing_or_ambiguous_selection() {
        assert_eq!(
            SelectActiveSplitRequest {
                split_template_id: None,
                split_template_slug: None,
            }
            .into_selection(),
            Err(active_splits::ActiveSplitModelError::MissingSelection)
        );
        assert_eq!(
            SelectActiveSplitRequest {
                split_template_id: Some(1),
                split_template_slug: Some("full-body-3-day".to_owned()),
            }
            .into_selection(),
            Err(active_splits::ActiveSplitModelError::AmbiguousSelection)
        );
    }
}
