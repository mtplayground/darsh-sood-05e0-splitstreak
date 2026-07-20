use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::EmailConfig;

#[derive(Debug, Clone)]
pub struct EmailService {
    config: Option<EmailConfig>,
    http: Client,
}

impl EmailService {
    pub fn new(config: Option<EmailConfig>) -> Self {
        Self {
            config,
            http: Client::new(),
        }
    }

    pub async fn send(&self, message: EmailMessage<'_>) -> Result<EmailDelivery, EmailError> {
        let Some(config) = &self.config else {
            return Ok(EmailDelivery::Skipped {
                reason: "email service is not configured",
            });
        };

        let response = self
            .http
            .post(&config.url)
            .bearer_auth(&config.app_token)
            .json(&message)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Ok(EmailDelivery::RateLimited);
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_else(|_| String::new());
            return Err(EmailError::Service { status, body });
        }

        let receipt = response.json::<EmailReceipt>().await?;
        Ok(EmailDelivery::Sent {
            message_id: receipt.id,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EmailMessage<'a> {
    pub to: &'a str,
    pub subject: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum EmailDelivery {
    Sent { message_id: String },
    Skipped { reason: &'static str },
    RateLimited,
}

#[derive(Debug, Deserialize)]
struct EmailReceipt {
    id: String,
}

#[derive(Debug)]
pub enum EmailError {
    Network(reqwest::Error),
    Service { status: u16, body: String },
}

impl std::fmt::Display for EmailError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(error) => write!(formatter, "email request failed: {error}"),
            Self::Service { status, body } => {
                write!(formatter, "email service returned {status}: {body}")
            }
        }
    }
}

impl std::error::Error for EmailError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Network(error) => Some(error),
            Self::Service { .. } => None,
        }
    }
}

impl From<reqwest::Error> for EmailError {
    fn from(error: reqwest::Error) -> Self {
        Self::Network(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn skips_send_when_email_service_is_not_configured() {
        let service = EmailService::new(None);
        let delivery = service
            .send(EmailMessage {
                to: "person@example.com",
                subject: "Verify",
                html: Some("<p>Verify</p>"),
                text: None,
                reply_to: None,
            })
            .await;

        match delivery {
            Ok(EmailDelivery::Skipped { reason }) => {
                assert_eq!(reason, "email service is not configured");
            }
            Ok(other) => panic!("expected skipped delivery, got {other:?}"),
            Err(error) => panic!("expected skipped delivery, got error: {error}"),
        }
    }
}
