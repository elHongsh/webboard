use serde::Serialize;

/// Health check response model
///
/// Domain entity representing the health status of the service.
/// Contains minimal information needed to verify service availability.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Current health status
    pub status: String,
    /// Application version
    pub version: String,
}

impl HealthResponse {
    /// Create a healthy response
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}
