use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderMap, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::users;
use crate::AppState;

#[derive(Debug, Clone)]
pub(crate) struct CurrentUser {
    pub user: users::User,
}

pub(crate) async fn require_auth(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AuthRejection> {
    let current_user = current_user_from_headers(&state, request.headers()).await?;
    request.extensions_mut().insert(current_user);

    Ok(next.run(request).await)
}

pub(crate) async fn current_user_from_headers(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<CurrentUser, AuthRejection> {
    let cookie_header = headers.get(header::COOKIE).and_then(|value| value.to_str().ok());

    current_user_from_cookie_header(state, cookie_header).await
}

pub(crate) async fn current_user_from_cookie_header(
    state: &AppState,
    cookie_header: Option<&str>,
) -> Result<CurrentUser, AuthRejection> {
    let Some(auth) = &state.auth else {
        return Err(AuthRejection::auth_not_configured());
    };

    let session = match auth.verify_session_cookie_header(cookie_header).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            return Err(AuthRejection::authentication_required(
                auth.login_url(&state.frontend_return_to()),
            ));
        }
        Err(error) => {
            tracing::warn!(%error, "request rejected invalid session cookie");
            return Err(AuthRejection::invalid_session(
                auth.login_url(&state.frontend_return_to()),
            ));
        }
    };

    let user = users::upsert_from_profile(&state.db, &session.profile)
        .await
        .map_err(|error| {
            tracing::error!(%error, "failed to upsert authenticated user");
            AuthRejection::authentication_failed()
        })?;

    Ok(CurrentUser { user })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuthRejection {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
    login_url: Option<String>,
}

impl AuthRejection {
    fn auth_not_configured() -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "auth_not_configured",
            message: "Sign-in is temporarily unavailable.",
            login_url: None,
        }
    }

    fn authentication_required(login_url: String) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "authentication_required",
            message: "Sign in to continue.",
            login_url: Some(login_url),
        }
    }

    fn invalid_session(login_url: String) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "invalid_session",
            message: "Your session expired. Sign in again.",
            login_url: Some(login_url),
        }
    }

    fn authentication_failed() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "authentication_failed",
            message: "Sign-in failed. Try again shortly.",
            login_url: None,
        }
    }

    pub(crate) fn status(&self) -> StatusCode {
        self.status
    }

    pub(crate) fn code(&self) -> &'static str {
        self.code
    }

    pub(crate) fn message(&self) -> &'static str {
        self.message
    }

    pub(crate) fn login_url(&self) -> Option<&str> {
        self.login_url.as_deref()
    }
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        let status = self.status();
        let error = self.code();
        let message = self.message();
        let login_url = self.login_url().map(ToOwned::to_owned);

        (
            status,
            Json(AuthErrorResponse {
                error,
                message,
                login_url,
            }),
        )
            .into_response()
    }
}

#[derive(Debug, Serialize)]
struct AuthErrorResponse {
    error: &'static str,
    message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    login_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unauthenticated_rejection_includes_login_url() {
        let rejection = AuthRejection::authentication_required(
            "https://auth.mctai.app/login?app_token=app&return_to=https%3A%2F%2Fapp.test%2F"
                .to_owned(),
        );

        assert_eq!(rejection.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(rejection.code(), "authentication_required");
        assert_eq!(rejection.message(), "Sign in to continue.");
        assert_eq!(
            rejection.login_url(),
            Some("https://auth.mctai.app/login?app_token=app&return_to=https%3A%2F%2Fapp.test%2F")
        );
    }

    #[test]
    fn unconfigured_auth_rejection_has_no_login_url() {
        let rejection = AuthRejection::auth_not_configured();

        assert_eq!(rejection.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(rejection.code(), "auth_not_configured");
        assert_eq!(rejection.message(), "Sign-in is temporarily unavailable.");
        assert_eq!(rejection.login_url(), None);
    }
}
