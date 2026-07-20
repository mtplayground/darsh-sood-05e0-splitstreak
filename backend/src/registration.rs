use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::Json;
use serde::Serialize;

use crate::email::{EmailDelivery, EmailMessage};
use crate::users;
use crate::AppState;

pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<RegistrationResponse>), (StatusCode, Json<RegistrationError>)> {
    let Some(auth) = &state.auth else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(RegistrationError {
                error: "auth service is not configured",
                login_url: None,
            }),
        ));
    };

    let cookie_header = headers.get(header::COOKIE).and_then(|value| value.to_str().ok());
    let session = match auth.verify_session_cookie_header(cookie_header).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(RegistrationError {
                    error: "authentication required",
                    login_url: Some(auth.login_url(&state.frontend_return_to())),
                }),
            ));
        }
        Err(error) => {
            tracing::warn!(%error, "registration rejected invalid session cookie");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(RegistrationError {
                    error: "invalid session",
                    login_url: Some(auth.login_url(&state.frontend_return_to())),
                }),
            ));
        }
    };

    let user = users::upsert_from_profile(&state.db, &session.profile)
        .await
        .map_err(|error| {
            tracing::error!(%error, "failed to upsert registered user");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegistrationError {
                    error: "registration failed",
                    login_url: None,
                }),
            )
        })?;

    let email = if session.profile.email_verified {
        EmailDelivery::Skipped {
            reason: "email is already verified",
        }
    } else {
        send_verification_email(&state, auth.login_url(&state.frontend_return_to()).as_str(), &user)
            .await
    };

    Ok((
        StatusCode::CREATED,
        Json(RegistrationResponse {
            status: "registered",
            user: RegisteredUser {
                sub: user.sub,
                email: user.email,
                email_verified: user.email_verified,
                name: user.name,
                picture_url: user.picture_url,
            },
            email,
        }),
    ))
}

async fn send_verification_email(
    state: &AppState,
    verification_url: &str,
    user: &users::User,
) -> EmailDelivery {
    let display_name = escape_html(user.name.as_deref().unwrap_or("there"));
    let verification_url = escape_html(verification_url);
    let html = format!(
        "<p>Hi {display_name},</p>\
         <p>Use this link to continue email verification for SplitStreak:</p>\
         <p><a href=\"{verification_url}\">Verify email</a></p>"
    );

    match state
        .email
        .send(EmailMessage {
            to: &user.email,
            subject: "Verify your SplitStreak email",
            html: Some(&html),
            text: Some("Open SplitStreak to continue email verification."),
            reply_to: None,
        })
        .await
    {
        Ok(delivery) => delivery,
        Err(error) => {
            tracing::warn!(%error, "verification email could not be sent");
            EmailDelivery::Skipped {
                reason: "verification email could not be sent",
            }
        }
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[derive(Debug, Serialize)]
pub struct RegistrationResponse {
    pub status: &'static str,
    pub user: RegisteredUser,
    pub email: EmailDelivery,
}

#[derive(Debug, Serialize)]
pub struct RegisteredUser {
    pub sub: String,
    pub email: String,
    pub email_verified: bool,
    pub name: Option<String>,
    pub picture_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegistrationError {
    pub error: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_url: Option<String>,
}
