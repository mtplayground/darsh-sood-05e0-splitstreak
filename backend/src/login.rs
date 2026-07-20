use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::{Extension, Json};
use serde::Serialize;

use crate::auth_middleware::CurrentUser;
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

pub async fn login(Extension(current_user): Extension<CurrentUser>) -> Json<LoginResponse> {
    let user = current_user.user;

    Json(LoginResponse {
        status: "authenticated",
        session: "mctai_session",
        user: AuthenticatedUser {
            sub: user.sub,
            email: user.email,
            email_verified: user.email_verified,
            name: user.name,
            picture_url: user.picture_url,
        },
    })
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
