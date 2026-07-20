use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::AuthConfig;
use crate::users::{UserModelError, UserProfile};

const SESSION_COOKIE_NAME: &str = "mctai_session";

#[derive(Debug, Clone)]
pub struct AuthService {
    config: AuthConfig,
    http: Client,
}

impl AuthService {
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config,
            http: Client::new(),
        }
    }

    pub fn login_url(&self, return_to: &str) -> String {
        format!(
            "{}/login?app_token={}&return_to={}",
            self.config.url.trim_end_matches('/'),
            urlencoding::encode(&self.config.app_token),
            urlencoding::encode(return_to)
        )
    }

    pub async fn verify_session_cookie_header(
        &self,
        cookie_header: Option<&str>,
    ) -> Result<Option<VerifiedSession>, AuthError> {
        let Some(token) = session_cookie_value(cookie_header) else {
            return Ok(None);
        };

        let claims = self.verify_session_token(&token).await?;
        let profile = UserProfile::from_session_claims(
            claims.sub.clone(),
            claims.email.clone(),
            claims.email_verified,
            claims.name.clone(),
            claims.picture.clone(),
        )?;

        Ok(Some(VerifiedSession { claims, profile }))
    }

    async fn verify_session_token(&self, token: &str) -> Result<MctaiSessionClaims, AuthError> {
        let header = decode_header(token)?;
        let key_id = header.kid.ok_or(AuthError::MissingKeyId)?;
        let jwk_set = self.fetch_jwks().await?;
        let jwk = jwk_set
            .keys
            .iter()
            .find(|key| key.common.key_id.as_deref() == Some(key_id.as_str()))
            .ok_or(AuthError::SigningKeyNotFound)?;
        let decoding_key = DecodingKey::from_jwk(jwk)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.config.app_token.as_str()]);
        validation.set_issuer(&[self.config.url.as_str()]);

        decode::<MctaiSessionClaims>(token, &decoding_key, &validation)
            .map(|token_data| token_data.claims)
            .map_err(AuthError::from)
    }

    async fn fetch_jwks(&self) -> Result<JwkSet, AuthError> {
        let response = self
            .http
            .get(&self.config.jwks_url)
            .send()
            .await?
            .error_for_status()?;

        response.json::<JwkSet>().await.map_err(AuthError::from)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifiedSession {
    pub claims: MctaiSessionClaims,
    pub profile: UserProfile,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MctaiSessionClaims {
    pub sub: String,
    pub email: String,
    #[serde(default)]
    pub email_verified: bool,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub picture: Option<String>,
    pub aud: serde_json::Value,
    pub iss: String,
    pub exp: usize,
    #[serde(default)]
    pub iat: Option<usize>,
}

pub fn session_cookie_value(cookie_header: Option<&str>) -> Option<String> {
    cookie_header.and_then(|header| {
        header.split(';').find_map(|cookie| {
            let (name, value) = cookie.trim().split_once('=')?;
            if name == SESSION_COOKIE_NAME && !value.is_empty() {
                Some(value.to_owned())
            } else {
                None
            }
        })
    })
}

#[derive(Debug)]
pub enum AuthError {
    Claims(UserModelError),
    Jwt(jsonwebtoken::errors::Error),
    MissingKeyId,
    Network(reqwest::Error),
    SigningKeyNotFound,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Claims(error) => write!(formatter, "session claims are invalid: {error}"),
            Self::Jwt(error) => write!(formatter, "session token is invalid: {error}"),
            Self::MissingKeyId => write!(formatter, "session token header is missing a key id"),
            Self::Network(error) => write!(formatter, "auth service request failed: {error}"),
            Self::SigningKeyNotFound => {
                write!(formatter, "session token signing key was not found")
            }
        }
    }
}

impl std::error::Error for AuthError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Claims(error) => Some(error),
            Self::Jwt(error) => Some(error),
            Self::Network(error) => Some(error),
            Self::MissingKeyId | Self::SigningKeyNotFound => None,
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        Self::Jwt(error)
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(error: reqwest::Error) -> Self {
        Self::Network(error)
    }
}

impl From<UserModelError> for AuthError {
    fn from(error: UserModelError) -> Self {
        Self::Claims(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_mctai_session_cookie() {
        let value = session_cookie_value(Some(
            "theme=dark; mctai_session=header.payload.signature; other=value",
        ));

        assert_eq!(value.as_deref(), Some("header.payload.signature"));
    }

    #[test]
    fn ignores_missing_session_cookie() {
        assert_eq!(session_cookie_value(Some("theme=dark")), None);
        assert_eq!(session_cookie_value(None), None);
    }
}
