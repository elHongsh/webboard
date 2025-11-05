use std::env;

/// Application configuration loaded from environment variables
#[derive(Clone, Debug)]
pub struct AppConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Maximum request body size in bytes
    pub max_body_size: usize,
}

impl AppConfig {
    /// Load configuration from environment variables with sensible defaults
    pub fn from_env() -> anyhow::Result<Self> {
        // Load .env file if present (ignored in production)
        let _ = dotenvy::dotenv();

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        let request_timeout_secs = env::var("REQUEST_TIMEOUT_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);
        let max_body_size = env::var("MAX_BODY_SIZE")
            .unwrap_or_else(|_| "2097152".to_string()) // 2MB default
            .parse()
            .unwrap_or(2_097_152);

        Ok(Self {
            host,
            port,
            log_level,
            request_timeout_secs,
            max_body_size,
        })
    }

    /// Get server address in format "host:port"
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
