use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde_json::Value;

use super::super::application::JsonRpcService;
use super::super::domain::{JsonRpcErrorCode, JsonRpcErrorResponse, JsonRpcRequest};

/// WebSocket handler for the /live endpoint
///
/// Presentation layer handler that upgrades HTTP to WebSocket and
/// processes JSON-RPC messages.
///
/// # Route
/// WebSocket: ws://127.0.0.1:3000/live
///
/// # Protocol
/// JSON-RPC 2.0 over WebSocket
///
/// # Example
/// ```json
/// // Request
/// {"jsonrpc":"2.0","method":"ping","id":1}
///
/// // Response
/// {"jsonrpc":"2.0","result":{"pong":true,"timestamp":1699564800},"id":1}
/// ```
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(jsonrpc_service): State<JsonRpcService>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, jsonrpc_service))
}

/// Handle an individual WebSocket connection
///
/// Processes incoming JSON-RPC messages and sends responses back.
/// Each connection is handled independently with its own task.
async fn handle_socket(socket: WebSocket, jsonrpc_service: JsonRpcService) {
    let (mut sender, mut receiver) = socket.split();

    tracing::info!("New WebSocket connection established");

    // Process incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                tracing::debug!("Received message: {}", text);

                // Process the JSON-RPC request
                match process_message(&text, &jsonrpc_service).await {
                    Some(response) => {
                        // Send response back to client
                        if let Err(e) = sender.send(Message::Text(response)).await {
                            tracing::error!("Failed to send response: {}", e);
                            break;
                        }
                    }
                    None => {
                        // No response needed (notification)
                        tracing::debug!("Processed notification, no response sent");
                    }
                }
            }
            Ok(Message::Binary(_)) => {
                tracing::warn!("Binary messages not supported, closing connection");
                let error = create_parse_error("Binary messages not supported".to_string());
                let _ = sender.send(Message::Text(error)).await;
                break;
            }
            Ok(Message::Ping(data)) => {
                // Respond to ping with pong
                if let Err(e) = sender.send(Message::Pong(data)).await {
                    tracing::error!("Failed to send pong: {}", e);
                    break;
                }
            }
            Ok(Message::Pong(_)) => {
                // Pong received, connection is alive
                tracing::debug!("Pong received");
            }
            Ok(Message::Close(_)) => {
                tracing::info!("Client closed connection");
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    tracing::info!("WebSocket connection closed");
}

/// Process a JSON-RPC message
///
/// # Arguments
/// * `text` - The raw JSON text from the client
/// * `jsonrpc_service` - The JSON-RPC service to handle the request
///
/// # Returns
/// * `Some(String)` - A JSON response to send back to the client
/// * `None` - For notifications that don't require a response
async fn process_message(text: &str, jsonrpc_service: &JsonRpcService) -> Option<String> {
    // Parse the JSON-RPC request
    let request: JsonRpcRequest = match serde_json::from_str(text) {
        Ok(req) => req,
        Err(e) => {
            tracing::warn!("Failed to parse JSON-RPC request: {}", e);
            let error = create_parse_error(format!("Invalid JSON: {}", e));
            return Some(error);
        }
    };

    // Handle the request
    let response = jsonrpc_service.handle_request(request).await;

    // Convert response to JSON string
    response.map(|result| match result {
        Ok(success) => serde_json::to_string(&success).unwrap_or_else(|e| {
            tracing::error!("Failed to serialize success response: {}", e);
            create_internal_error()
        }),
        Err(error) => serde_json::to_string(&error).unwrap_or_else(|e| {
            tracing::error!("Failed to serialize error response: {}", e);
            create_internal_error()
        }),
    })
}

/// Create a parse error response
fn create_parse_error(message: String) -> String {
    let error = JsonRpcErrorResponse::custom(
        JsonRpcErrorCode::ParseError,
        message,
        Value::Null,
    );
    serde_json::to_string(&error).unwrap_or_else(|_| {
        r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":null}"#.to_string()
    })
}

/// Create an internal error response
fn create_internal_error() -> String {
    let error = JsonRpcErrorResponse::from_code(JsonRpcErrorCode::InternalError, Value::Null);
    serde_json::to_string(&error).unwrap_or_else(|_| {
        r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"},"id":null}"#
            .to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_process_valid_request() {
        let service = JsonRpcService::new();

        // Give time for builtin methods to register
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let request = r#"{"jsonrpc":"2.0","method":"echo","params":{"test":"value"},"id":1}"#;

        let response = process_message(request, &service).await;
        assert!(response.is_some());

        if let Some(resp) = response {
            assert!(resp.contains("test"));
            assert!(resp.contains("value"));
        }
    }

    #[tokio::test]
    async fn test_process_invalid_json() {
        let service = JsonRpcService::new();

        let request = r#"{"invalid json"#;

        let response = process_message(request, &service).await;
        assert!(response.is_some());

        if let Some(resp) = response {
            assert!(resp.contains("Parse error") || resp.contains("-32700"));
        }
    }

    #[tokio::test]
    async fn test_process_notification() {
        let service = JsonRpcService::new();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Notification has no id
        let request = r#"{"jsonrpc":"2.0","method":"echo","params":{"test":"value"}}"#;

        let response = process_message(request, &service).await;
        // Notifications should not return a response
        assert!(response.is_none());
    }
}
