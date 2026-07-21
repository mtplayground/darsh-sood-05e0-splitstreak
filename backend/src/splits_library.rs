use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::split_templates::{self, SplitDepthLevel, SplitTemplate, SplitTemplateFilter};
use crate::AppState;

const DEFAULT_TEMPLATE_LIMIT: i64 = 50;

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<SplitsLibraryQuery>,
) -> Result<Json<SplitsLibraryResponse>, (StatusCode, Json<SplitsLibraryError>)> {
    let filter = query.to_filter().map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(SplitsLibraryError {
                error: error.to_string(),
            }),
        )
    })?;

    let templates = split_templates::list_templates(&state.db, filter)
        .await
        .map_err(|error| {
            tracing::error!(%error, "split template library query failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SplitsLibraryError {
                    error: "split templates unavailable".to_owned(),
                }),
            )
        })?;

    Ok(Json(SplitsLibraryResponse {
        count: templates.len(),
        templates: templates.into_iter().map(SplitTemplateItem::from).collect(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct SplitsLibraryQuery {
    #[serde(default)]
    depth: Option<String>,
    #[serde(default)]
    depth_level: Option<String>,
    #[serde(default)]
    limit: Option<i64>,
}

impl SplitsLibraryQuery {
    fn to_filter(&self) -> Result<SplitTemplateFilter, SplitsLibraryQueryError> {
        let depth_level = self
            .depth
            .as_deref()
            .or(self.depth_level.as_deref())
            .map(str::parse::<SplitDepthLevel>)
            .transpose()
            .map_err(|_| SplitsLibraryQueryError::InvalidDepth)?;

        SplitTemplateFilter::new(depth_level, self.limit.unwrap_or(DEFAULT_TEMPLATE_LIMIT))
            .map_err(SplitsLibraryQueryError::from)
    }
}

#[derive(Debug, Serialize)]
pub struct SplitsLibraryResponse {
    pub count: usize,
    pub templates: Vec<SplitTemplateItem>,
}

#[derive(Debug, Serialize)]
pub struct SplitTemplateItem {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub depth_level: String,
    pub schedule: Vec<String>,
    pub training_days_per_cycle: i32,
    pub rest_days_per_cycle: i32,
    pub rationale: String,
}

impl From<SplitTemplate> for SplitTemplateItem {
    fn from(template: SplitTemplate) -> Self {
        Self {
            id: template.id,
            slug: template.slug,
            name: template.name,
            depth_level: template.depth_level,
            schedule: template.schedule,
            training_days_per_cycle: template.training_days_per_cycle,
            rest_days_per_cycle: template.rest_days_per_cycle,
            rationale: template.rationale,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SplitsLibraryError {
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SplitsLibraryQueryError {
    InvalidDepth,
    InvalidLimit,
}

impl std::fmt::Display for SplitsLibraryQueryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDepth => write!(formatter, "depth must be simple or advanced"),
            Self::InvalidLimit => write!(formatter, "limit must be 1 through 100"),
        }
    }
}

impl std::error::Error for SplitsLibraryQueryError {}

impl From<split_templates::SplitTemplateModelError> for SplitsLibraryQueryError {
    fn from(error: split_templates::SplitTemplateModelError) -> Self {
        match error {
            split_templates::SplitTemplateModelError::InvalidDepthLevel => Self::InvalidDepth,
            split_templates::SplitTemplateModelError::InvalidLimit => Self::InvalidLimit,
            split_templates::SplitTemplateModelError::InvalidScheduleDay
            | split_templates::SplitTemplateModelError::InvalidScheduleLength
            | split_templates::SplitTemplateModelError::InvalidTrainingDayCount => {
                Self::InvalidDepth
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_filter_from_depth_alias() {
        let query = SplitsLibraryQuery {
            depth: Some(" simple ".to_owned()),
            depth_level: None,
            limit: Some(12),
        };

        assert_eq!(
            query.to_filter(),
            Ok(SplitTemplateFilter {
                depth_level: Some(SplitDepthLevel::Simple),
                limit: 12,
            })
        );
    }

    #[test]
    fn builds_filter_from_depth_level_alias() {
        let query = SplitsLibraryQuery {
            depth: None,
            depth_level: Some("ADVANCED".to_owned()),
            limit: None,
        };

        assert_eq!(
            query.to_filter(),
            Ok(SplitTemplateFilter {
                depth_level: Some(SplitDepthLevel::Advanced),
                limit: DEFAULT_TEMPLATE_LIMIT,
            })
        );
    }

    #[test]
    fn rejects_invalid_depth() {
        let query = SplitsLibraryQuery {
            depth: Some("expert".to_owned()),
            depth_level: None,
            limit: None,
        };

        assert_eq!(query.to_filter(), Err(SplitsLibraryQueryError::InvalidDepth));
    }

    #[test]
    fn rejects_invalid_limit() {
        let query = SplitsLibraryQuery {
            depth: None,
            depth_level: None,
            limit: Some(101),
        };

        assert_eq!(query.to_filter(), Err(SplitsLibraryQueryError::InvalidLimit));
    }
}
