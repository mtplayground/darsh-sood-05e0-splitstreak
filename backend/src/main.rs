mod config;

use axum::{routing::get, Json, Router};
use config::Config;
use serde::Serialize;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config = Config::from_env()?;
    let listener = tokio::net::TcpListener::bind(config.socket_addr()).await?;

    tracing::info!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn app() -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "splitstreak-api",
    })
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
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_route_returns_ok() {
        let request = match Request::builder().uri("/api/health").body(Body::empty()) {
            Ok(request) => request,
            Err(error) => panic!("test request should be valid: {error}"),
        };

        let response = match app().oneshot(request).await {
            Ok(response) => response,
            Err(error) => panic!("health route should respond: {error}"),
        };

        assert_eq!(response.status(), StatusCode::OK);
    }
}
