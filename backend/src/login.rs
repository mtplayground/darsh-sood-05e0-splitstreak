use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::Redirect;
use axum::Json;
use serde::Serialize;

use crate::users;
use crate::AppState;

pub async fn redirect_to_login(
    State(state): State<AppState>,
) -> Result<Redirect, (StatusCode, Json<LoginError>)> {
    let Some(auth) = &state.auth else {
        return Err(auth_not_configured());
    };

    Ok(Redirect::temporary(
        &auth.login_url(&state.frontend_return_to()),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<LoginResponse>, (StatusCode, Json<LoginError>)> {
    let Some(auth) = &state.auth else {
        return Err(auth_not_configured());
    };

    let cookie_header = headers.get(header::COOKIE).and_then(|value| value.to_str().ok());
    let session = match auth.verify_session_cookie_header(cookie_header).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(LoginError {
                    error: "authentication required",
                    login_url: Some(auth.login_url(&state.frontend_return_to())),
                }),
            ));
        }
        Err(error) => {
            tracing::warn!(%error, "login rejected invalid session cookie");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(LoginError {
                    error: "invalid session",
                    login_url: Some(auth.login_url(&state.frontend_return_to())),
                }),
            ));
        }
    };

    let user = users::upsert_from_profile(&state.db, &session.profile)
        .await
        .map_err(|error| {
            tracing::error!(%error, "failed to upsert logged-in user");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginError {
                    error: "login failed",
                    login_url: None,
                }),
            )
        })?;

    Ok(Json(LoginResponse {
        status: "authenticated",
        session: "mctai_session",
        user: AuthenticatedUser {
            sub: user.sub,
            email: user.email,
            email_verified: user.email_verified,
            name: user.name,
            picture_url: user.picture_url,
        },
    }))
}

fn auth_not_configured() -> (StatusCode, Json<LoginError>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(LoginError {
            error: "auth service is not configured",
            login_url: None,
        }),
    )
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub status: &'static str,
    pub session: &'static str,
    pub user: AuthenticatedUser,
}

#[derive(Debug, Serialize)]
pub struct AuthenticatedUser {
    pub sub: String,
    pub email: String,
    pub email_verified: bool,
    pub name: Option<String>,
    pub picture_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginError {
    pub error: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_not_configured_returns_service_unavailable() {
        let (status, Json(error)) = auth_not_configured();

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(error.error, "auth service is not configured");
        assert_eq!(error.login_url, None);
    }
}
