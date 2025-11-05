use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 Error codes
///
/// Standard error codes as defined by the JSON-RPC 2.0 specification.
#[derive(Debug, Clone, Copy)]
pub enum JsonRpcErrorCode {
    /// Invalid JSON was received by the server
    ParseError = -32700,

    /// The JSON sent is not a valid Request object
    InvalidRequest = -32600,

    /// The method does not exist / is not available
    MethodNotFound = -32601,

    /// Invalid method parameter(s)
    InvalidParams = -32602,

    /// Internal JSON-RPC error
    InternalError = -32603,

    /// Server error (reserved for implementation-defined server-errors)
    ServerError = -32000,
}

impl JsonRpcErrorCode {
    pub fn code(&self) -> i32 {
        *self as i32
    }

    pub fn message(&self) -> &'static str {
        match self {
            JsonRpcErrorCode::ParseError => "Parse error",
            JsonRpcErrorCode::InvalidRequest => "Invalid Request",
            JsonRpcErrorCode::MethodNotFound => "Method not found",
            JsonRpcErrorCode::InvalidParams => "Invalid params",
            JsonRpcErrorCode::InternalError => "Internal error",
            JsonRpcErrorCode::ServerError => "Server error",
        }
    }
}

/// JSON-RPC 2.0 Error object
///
/// Represents an error that occurred during request processing.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcErrorObject {
    /// A Number that indicates the error type that occurred.
    pub code: i32,

    /// A String providing a short description of the error.
    pub message: String,

    /// A Primitive or Structured value that contains additional information about the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcErrorObject {
    /// Create a new error object
    pub fn new(code: JsonRpcErrorCode, data: Option<Value>) -> Self {
        Self {
            code: code.code(),
            message: code.message().to_string(),
            data,
        }
    }

    /// Create a custom error with a specific message
    pub fn custom(code: JsonRpcErrorCode, message: String, data: Option<Value>) -> Self {
        Self {
            code: code.code(),
            message,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(JsonRpcErrorCode::ParseError.code(), -32700);
        assert_eq!(JsonRpcErrorCode::InvalidRequest.code(), -32600);
        assert_eq!(JsonRpcErrorCode::MethodNotFound.code(), -32601);
        assert_eq!(JsonRpcErrorCode::InvalidParams.code(), -32602);
        assert_eq!(JsonRpcErrorCode::InternalError.code(), -32603);
        assert_eq!(JsonRpcErrorCode::ServerError.code(), -32000);
    }

    #[test]
    fn test_error_messages() {
        assert_eq!(JsonRpcErrorCode::ParseError.message(), "Parse error");
        assert_eq!(JsonRpcErrorCode::MethodNotFound.message(), "Method not found");
    }
}
