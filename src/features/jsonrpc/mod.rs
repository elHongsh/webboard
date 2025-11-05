/// JSON-RPC Feature Module
///
/// Implements WebSocket-based JSON-RPC 2.0 protocol for real-time bidirectional communication.
///
/// ## Architecture
///
/// This module follows clean architecture with three distinct layers:
///
/// ### Domain Layer (`domain/`)
/// - `message`: Request, Response, Error message types
/// - `error_code`: Standard JSON-RPC error codes and error objects
/// - Protocol validation and business rules
/// - No external dependencies
///
/// ### Application Layer (`application/`)
/// - `service`: JSON-RPC service with method registry
/// - Business logic orchestration
/// - Method registration and dispatching
/// - Request/response handling
///
/// ### Presentation Layer (`presentation/`)
/// - `handler`: WebSocket connection handler
/// - HTTP upgrade handling
/// - Message serialization/deserialization
/// - Connection lifecycle management
///
/// ## Usage
///
/// ```rust
/// use features::jsonrpc;
///
/// // Initialize service
/// let jsonrpc_service = jsonrpc::JsonRpcService::new();
///
/// // Register custom method
/// jsonrpc_service.register_method("myMethod".to_string(), |params| async move {
///     // Your logic here
///     Ok(json!({"result": "success"}))
/// }).await;
///
/// // Add WebSocket route
/// Router::new()
///     .route("/live", get(jsonrpc::websocket_handler))
///     .with_state(jsonrpc_service)
/// ```
///
/// ## Built-in Methods
///
/// - `ping`: Health check with timestamp
/// - `echo`: Echo back parameters
/// - `add`: Add two numbers
/// - `getServerInfo`: Get server information
///
/// ## Protocol
///
/// Implements JSON-RPC 2.0 specification:
/// - Request/Response pattern
/// - Notifications (one-way messages)
/// - Standard error codes
/// - Parameter validation

pub mod application;
pub mod domain;
pub mod presentation;

// Re-export commonly used types for convenience
pub use application::JsonRpcService;
pub use domain::{
    JsonRpcErrorCode, JsonRpcErrorObject, JsonRpcErrorResponse, JsonRpcMessage, JsonRpcRequest,
    JsonRpcResponse,
};
pub use presentation::websocket_handler;
