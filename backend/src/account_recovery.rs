use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use crate::auth_middleware::CurrentUser;
use crate::email::{EmailDelivery, EmailMessage};
use crate::users;
use crate::AppState;

pub async fn send_verification(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<VerificationEmailResponse>, (StatusCode, Json<AccountRecoveryError>)> {
    let user = current_user.user;
    let delivery = if user.email_verified {
        EmailDelivery::Skipped {
            reason: "email is already verified",
        }
    } else {
        send_email_verification_link(&state, &user).await
    };

    Ok(Json(VerificationEmailResponse {
        status: "verification_email_processed",
        email_verified: user.email_verified,
        delivery,
    }))
}

pub async fn confirm_verification(
    Extension(current_user): Extension<CurrentUser>,
) -> Json<VerificationConfirmResponse> {
    Json(VerificationConfirmResponse {
        status: if current_user.user.email_verified {
            "verified"
        } else {
            "pending"
        },
        email_verified: current_user.user.email_verified,
    })
}

pub async fn request_password_reset(
    State(state): State<AppState>,
    Json(request): Json<PasswordResetRequest>,
) -> Result<(StatusCode, Json<PasswordResetResponse>), (StatusCode, Json<AccountRecoveryError>)> {
    let email = normalize_email(&request.email).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(AccountRecoveryError { error: message }),
        )
    })?;

    let Some(auth) = &state.auth else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(AccountRecoveryError {
                error: "auth service is not configured",
            }),
        ));
    };

    let user = users::find_by_email(&state.db, &email)
        .await
        .map_err(|error| {
            tracing::error!(%error, "failed to look up password reset user");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AccountRecoveryError {
                    error: "password reset request failed",
                }),
            )
        })?;

    if let Some(user) = user {
        let login_url = auth.login_url(&state.frontend_return_to());
        let delivery = send_password_reset_link(&state, &user, &login_url).await;
        log_delivery_outcome("password reset email", &delivery);
    }

    Ok((
        StatusCode::ACCEPTED,
        Json(PasswordResetResponse {
            status: "accepted",
            message: "If an account exists for that email, a recovery link will be sent.",
        }),
    ))
}

async fn send_email_verification_link(state: &AppState, user: &users::User) -> EmailDelivery {
    let Some(auth) = &state.auth else {
        return EmailDelivery::Skipped {
            reason: "auth service is not configured",
        };
    };

    let verification_url = auth.login_url(&state.frontend_return_to());
    let display_name = escape_html(user.name.as_deref().unwrap_or("there"));
    let verification_url = escape_html(&verification_url);
    let html = format!(
        "<p>Hi {display_name},</p>\
         <p>Open this SplitStreak link to complete email verification through the secure sign-in service:</p>\
         <p><a href=\"{verification_url}\">Verify email</a></p>"
    );

    send_email(
        state,
        EmailMessage {
            to: &user.email,
            subject: "Verify your SplitStreak email",
            html: Some(&html),
            text: Some("Open SplitStreak to complete email verification."),
            reply_to: None,
        },
        "verification email",
    )
    .await
}

async fn send_password_reset_link(
    state: &AppState,
    user: &users::User,
    login_url: &str,
) -> EmailDelivery {
    let display_name = escape_html(user.name.as_deref().unwrap_or("there"));
    let login_url = escape_html(login_url);
    let html = format!(
        "<p>Hi {display_name},</p>\
         <p>Use this SplitStreak sign-in link to recover access to your account:</p>\
         <p><a href=\"{login_url}\">Recover account access</a></p>\
         <p>If you did not request this, you can ignore this email.</p>"
    );

    send_email(
        state,
        EmailMessage {
            to: &user.email,
            subject: "Recover your SplitStreak account",
            html: Some(&html),
            text: Some("Open SplitStreak to recover account access."),
            reply_to: None,
        },
        "password reset email",
    )
    .await
}

async fn send_email(
    state: &AppState,
    message: EmailMessage<'_>,
    label: &'static str,
) -> EmailDelivery {
    match state.email.send(message).await {
        Ok(delivery) => delivery,
        Err(error) => {
            tracing::warn!(%error, "{label} could not be sent");
            EmailDelivery::Skipped {
                reason: "email could not be sent",
            }
        }
    }
}

fn log_delivery_outcome(label: &'static str, delivery: &EmailDelivery) {
    match delivery {
        EmailDelivery::Sent { .. } => tracing::info!("{label} sent"),
        EmailDelivery::RateLimited => tracing::warn!("{label} rate limited"),
        EmailDelivery::Skipped { reason } => tracing::info!(%reason, "{label} skipped"),
    }
}

fn normalize_email(email: &str) -> Result<String, &'static str> {
    let email = email.trim();
    if email.is_empty() {
        return Err("email is required");
    }

    if !email.contains('@') {
        return Err("email is invalid");
    }

    Ok(email.to_owned())
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
pub struct VerificationEmailResponse {
    pub status: &'static str,
    pub email_verified: bool,
    pub delivery: EmailDelivery,
}

#[derive(Debug, Serialize)]
pub struct VerificationConfirmResponse {
    pub status: &'static str,
    pub email_verified: bool,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct PasswordResetResponse {
    pub status: &'static str,
    pub message: &'static str,
}

#[derive(Debug, Serialize)]
pub struct AccountRecoveryError {
    pub error: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_password_reset_email() {
        let email = normalize_email(" Person@Example.COM ");

        assert_eq!(email.as_deref(), Ok("Person@Example.COM"));
    }

    #[test]
    fn rejects_blank_password_reset_email() {
        assert_eq!(normalize_email(" "), Err("email is required"));
    }

    #[test]
    fn rejects_malformed_password_reset_email() {
        assert_eq!(normalize_email("person.example.com"), Err("email is invalid"));
    }

    #[test]
    fn escapes_email_html() {
        assert_eq!(
            escape_html("<a href=\"x\">O'Reilly & Co</a>"),
            "&lt;a href=&quot;x&quot;&gt;O&#39;Reilly &amp; Co&lt;/a&gt;"
        );
    }
}
