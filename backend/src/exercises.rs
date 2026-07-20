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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExerciseSearch {
    pub query: String,
    pub modality: Option<ExerciseModality>,
    pub limit: i64,
}

impl ExerciseSearch {
    pub fn new(
        query: impl Into<String>,
        modality: Option<ExerciseModality>,
        limit: i64,
    ) -> Result<Self, ExerciseModelError> {
        let query = query.into();
        let query = query.trim();
        if query.is_empty() {
            return Err(ExerciseModelError::InvalidSearchQuery);
        }

        if query.chars().count() > 80 {
            return Err(ExerciseModelError::InvalidSearchQuery);
        }

        if !(1..=20).contains(&limit) {
            return Err(ExerciseModelError::InvalidLimit);
        }

        Ok(Self {
            query: query.to_owned(),
            modality,
            limit,
        })
    }
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

pub async fn search_catalog(
    pool: &PgPool,
    search: &ExerciseSearch,
) -> Result<Vec<Exercise>, sqlx::Error> {
    let escaped_query = escape_like_pattern(&search.query.to_ascii_lowercase());
    let contains_pattern = format!("%{escaped_query}%");
    let prefix_pattern = format!("{escaped_query}%");

    sqlx::query_as::<_, Exercise>(
        r#"
        WITH input AS (
            SELECT lower($1::TEXT) AS query
        )
        SELECT
            exercises.id,
            exercises.slug,
            exercises.name,
            exercises.modality,
            exercises.primary_muscle_group,
            exercises.equipment,
            exercises.aliases,
            exercises.is_bodyweight,
            exercises.created_at,
            exercises.updated_at
        FROM exercises, input
        WHERE ($4::TEXT IS NULL OR exercises.modality = $4)
            AND (
                lower(exercises.name) LIKE $2 ESCAPE '\'
                OR EXISTS (
                    SELECT 1
                    FROM unnest(exercises.aliases) AS alias
                    WHERE lower(alias) LIKE $2 ESCAPE '\'
                )
            )
        ORDER BY
            CASE
                WHEN lower(exercises.name) = input.query THEN 0
                WHEN lower(exercises.name) LIKE $3 ESCAPE '\' THEN 1
                WHEN EXISTS (
                    SELECT 1
                    FROM unnest(exercises.aliases) AS alias
                    WHERE lower(alias) = input.query
                ) THEN 2
                WHEN EXISTS (
                    SELECT 1
                    FROM unnest(exercises.aliases) AS alias
                    WHERE lower(alias) LIKE $3 ESCAPE '\'
                ) THEN 3
                WHEN lower(exercises.name) LIKE $2 ESCAPE '\' THEN 4
                ELSE 5
            END,
            exercises.name ASC
        LIMIT $5
        "#,
    )
    .bind(&search.query)
    .bind(&contains_pattern)
    .bind(&prefix_pattern)
    .bind(search.modality.map(ExerciseModality::as_str))
    .bind(search.limit)
    .persistent(false)
    .fetch_all(pool)
    .await
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
    InvalidSearchQuery,
}

impl std::fmt::Display for ExerciseModelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLimit => write!(formatter, "exercise catalog limit must be 1 through 200"),
            Self::InvalidModality => write!(formatter, "exercise modality must be strength or cardio"),
            Self::InvalidSearchQuery => write!(
                formatter,
                "exercise search query must be 1 through 80 characters"
            ),
        }
    }
}

impl std::error::Error for ExerciseModelError {}

fn escape_like_pattern(value: &str) -> String {
    value
        .replace('\\', r"\\")
        .replace('%', r"\%")
        .replace('_', r"\_")
}

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

    #[test]
    fn validates_search_query_and_limit() {
        let search = ExerciseSearch::new(" bench ", Some(ExerciseModality::Strength), 8);
        assert_eq!(
            search,
            Ok(ExerciseSearch {
                query: "bench".to_owned(),
                modality: Some(ExerciseModality::Strength),
                limit: 8,
            })
        );
        assert_eq!(
            ExerciseSearch::new(" ", None, 8),
            Err(ExerciseModelError::InvalidSearchQuery)
        );
        assert_eq!(
            ExerciseSearch::new("bench", None, 21),
            Err(ExerciseModelError::InvalidLimit)
        );
    }

    #[test]
    fn escapes_like_pattern_wildcards() {
        assert_eq!(escape_like_pattern(r"50% incline_row"), r"50\% incline\_row");
    }
}
