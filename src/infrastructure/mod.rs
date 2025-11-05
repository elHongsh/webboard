/// Infrastructure Layer
///
/// Contains cross-cutting concerns and infrastructure components:
/// - Configuration management
/// - Error handling and error types
/// - Logging setup
/// - Common utilities
///
/// This layer provides foundational services that all features can use.

pub mod config;
pub mod error;

pub use config::AppConfig;
pub use error::AppError;
