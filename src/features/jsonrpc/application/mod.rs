/// JSON-RPC Application Layer
///
/// Contains business logic and orchestration for JSON-RPC operations.
///
/// ## Components
/// - `service`: Method registry and request dispatcher
///
/// ## Responsibilities
/// - Register and manage RPC method handlers
/// - Dispatch requests to appropriate handlers
/// - Execute business logic
/// - Handle async operations
/// - Manage method lifecycle

pub mod service;

// Re-export commonly used types
pub use service::JsonRpcService;
