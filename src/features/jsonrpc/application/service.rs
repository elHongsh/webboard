use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::super::domain::{
    JsonRpcErrorCode, JsonRpcErrorObject, JsonRpcErrorResponse, JsonRpcRequest, JsonRpcResponse,
};

/// Type alias for JSON-RPC method handlers
///
/// A method handler is an async function that takes optional parameters
/// and returns a Result with either a JSON value or an error object.
type MethodHandler = Arc<
    dyn Fn(Option<Value>) -> futures::future::BoxFuture<'static, Result<Value, JsonRpcErrorObject>>
        + Send
        + Sync,
>;

/// JSON-RPC Service
///
/// Application layer service that manages method registration and dispatching.
/// Follows the Single Responsibility Principle by only handling RPC logic.
///
/// ## Responsibilities
/// - Register and manage method handlers
/// - Dispatch requests to appropriate handlers
/// - Handle notifications (no response)
/// - Validate requests
/// - Generate appropriate error responses
#[derive(Clone)]
pub struct JsonRpcService {
    /// Registry of available methods
    methods: Arc<RwLock<HashMap<String, MethodHandler>>>,
}

impl JsonRpcService {
    /// Create a new JSON-RPC service with built-in methods
    pub fn new() -> Self {
        let service = Self {
            methods: Arc::new(RwLock::new(HashMap::new())),
        };

        // Register built-in methods
        service.register_builtin_methods();

        service
    }

    /// Register a new method handler
    ///
    /// # Arguments
    /// * `name` - The method name
    /// * `handler` - The async function to handle this method
    pub async fn register_method<F, Fut>(&self, name: String, handler: F)
    where
        F: Fn(Option<Value>) -> Fut + Send + Sync + 'static,
        Fut: futures::future::Future<Output = Result<Value, JsonRpcErrorObject>> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |params: Option<Value>| {
            let fut = handler(params);
            Box::pin(fut) as futures::future::BoxFuture<'static, Result<Value, JsonRpcErrorObject>>
        });

        let mut methods = self.methods.write().await;
        methods.insert(name, wrapped_handler);
    }

    /// Process a JSON-RPC request
    ///
    /// # Arguments
    /// * `request` - The JSON-RPC request to process
    ///
    /// # Returns
    /// * `Some(response)` - For requests that expect a response
    /// * `None` - For notifications (no response needed)
    pub async fn handle_request(
        &self,
        request: JsonRpcRequest,
    ) -> Option<Result<JsonRpcResponse, JsonRpcErrorResponse>> {
        // Validate the request
        if let Err(e) = request.validate() {
            let error_response = JsonRpcErrorResponse::custom(
                JsonRpcErrorCode::InvalidRequest,
                e,
                request.id.clone().unwrap_or(Value::Null),
            );
            return Some(Err(error_response));
        }

        // If it's a notification, don't send a response
        if request.is_notification() {
            // Still process it, but don't return a response
            let methods = self.methods.read().await;
            if let Some(handler) = methods.get(&request.method) {
                let _ = handler(request.params).await;
            }
            return None;
        }

        let id = request.id.clone().unwrap_or(Value::Null);

        // Look up the method
        let methods = self.methods.read().await;
        let handler = match methods.get(&request.method) {
            Some(h) => h.clone(),
            None => {
                let error_response = JsonRpcErrorResponse::custom(
                    JsonRpcErrorCode::MethodNotFound,
                    format!("Method '{}' not found", request.method),
                    id,
                );
                return Some(Err(error_response));
            }
        };

        // Release the read lock before calling the handler
        drop(methods);

        // Execute the method handler
        match handler(request.params).await {
            Ok(result) => Some(Ok(JsonRpcResponse::new(result, id))),
            Err(error) => Some(Err(JsonRpcErrorResponse::new(error, id))),
        }
    }

    /// Register built-in methods that are always available
    fn register_builtin_methods(&self) {
        let service = self.clone();

        // Echo method - returns the parameters sent
        tokio::spawn(async move {
            service
                .register_method("echo".to_string(), |params| async move {
                    Ok(params.unwrap_or(Value::Null))
                })
                .await;
        });

        let service = self.clone();
        // Ping method - simple health check
        tokio::spawn(async move {
            service
                .register_method("ping".to_string(), |_params| async move {
                    Ok(json!({"pong": true, "timestamp": chrono::Utc::now().timestamp()}))
                })
                .await;
        });

        let service = self.clone();
        // Add method - adds two numbers
        tokio::spawn(async move {
            service
                .register_method("add".to_string(), |params| async move {
                    let params = params.ok_or_else(|| {
                        JsonRpcErrorObject::custom(
                            JsonRpcErrorCode::InvalidParams,
                            "Parameters required".to_string(),
                            None,
                        )
                    })?;

                    let numbers = params.as_array().ok_or_else(|| {
                        JsonRpcErrorObject::custom(
                            JsonRpcErrorCode::InvalidParams,
                            "Parameters must be an array of numbers".to_string(),
                            None,
                        )
                    })?;

                    if numbers.len() != 2 {
                        return Err(JsonRpcErrorObject::custom(
                            JsonRpcErrorCode::InvalidParams,
                            "Exactly two numbers required".to_string(),
                            None,
                        ));
                    }

                    let a = numbers[0].as_f64().ok_or_else(|| {
                        JsonRpcErrorObject::custom(
                            JsonRpcErrorCode::InvalidParams,
                            "First parameter must be a number".to_string(),
                            None,
                        )
                    })?;

                    let b = numbers[1].as_f64().ok_or_else(|| {
                        JsonRpcErrorObject::custom(
                            JsonRpcErrorCode::InvalidParams,
                            "Second parameter must be a number".to_string(),
                            None,
                        )
                    })?;

                    Ok(json!(a + b))
                })
                .await;
        });

        let service = self.clone();
        // Server info method - returns information about the server
        tokio::spawn(async move {
            service
                .register_method("getServerInfo".to_string(), |_params| async move {
                    Ok(json!({
                        "name": "webboard",
                        "version": env!("CARGO_PKG_VERSION"),
                        "jsonrpc_version": "2.0",
                        "capabilities": ["echo", "ping", "add", "getServerInfo"]
                    }))
                })
                .await;
        });
    }

    /// Get the list of registered methods
    pub async fn list_methods(&self) -> Vec<String> {
        let methods = self.methods.read().await;
        methods.keys().cloned().collect()
    }
}

impl Default for JsonRpcService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_echo_method() {
        let service = JsonRpcService::new();

        // Give some time for builtin methods to register
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let request = JsonRpcRequest::new(
            "echo".to_string(),
            Some(json!({"message": "hello"})),
            Some(json!(1)),
        );

        let response = service.handle_request(request).await;
        assert!(response.is_some());

        if let Some(Ok(resp)) = response {
            assert_eq!(resp.result, json!({"message": "hello"}));
        }
    }

    #[tokio::test]
    async fn test_method_not_found() {
        let service = JsonRpcService::new();

        let request = JsonRpcRequest::new(
            "nonexistent_method".to_string(),
            None,
            Some(json!(1)),
        );

        let response = service.handle_request(request).await;
        assert!(response.is_some());

        if let Some(Err(err)) = response {
            assert_eq!(err.error.code, JsonRpcErrorCode::MethodNotFound.code());
        }
    }

    #[tokio::test]
    async fn test_notification_no_response() {
        let service = JsonRpcService::new();

        let notification = JsonRpcRequest::new(
            "echo".to_string(),
            Some(json!({"message": "notify"})),
            None, // No ID = notification
        );

        let response = service.handle_request(notification).await;
        assert!(response.is_none());
    }
}
