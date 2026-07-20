use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Exercise {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub modality: String,
    pub primary_muscle_group: Option<String>,
    pub equipment: Option<String>,
    pub aliases: Vec<String>,
    pub is_bodyweight: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExerciseModality {
    Strength,
    Cardio,
}

impl ExerciseModality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Strength => "strength",
            Self::Cardio => "cardio",
        }
    }
}

impl std::str::FromStr for ExerciseModality {
    type Err = ExerciseModelError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "strength" => Ok(Self::Strength),
            "cardio" => Ok(Self::Cardio),
            _ => Err(ExerciseModelError::InvalidModality),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExerciseCatalogFilter {
    pub modality: Option<ExerciseModality>,
    pub limit: i64,
}

impl Default for ExerciseCatalogFilter {
    fn default() -> Self {
        Self {
            modality: None,
            limit: 50,
        }
    }
}

impl ExerciseCatalogFilter {
    pub fn new(modality: Option<ExerciseModality>, limit: i64) -> Result<Self, ExerciseModelError> {
        if !(1..=200).contains(&limit) {
            return Err(ExerciseModelError::InvalidLimit);
        }

        Ok(Self { modality, limit })
    }
}

pub async fn list_catalog(
    pool: &PgPool,
    filter: ExerciseCatalogFilter,
) -> Result<Vec<Exercise>, sqlx::Error> {
    sqlx::query_as::<_, Exercise>(
        r#"
        SELECT
            id,
            slug,
            name,
            modality,
            primary_muscle_group,
            equipment,
            aliases,
            is_bodyweight,
            created_at,
            updated_at
        FROM exercises
        WHERE ($1::TEXT IS NULL OR modality = $1)
        ORDER BY name ASC
        LIMIT $2
        "#,
    )
    .bind(filter.modality.map(ExerciseModality::as_str))
    .bind(filter.limit)
    .persistent(false)
    .fetch_all(pool)
    .await
}

pub async fn find_by_slug(pool: &PgPool, slug: &str) -> Result<Option<Exercise>, sqlx::Error> {
    sqlx::query_as::<_, Exercise>(
        r#"
        SELECT
            id,
            slug,
            name,
            modality,
            primary_muscle_group,
            equipment,
            aliases,
            is_bodyweight,
            created_at,
            updated_at
        FROM exercises
        WHERE slug = $1
        "#,
    )
    .bind(slug)
    .persistent(false)
    .fetch_optional(pool)
    .await
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExerciseModelError {
    InvalidLimit,
    InvalidModality,
}

impl std::fmt::Display for ExerciseModelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLimit => write!(formatter, "exercise catalog limit must be 1 through 200"),
            Self::InvalidModality => write!(formatter, "exercise modality must be strength or cardio"),
        }
    }
}

impl std::error::Error for ExerciseModelError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_exercise_modality_case_insensitively() {
        assert_eq!("Strength".parse(), Ok(ExerciseModality::Strength));
        assert_eq!(" cardio ".parse(), Ok(ExerciseModality::Cardio));
    }

    #[test]
    fn rejects_unknown_exercise_modality() {
        assert_eq!(
            "mobility".parse::<ExerciseModality>(),
            Err(ExerciseModelError::InvalidModality)
        );
    }

    #[test]
    fn validates_catalog_filter_limit() {
        assert!(ExerciseCatalogFilter::new(None, 1).is_ok());
        assert!(ExerciseCatalogFilter::new(None, 200).is_ok());
        assert_eq!(
            ExerciseCatalogFilter::new(None, 0),
            Err(ExerciseModelError::InvalidLimit)
        );
        assert_eq!(
            ExerciseCatalogFilter::new(None, 201),
            Err(ExerciseModelError::InvalidLimit)
        );
    }
}
