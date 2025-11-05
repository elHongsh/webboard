/// JSON-RPC Presentation Layer
///
/// Contains HTTP and WebSocket handlers for JSON-RPC communication.
///
/// ## Components
/// - `handler`: WebSocket connection and message handling
///
/// ## Responsibilities
/// - Handle WebSocket protocol (upgrade, ping/pong, close)
/// - Parse incoming messages
/// - Serialize outgoing messages
/// - Manage connection lifecycle
/// - Handle protocol errors

pub mod handler;

// Re-export commonly used types
pub use handler::websocket_handler;
