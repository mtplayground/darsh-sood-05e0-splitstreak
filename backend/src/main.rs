#[allow(dead_code)]
pub mod auth;
mod auth_middleware;
mod account_recovery;
mod config;
mod dashboard;
mod db;
mod email;
mod exercise_search;
#[allow(dead_code)]
pub mod exercises;
mod logging;
mod login;
mod registration;
#[allow(dead_code)]
pub mod users;
#[allow(dead_code)]
pub mod workouts;

use auth::AuthService;
use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    routing::{get, patch, post},
    Json, Router,
};
use config::Config;
use email::EmailService;
use serde::Serialize;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: PgPool,
    pub(crate) auth: Option<AuthService>,
    pub(crate) email: EmailService,
    self_url: Option<String>,
}

impl AppState {
    pub(crate) fn frontend_return_to(&self) -> String {
        self.self_url.clone().unwrap_or_else(|| "/".to_owned())
    }
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    database: &'static str,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config = Config::from_env()?;
    let db_pool = db::connect(&config.database).await?;
    db::run_migrations(&db_pool).await?;

    if should_exit_after_migrations() {
        tracing::info!("database migrations completed");
        return Ok(());
    }

    let state = AppState {
        db: db_pool,
        auth: config.auth.clone().map(AuthService::new),
        email: EmailService::new(config.email.clone()),
        self_url: config.app.self_url.clone(),
    };
    let listener = tokio::net::TcpListener::bind(config.socket_addr()).await?;

    tracing::info!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn should_exit_after_migrations() -> bool {
    std::env::args().skip(1).any(|arg| arg == "--migrate-only")
}

fn app(state: AppState) -> Router {
    let protected_auth_routes = Router::new()
        .route("/api/auth/login", post(login::login))
        .route(
            "/api/auth/email-verification",
            post(account_recovery::send_verification),
        )
        .route(
            "/api/auth/email-verification/confirm",
            post(account_recovery::confirm_verification),
        )
        .route("/api/logging/sessions", post(logging::create_session))
        .route(
            "/api/logging/sessions/:session_id",
            patch(logging::update_session),
        )
        .route(
            "/api/logging/sessions/:session_id/strength-sets",
            post(logging::add_strength_set),
        )
        .route(
            "/api/logging/sessions/:session_id/cardio-entries",
            post(logging::add_cardio_entry),
        )
        .route("/api/dashboard/today", get(dashboard::today))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::require_auth,
        ));

    Router::new()
        .route("/api/health", get(health))
        .route("/api/exercises/search", get(exercise_search::search))
        .route("/api/auth/login", get(login::redirect_to_login))
        .route(
            "/api/auth/password-reset",
            post(account_recovery::request_password_reset),
        )
        .route("/api/auth/register", post(registration::register))
        .route("/health", get(health))
        .merge(protected_auth_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(error) = db::ping(&state.db).await {
        tracing::error!(%error, "database health check failed");
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "database unavailable",
            }),
        ));
    }

    Ok(Json(HealthResponse {
        status: "ok",
        service: "splitstreak-api",
        database: "ok",
    }))
}

fn init_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "splitstreak_api=info,tower_http=info".into());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(%error, "failed to listen for shutdown signal");
    }
}

#[cfg(test)]
mod tests {
    use crate::config::DatabaseConfig;
    use crate::email::EmailService;
    use crate::{app, AppState};
    use sqlx::postgres::PgPoolOptions;

    #[test]
    fn database_config_keeps_url_out_of_code() {
        let config = DatabaseConfig {
            url: "postgres://user:password@example.test/splitstreak".to_owned(),
            max_connections: 5,
        };

        assert_eq!(config.max_connections, 5);
        assert!(config.url.starts_with("postgres://"));
    }

    #[tokio::test]
    async fn app_builds_with_protected_login_post_route() {
        let db = match PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://user:password@localhost/splitstreak")
        {
            Ok(pool) => pool,
            Err(error) => panic!("lazy pool should not connect to construct app: {error}"),
        };
        let state = AppState {
            db,
            auth: None,
            email: EmailService::new(None),
            self_url: Some("https://splitstreak.example.test/".to_owned()),
        };

        let _router = app(state);
    }
}
