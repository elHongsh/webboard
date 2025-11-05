use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 Request
///
/// A remote procedure call is represented by sending a Request object to a Server.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    /// A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
    pub jsonrpc: String,

    /// A String containing the name of the method to be invoked.
    pub method: String,

    /// A Structured value that holds the parameter values to be used during the invocation.
    /// This member MAY be omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,

    /// An identifier established by the Client. If not included, it is assumed to be a notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(method: String, params: Option<Value>, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id,
        }
    }

    /// Check if this request is a notification (no id field)
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }

    /// Validate the request structure
    pub fn validate(&self) -> Result<(), String> {
        if self.jsonrpc != "2.0" {
            return Err("Invalid JSON-RPC version. Must be '2.0'".to_string());
        }

        if self.method.is_empty() {
            return Err("Method name cannot be empty".to_string());
        }

        if self.method.starts_with("rpc.") {
            return Err("Method names starting with 'rpc.' are reserved".to_string());
        }

        Ok(())
    }
}

/// JSON-RPC 2.0 Response (Success)
///
/// When a remote procedure call completes successfully, the Server sends a Response object.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcResponse {
    /// A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
    pub jsonrpc: String,

    /// The result of the method invocation. Required on success.
    pub result: Value,

    /// The request id. Must match the request id.
    pub id: Value,
}

impl JsonRpcResponse {
    /// Create a new successful JSON-RPC response
    pub fn new(result: Value, id: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result,
            id,
        }
    }
}

/// JSON-RPC 2.0 Error codes
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

/// JSON-RPC 2.0 Error Response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcErrorResponse {
    /// A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
    pub jsonrpc: String,

    /// The error object.
    pub error: JsonRpcErrorObject,

    /// The request id. Must match the request id or be null if id couldn't be determined.
    pub id: Value,
}

impl JsonRpcErrorResponse {
    /// Create a new error response
    pub fn new(error: JsonRpcErrorObject, id: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            error,
            id,
        }
    }

    /// Create an error response from an error code
    pub fn from_code(code: JsonRpcErrorCode, id: Value) -> Self {
        Self::new(JsonRpcErrorObject::new(code, None), id)
    }

    /// Create an error response with custom message
    pub fn custom(code: JsonRpcErrorCode, message: String, id: Value) -> Self {
        Self::new(JsonRpcErrorObject::custom(code, message, None), id)
    }
}

/// Enum representing either a success or error response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Error(JsonRpcErrorResponse),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_validation() {
        let valid_req = JsonRpcRequest::new(
            "test.method".to_string(),
            Some(json!({"param": "value"})),
            Some(json!(1)),
        );
        assert!(valid_req.validate().is_ok());

        let invalid_version = JsonRpcRequest {
            jsonrpc: "1.0".to_string(),
            method: "test".to_string(),
            params: None,
            id: Some(json!(1)),
        };
        assert!(invalid_version.validate().is_err());

        let reserved_method = JsonRpcRequest::new(
            "rpc.reserved".to_string(),
            None,
            Some(json!(1)),
        );
        assert!(reserved_method.validate().is_err());
    }

    #[test]
    fn test_notification() {
        let notification = JsonRpcRequest::new(
            "notify".to_string(),
            Some(json!({"message": "hello"})),
            None,
        );
        assert!(notification.is_notification());

        let request = JsonRpcRequest::new(
            "call".to_string(),
            None,
            Some(json!(1)),
        );
        assert!(!request.is_notification());
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(JsonRpcErrorCode::ParseError.code(), -32700);
        assert_eq!(JsonRpcErrorCode::MethodNotFound.code(), -32601);
    }
}
