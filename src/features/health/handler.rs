use axum::Json;

use super::domain::HealthResponse;

/// Health check handler
///
/// Presentation layer handler for the health check endpoint.
/// Returns the current health status of the service.
///
/// # Route
/// GET /health
///
/// # Response
/// ```json
/// {
///   "status": "healthy",
///   "version": "0.1.0"
/// }
/// ```
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse::healthy())
}
