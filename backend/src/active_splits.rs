use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct ActiveSplit {
    pub user_sub: String,
    pub split_template_id: i64,
    pub template_slug: String,
    pub template_name: String,
    pub depth_level: String,
    pub schedule: Vec<String>,
    pub training_days_per_cycle: i32,
    pub rest_days_per_cycle: i32,
    pub selected_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveSplitSelection {
    TemplateId(i64),
    TemplateSlug(String),
}

impl ActiveSplitSelection {
    pub fn by_template_id(template_id: i64) -> Result<Self, ActiveSplitModelError> {
        if template_id <= 0 {
            return Err(ActiveSplitModelError::InvalidTemplateId);
        }

        Ok(Self::TemplateId(template_id))
    }

    pub fn by_template_slug(slug: impl Into<String>) -> Result<Self, ActiveSplitModelError> {
        let slug = slug.into();
        let slug = slug.trim();
        if slug.is_empty() {
            return Err(ActiveSplitModelError::InvalidTemplateSlug);
        }

        if slug.chars().count() > 120 {
            return Err(ActiveSplitModelError::InvalidTemplateSlug);
        }

        Ok(Self::TemplateSlug(slug.to_owned()))
    }
}

pub async fn find_active_split(
    pool: &PgPool,
    user_sub: &str,
) -> Result<Option<ActiveSplit>, sqlx::Error> {
    sqlx::query_as::<_, ActiveSplit>(
        r#"
        SELECT
            user_sub,
            split_template_id,
            template_slug,
            template_name,
            depth_level,
            schedule,
            training_days_per_cycle,
            rest_days_per_cycle,
            selected_at,
            updated_at
        FROM user_active_splits
        WHERE user_sub = $1
        "#,
    )
    .bind(user_sub)
    .persistent(false)
    .fetch_optional(pool)
    .await
}

pub async fn select_active_split(
    pool: &PgPool,
    user_sub: &str,
    selection: &ActiveSplitSelection,
) -> Result<Option<ActiveSplit>, sqlx::Error> {
    match selection {
        ActiveSplitSelection::TemplateId(template_id) => {
            select_active_split_by_template_id(pool, user_sub, *template_id).await
        }
        ActiveSplitSelection::TemplateSlug(slug) => {
            select_active_split_by_template_slug(pool, user_sub, slug).await
        }
    }
}

async fn select_active_split_by_template_id(
    pool: &PgPool,
    user_sub: &str,
    template_id: i64,
) -> Result<Option<ActiveSplit>, sqlx::Error> {
    sqlx::query_as::<_, ActiveSplit>(
        r#"
        INSERT INTO user_active_splits (
            user_sub,
            split_template_id,
            template_slug,
            template_name,
            depth_level,
            schedule,
            training_days_per_cycle,
            rest_days_per_cycle
        )
        SELECT
            $1,
            id,
            slug,
            name,
            depth_level,
            schedule,
            training_days_per_cycle,
            rest_days_per_cycle
        FROM split_templates
        WHERE id = $2
        ON CONFLICT (user_sub) DO UPDATE
        SET
            split_template_id = EXCLUDED.split_template_id,
            template_slug = EXCLUDED.template_slug,
            template_name = EXCLUDED.template_name,
            depth_level = EXCLUDED.depth_level,
            schedule = EXCLUDED.schedule,
            training_days_per_cycle = EXCLUDED.training_days_per_cycle,
            rest_days_per_cycle = EXCLUDED.rest_days_per_cycle,
            selected_at = NOW()
        RETURNING
            user_sub,
            split_template_id,
            template_slug,
            template_name,
            depth_level,
            schedule,
            training_days_per_cycle,
            rest_days_per_cycle,
            selected_at,
            updated_at
        "#,
    )
    .bind(user_sub)
    .bind(template_id)
    .persistent(false)
    .fetch_optional(pool)
    .await
}

async fn select_active_split_by_template_slug(
    pool: &PgPool,
    user_sub: &str,
    slug: &str,
) -> Result<Option<ActiveSplit>, sqlx::Error> {
    sqlx::query_as::<_, ActiveSplit>(
        r#"
        INSERT INTO user_active_splits (
            user_sub,
            split_template_id,
            template_slug,
            template_name,
            depth_level,
            schedule,
            training_days_per_cycle,
            rest_days_per_cycle
        )
        SELECT
            $1,
            id,
            slug,
            name,
            depth_level,
            schedule,
            training_days_per_cycle,
            rest_days_per_cycle
        FROM split_templates
        WHERE slug = $2
        ON CONFLICT (user_sub) DO UPDATE
        SET
            split_template_id = EXCLUDED.split_template_id,
            template_slug = EXCLUDED.template_slug,
            template_name = EXCLUDED.template_name,
            depth_level = EXCLUDED.depth_level,
            schedule = EXCLUDED.schedule,
            training_days_per_cycle = EXCLUDED.training_days_per_cycle,
            rest_days_per_cycle = EXCLUDED.rest_days_per_cycle,
            selected_at = NOW()
        RETURNING
            user_sub,
            split_template_id,
            template_slug,
            template_name,
            depth_level,
            schedule,
            training_days_per_cycle,
            rest_days_per_cycle,
            selected_at,
            updated_at
        "#,
    )
    .bind(user_sub)
    .bind(slug)
    .persistent(false)
    .fetch_optional(pool)
    .await
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveSplitModelError {
    AmbiguousSelection,
    InvalidTemplateId,
    InvalidTemplateSlug,
    MissingSelection,
}

impl std::fmt::Display for ActiveSplitModelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AmbiguousSelection => write!(formatter, "select either split_template_id or split_template_slug"),
            Self::InvalidTemplateId => write!(formatter, "split_template_id must be positive"),
            Self::InvalidTemplateSlug => write!(formatter, "split_template_slug must not be empty"),
            Self::MissingSelection => write!(formatter, "a split template selection is required"),
        }
    }
}

impl std::error::Error for ActiveSplitModelError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_template_id_selection() {
        assert_eq!(
            ActiveSplitSelection::by_template_id(42),
            Ok(ActiveSplitSelection::TemplateId(42))
        );
        assert_eq!(
            ActiveSplitSelection::by_template_id(0),
            Err(ActiveSplitModelError::InvalidTemplateId)
        );
    }

    #[test]
    fn trims_template_slug_selection() {
        assert_eq!(
            ActiveSplitSelection::by_template_slug(" upper-lower-4-day "),
            Ok(ActiveSplitSelection::TemplateSlug(
                "upper-lower-4-day".to_owned()
            ))
        );
        assert_eq!(
            ActiveSplitSelection::by_template_slug(" "),
            Err(ActiveSplitModelError::InvalidTemplateSlug)
        );
    }
}
