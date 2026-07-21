use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct SplitTemplate {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub depth_level: String,
    pub schedule: Vec<String>,
    pub training_days_per_cycle: i32,
    pub rest_days_per_cycle: i32,
    pub rationale: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitDepthLevel {
    Simple,
    Advanced,
}

impl SplitDepthLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Advanced => "advanced",
        }
    }
}

impl std::str::FromStr for SplitDepthLevel {
    type Err = SplitTemplateModelError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "simple" => Ok(Self::Simple),
            "advanced" => Ok(Self::Advanced),
            _ => Err(SplitTemplateModelError::InvalidDepthLevel),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitTemplateSchedule {
    pub schedule: Vec<String>,
    pub training_days_per_cycle: i32,
    pub rest_days_per_cycle: i32,
}

impl SplitTemplateSchedule {
    pub fn new(schedule: Vec<String>) -> Result<Self, SplitTemplateModelError> {
        if !(1..=14).contains(&schedule.len()) {
            return Err(SplitTemplateModelError::InvalidScheduleLength);
        }

        let mut normalized = Vec::with_capacity(schedule.len());
        let mut rest_days = 0;
        for item in schedule {
            let day = item.trim();
            if day.is_empty() {
                return Err(SplitTemplateModelError::InvalidScheduleDay);
            }

            if day.eq_ignore_ascii_case("rest") {
                rest_days += 1;
            }
            normalized.push(day.to_owned());
        }

        let training_days = normalized.len() - rest_days;
        if training_days == 0 {
            return Err(SplitTemplateModelError::InvalidTrainingDayCount);
        }

        Ok(Self {
            schedule: normalized,
            training_days_per_cycle: training_days as i32,
            rest_days_per_cycle: rest_days as i32,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitTemplateFilter {
    pub depth_level: Option<SplitDepthLevel>,
    pub limit: i64,
}

impl Default for SplitTemplateFilter {
    fn default() -> Self {
        Self {
            depth_level: None,
            limit: 50,
        }
    }
}

impl SplitTemplateFilter {
    pub fn new(
        depth_level: Option<SplitDepthLevel>,
        limit: i64,
    ) -> Result<Self, SplitTemplateModelError> {
        if !(1..=100).contains(&limit) {
            return Err(SplitTemplateModelError::InvalidLimit);
        }

        Ok(Self { depth_level, limit })
    }
}

pub async fn list_templates(
    pool: &PgPool,
    filter: SplitTemplateFilter,
) -> Result<Vec<SplitTemplate>, sqlx::Error> {
    sqlx::query_as::<_, SplitTemplate>(
        r#"
        SELECT
            id,
            slug,
            name,
            depth_level,
            schedule,
            training_days_per_cycle,
            rest_days_per_cycle,
            rationale,
            created_at,
            updated_at
        FROM split_templates
        WHERE ($1::TEXT IS NULL OR depth_level = $1)
        ORDER BY
            CASE depth_level
                WHEN 'simple' THEN 0
                ELSE 1
            END,
            training_days_per_cycle ASC,
            name ASC
        LIMIT $2
        "#,
    )
    .bind(filter.depth_level.map(SplitDepthLevel::as_str))
    .bind(filter.limit)
    .persistent(false)
    .fetch_all(pool)
    .await
}

pub async fn find_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<SplitTemplate>, sqlx::Error> {
    sqlx::query_as::<_, SplitTemplate>(
        r#"
        SELECT
            id,
            slug,
            name,
            depth_level,
            schedule,
            training_days_per_cycle,
            rest_days_per_cycle,
            rationale,
            created_at,
            updated_at
        FROM split_templates
        WHERE slug = $1
        "#,
    )
    .bind(slug)
    .persistent(false)
    .fetch_optional(pool)
    .await
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SplitTemplateModelError {
    InvalidDepthLevel,
    InvalidLimit,
    InvalidScheduleDay,
    InvalidScheduleLength,
    InvalidTrainingDayCount,
}

impl std::fmt::Display for SplitTemplateModelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDepthLevel => {
                write!(formatter, "split template depth must be simple or advanced")
            }
            Self::InvalidLimit => write!(formatter, "split template limit must be 1 through 100"),
            Self::InvalidScheduleDay => write!(formatter, "split schedule days must not be empty"),
            Self::InvalidScheduleLength => {
                write!(formatter, "split schedule must be 1 through 14 days")
            }
            Self::InvalidTrainingDayCount => {
                write!(formatter, "split schedule must include at least one training day")
            }
        }
    }
}

impl std::error::Error for SplitTemplateModelError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_depth_level_case_insensitively() {
        assert_eq!(" Simple ".parse(), Ok(SplitDepthLevel::Simple));
        assert_eq!("ADVANCED".parse(), Ok(SplitDepthLevel::Advanced));
        assert_eq!(
            "expert".parse::<SplitDepthLevel>(),
            Err(SplitTemplateModelError::InvalidDepthLevel)
        );
    }

    #[test]
    fn validates_template_filter_limit() {
        assert_eq!(
            SplitTemplateFilter::new(Some(SplitDepthLevel::Simple), 8),
            Ok(SplitTemplateFilter {
                depth_level: Some(SplitDepthLevel::Simple),
                limit: 8,
            })
        );
        assert_eq!(
            SplitTemplateFilter::new(None, 0),
            Err(SplitTemplateModelError::InvalidLimit)
        );
    }

    #[test]
    fn schedule_counts_training_and_rest_days() {
        let schedule = SplitTemplateSchedule::new(vec![
            " Upper ".to_owned(),
            "Lower".to_owned(),
            "Rest".to_owned(),
        ]);

        assert_eq!(
            schedule,
            Ok(SplitTemplateSchedule {
                schedule: vec!["Upper".to_owned(), "Lower".to_owned(), "Rest".to_owned()],
                training_days_per_cycle: 2,
                rest_days_per_cycle: 1,
            })
        );
    }

    #[test]
    fn schedule_rejects_empty_or_all_rest_cycles() {
        assert_eq!(
            SplitTemplateSchedule::new(vec!["Rest".to_owned()]),
            Err(SplitTemplateModelError::InvalidTrainingDayCount)
        );
        assert_eq!(
            SplitTemplateSchedule::new(vec![" ".to_owned()]),
            Err(SplitTemplateModelError::InvalidScheduleDay)
        );
    }
}
