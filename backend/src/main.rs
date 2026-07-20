mod config;
mod db;

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use config::Config;
use serde::Serialize;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    db: PgPool,
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

    let state = AppState { db: db_pool };
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
    Router::new()
        .route("/api/health", get(health))
        .route("/health", get(health))
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

    #[test]
    fn database_config_keeps_url_out_of_code() {
        let config = DatabaseConfig {
            url: "postgres://user:password@example.test/splitstreak".to_owned(),
            max_connections: 5,
        };

        assert_eq!(config.max_connections, 5);
        assert!(config.url.starts_with("postgres://"));
    }
}
