use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::exercises::{self, Exercise, ExerciseModality, ExerciseSearch};
use crate::AppState;

const DEFAULT_SEARCH_LIMIT: i64 = 8;

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<ExerciseSearchQuery>,
) -> Result<Json<ExerciseSearchResponse>, (StatusCode, Json<ExerciseSearchError>)> {
    let search = query.to_search().map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(ExerciseSearchError {
                error: error.to_string(),
            }),
        )
    })?;

    let exercises = exercises::search_catalog(&state.db, &search)
        .await
        .map_err(|error| {
            tracing::error!(%error, "exercise search query failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ExerciseSearchError {
                    error: "exercise search failed".to_owned(),
                }),
            )
        })?;

    Ok(Json(ExerciseSearchResponse {
        query: search.query,
        count: exercises.len(),
        exercises: exercises.into_iter().map(ExerciseSearchItem::from).collect(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct ExerciseSearchQuery {
    #[serde(default)]
    q: Option<String>,
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    modality: Option<String>,
    #[serde(default)]
    limit: Option<i64>,
}

impl ExerciseSearchQuery {
    fn to_search(&self) -> Result<ExerciseSearch, ExerciseSearchQueryError> {
        let raw_query = self
            .q
            .as_deref()
            .or(self.query.as_deref())
            .ok_or(ExerciseSearchQueryError::MissingQuery)?;
        let modality = self
            .modality
            .as_deref()
            .map(str::parse::<ExerciseModality>)
            .transpose()
            .map_err(|_| ExerciseSearchQueryError::InvalidModality)?;

        ExerciseSearch::new(raw_query, modality, self.limit.unwrap_or(DEFAULT_SEARCH_LIMIT))
            .map_err(ExerciseSearchQueryError::from)
    }
}

#[derive(Debug, Serialize)]
pub struct ExerciseSearchResponse {
    pub query: String,
    pub count: usize,
    pub exercises: Vec<ExerciseSearchItem>,
}

#[derive(Debug, Serialize)]
pub struct ExerciseSearchItem {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub modality: String,
    pub primary_muscle_group: Option<String>,
    pub equipment: Option<String>,
    pub aliases: Vec<String>,
    pub is_bodyweight: bool,
}

impl From<Exercise> for ExerciseSearchItem {
    fn from(exercise: Exercise) -> Self {
        Self {
            id: exercise.id,
            slug: exercise.slug,
            name: exercise.name,
            modality: exercise.modality,
            primary_muscle_group: exercise.primary_muscle_group,
            equipment: exercise.equipment,
            aliases: exercise.aliases,
            is_bodyweight: exercise.is_bodyweight,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExerciseSearchError {
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExerciseSearchQueryError {
    InvalidLimit,
    InvalidModality,
    InvalidSearchQuery,
    MissingQuery,
}

impl std::fmt::Display for ExerciseSearchQueryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLimit => write!(formatter, "limit must be 1 through 20"),
            Self::InvalidModality => write!(formatter, "modality must be strength or cardio"),
            Self::InvalidSearchQuery => {
                write!(formatter, "query must be 1 through 80 characters")
            }
            Self::MissingQuery => write!(formatter, "query is required"),
        }
    }
}

impl std::error::Error for ExerciseSearchQueryError {}

impl From<exercises::ExerciseModelError> for ExerciseSearchQueryError {
    fn from(error: exercises::ExerciseModelError) -> Self {
        match error {
            exercises::ExerciseModelError::InvalidLimit => Self::InvalidLimit,
            exercises::ExerciseModelError::InvalidModality => Self::InvalidModality,
            exercises::ExerciseModelError::InvalidSearchQuery => Self::InvalidSearchQuery,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_search_from_q_alias() {
        let query = ExerciseSearchQuery {
            q: Some(" bench ".to_owned()),
            query: None,
            modality: Some("strength".to_owned()),
            limit: Some(6),
        };

        let search = query.to_search();

        assert_eq!(
            search,
            Ok(ExerciseSearch {
                query: "bench".to_owned(),
                modality: Some(ExerciseModality::Strength),
                limit: 6,
            })
        );
    }

    #[test]
    fn rejects_missing_search_query() {
        let query = ExerciseSearchQuery {
            q: None,
            query: None,
            modality: None,
            limit: None,
        };

        assert_eq!(query.to_search(), Err(ExerciseSearchQueryError::MissingQuery));
    }

    #[test]
    fn caps_search_result_limit() {
        let query = ExerciseSearchQuery {
            q: Some("run".to_owned()),
            query: None,
            modality: None,
            limit: Some(30),
        };

        assert_eq!(query.to_search(), Err(ExerciseSearchQueryError::InvalidLimit));
    }
}
