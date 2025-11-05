/// JSON-RPC Domain Layer
///
/// Contains the core business entities and protocol definitions for JSON-RPC 2.0.
/// This layer has no dependencies on other layers and defines the protocol rules.
///
/// ## Components
/// - `message`: Request, Response, and Error message types
/// - `error_code`: Standard JSON-RPC error codes and error objects
///
/// ## Responsibilities
/// - Define the JSON-RPC 2.0 protocol structure
/// - Validate message format and structure
/// - Enforce protocol rules (version, reserved names, etc.)

pub mod error_code;
pub mod message;

// Re-export commonly used types
pub use error_code::{JsonRpcErrorCode, JsonRpcErrorObject};
pub use message::{JsonRpcErrorResponse, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse};
