mod config;
mod error;
mod handlers;
mod models;
mod services;

use axum::{
    extract::DefaultBodyLimit,
    http::{HeaderValue, Method},
    routing::get,
    Router,
};
use config::AppConfig;
use services::UserService;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = AppConfig::from_env()?;

    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_level.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting server with config: {:?}", config);

    // Initialize services
    let user_service = UserService::new();

    // Build application with routes and middleware
    let app = build_app(config.clone(), user_service);

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(&config.address()).await?;
    tracing::info!("Server listening on {}", config.address());

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}

/// Build the application router with all routes and middleware
fn build_app(config: AppConfig, user_service: UserService) -> Router {
    // Build API routes
    let api_routes = Router::new()
        .route("/users", get(handlers::list_users).post(handlers::create_user))
        .route("/users/:id", get(handlers::get_user))
        .with_state(user_service);

    // Build main router
    Router::new()
        .route("/health", get(handlers::health_check))
        .nest("/api/v1", api_routes)
        // Set a request body size limit
        .layer(DefaultBodyLimit::max(config.max_body_size))
        // Add middleware stack
        .layer(
            ServiceBuilder::new()
                // Add tracing for request/response logging
                .layer(TraceLayer::new_for_http())
                // Add CORS support
                .layer(
                    CorsLayer::new()
                        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                        .allow_headers(tower_http::cors::Any),
                )
                // Add request timeout
                .layer(TimeoutLayer::new(Duration::from_secs(
                    config.request_timeout_secs,
                ))),
        )
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal, shutting down gracefully...");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, shutting down gracefully...");
        },
    }
}
