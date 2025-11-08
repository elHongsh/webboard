// Module declarations
mod features;
mod infrastructure;

use axum::{
    extract::DefaultBodyLimit,
    http::{HeaderValue, Method},
    routing::{get, post},
    Router,
};
use infrastructure::AppConfig;
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
    let user_service = features::UserService::new();
    let jsonrpc_service = features::JsonRpcService::new();
    let auth_service = features::AuthService::new(config.jwt_secret.clone());

    // Give time for JSON-RPC builtin methods to register
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Build application with routes and middleware
    let app = build_app(config.clone(), user_service, jsonrpc_service, auth_service);

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
///
/// Organizes routes by feature with clear separation:
/// - Health check at /health
/// - WebSocket JSON-RPC at /live
/// - Auth API at /api/v1/auth
/// - Users API at /api/v1/users
fn build_app(
    config: AppConfig,
    user_service: features::UserService,
    jsonrpc_service: features::JsonRpcService,
    auth_service: features::AuthService,
) -> Router {
    // Build Auth API routes
    let auth_routes = Router::new()
        .route("/register", post(features::register))
        .route("/login", post(features::login))
        .route("/anonymous", post(features::anonymous_token))
        .route("/me", get(features::me).layer(axum::middleware::from_fn_with_state(
            auth_service.clone(),
            features::auth_middleware,
        )))
        .with_state(auth_service.clone());

    // Build Users API routes
    let api_routes = Router::new()
        .route(
            "/users",
            get(features::list_users).post(features::create_user),
        )
        .route("/users/:id", get(features::get_user))
        .with_state(user_service)
        .merge(Router::new().nest("/auth", auth_routes));

    // Build main router
    Router::new()
        // Health check endpoint
        .route("/health", get(features::health_check))
        // WebSocket JSON-RPC endpoint
        .route("/live", get(features::websocket_handler))
        .with_state(jsonrpc_service.clone())
        // Nest API routes under /api/v1
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
