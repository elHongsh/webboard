/// Health Check Feature
///
/// Provides a simple health check endpoint to verify service availability.
/// This is a lightweight feature with only domain and presentation layers.
///
/// ## Architecture
/// - `domain`: Health response model
/// - `handler`: HTTP handler for the health endpoint
///
/// ## Usage
/// ```rust
/// use features::health;
///
/// Router::new()
///     .route("/health", get(health::handler::health_check))
/// ```

pub mod domain;
pub mod handler;

// Re-export commonly used items
pub use domain::HealthResponse;
pub use handler::health_check;
