use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct User {
    pub sub: String,
    pub email: String,
    pub email_verified: bool,
    pub name: Option<String>,
    pub picture_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserProfile {
    pub sub: String,
    pub email: String,
    pub email_verified: bool,
    pub name: Option<String>,
    pub picture_url: Option<String>,
}

impl UserProfile {
    pub fn from_session_claims(
        sub: impl Into<String>,
        email: impl Into<String>,
        email_verified: bool,
        name: Option<String>,
        picture_url: Option<String>,
    ) -> Result<Self, UserModelError> {
        let sub = normalize_required("sub", sub.into())?;
        let email = normalize_required("email", email.into())?;

        Ok(Self {
            sub,
            email,
            email_verified,
            name: normalize_optional(name),
            picture_url: normalize_optional(picture_url),
        })
    }
}

pub async fn upsert_from_profile(
    pool: &PgPool,
    profile: &UserProfile,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (sub, email, email_verified, name, picture_url)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (sub) DO UPDATE
        SET
            email = EXCLUDED.email,
            email_verified = EXCLUDED.email_verified,
            name = EXCLUDED.name,
            picture_url = EXCLUDED.picture_url,
            last_seen_at = NOW()
        RETURNING
            sub,
            email,
            email_verified,
            name,
            picture_url,
            created_at,
            updated_at,
            last_seen_at
        "#,
    )
    .bind(&profile.sub)
    .bind(&profile.email)
    .bind(profile.email_verified)
    .bind(&profile.name)
    .bind(&profile.picture_url)
    .persistent(false)
    .fetch_one(pool)
    .await
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT
            sub,
            email,
            email_verified,
            name,
            picture_url,
            created_at,
            updated_at,
            last_seen_at
        FROM users
        WHERE lower(email) = lower($1)
        "#,
    )
    .bind(email)
    .persistent(false)
    .fetch_optional(pool)
    .await
}

fn normalize_required(field: &'static str, value: String) -> Result<String, UserModelError> {
    let normalized = value.trim().to_owned();
    if normalized.is_empty() {
        return Err(UserModelError::RequiredFieldEmpty { field });
    }

    Ok(normalized)
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let normalized = item.trim().to_owned();
        if normalized.is_empty() {
            None
        } else {
            Some(normalized)
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserModelError {
    RequiredFieldEmpty { field: &'static str },
}

impl std::fmt::Display for UserModelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequiredFieldEmpty { field } => write!(formatter, "{field} must not be empty"),
        }
    }
}

impl std::error::Error for UserModelError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_claim_profile_trims_values() {
        let profile = UserProfile::from_session_claims(
            " user-sub ",
            " person@example.com ",
            true,
            Some(" Person ".to_owned()),
            Some(" ".to_owned()),
        );

        let profile = match profile {
            Ok(profile) => profile,
            Err(error) => panic!("valid claims should produce a profile: {error}"),
        };

        assert_eq!(profile.sub, "user-sub");
        assert_eq!(profile.email, "person@example.com");
        assert!(profile.email_verified);
        assert_eq!(profile.name.as_deref(), Some("Person"));
        assert_eq!(profile.picture_url, None);
    }

    #[test]
    fn session_claim_profile_rejects_empty_sub() {
        let error = match UserProfile::from_session_claims(
            " ",
            "person@example.com",
            true,
            None,
            None,
        ) {
            Ok(profile) => panic!("empty sub should be rejected: {profile:?}"),
            Err(error) => error,
        };

        assert_eq!(
            error,
            UserModelError::RequiredFieldEmpty { field: "sub" }
        );
    }
}
