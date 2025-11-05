# WebSocket JSON-RPC Implementation

## Architecture Overview

This document describes the WebSocket JSON-RPC implementation following clean code principles and clean architecture patterns.

## Design Principles

### SOLID Principles

1. **Single Responsibility Principle (SRP)**
   - `JsonRpcRequest/Response/Error`: Only handle message structure
   - `JsonRpcService`: Only handle method registry and dispatching
   - `websocket_handler`: Only handle WebSocket connection lifecycle
   - Each component has one clear, well-defined purpose

2. **Open/Closed Principle (OCP)**
   - Open for extension: New JSON-RPC methods can be added without modifying existing code
   - Closed for modification: Core protocol handling doesn't need changes when adding methods
   - Method registration system allows dynamic extension

3. **Liskov Substitution Principle (LSP)**
   - All JSON-RPC messages implement proper serialization/deserialization
   - Error types properly implement the protocol specification

4. **Interface Segregation Principle (ISP)**
   - Clear separation between request, response, error, and notification types
   - Handlers only depend on the specific message types they need

5. **Dependency Inversion Principle (DIP)**
   - Handlers depend on the `JsonRpcService` abstraction
   - Business logic is injected via the service layer

### Clean Code Practices

1. **Meaningful Names**
   - `JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcErrorCode` - self-documenting
   - `handle_socket`, `process_message` - clear intent
   - `register_method`, `list_methods` - verb-noun naming

2. **Small Functions**
   - Each function does one thing well
   - `handle_socket` manages connection lifecycle
   - `process_message` handles message parsing and dispatching
   - Error creation separated into dedicated functions

3. **No Magic Numbers**
   - Error codes defined as named constants (`JsonRpcErrorCode` enum)
   - Version "2.0" defined in a single place per type

4. **Error Handling**
   - Comprehensive error handling at each layer
   - Proper JSON-RPC error codes for different scenarios
   - Graceful degradation and informative error messages

5. **Comments Where Needed**
   - Doc comments on public APIs
   - Architecture decisions documented
   - Complex logic explained

## Layer Architecture

### 1. Domain Layer (`models/jsonrpc.rs`)

**Purpose**: Define core business entities and protocol rules

**Components**:
- `JsonRpcRequest`: Represents a JSON-RPC 2.0 request
- `JsonRpcResponse`: Represents a successful response
- `JsonRpcErrorResponse`: Represents an error response
- `JsonRpcErrorCode`: Standard error codes
- `JsonRpcErrorObject`: Error details

**Key Features**:
- Full JSON-RPC 2.0 specification compliance
- Request validation (version, method names, reserved prefixes)
- Notification detection (requests without IDs)
- Type-safe error codes
- Comprehensive unit tests

**Clean Code:**
```rust
// Clear, self-documenting structure
pub struct JsonRpcRequest {
    pub jsonrpc: String,      // Version must be "2.0"
    pub method: String,        // Method to invoke
    pub params: Option<Value>, // Optional parameters
    pub id: Option<Value>,     // Optional ID (omit for notifications)
}

// Validation encapsulated in the model
impl JsonRpcRequest {
    pub fn validate(&self) -> Result<(), String> {
        // Business rules enforced here
    }
}
```

### 2. Application Layer (`services/jsonrpc_service.rs`)

**Purpose**: Business logic and orchestration

**Components**:
- `JsonRpcService`: Method registry and request dispatcher
- `MethodHandler`: Type alias for method handler functions
- Built-in methods: `ping`, `echo`, `add`, `getServerInfo`

**Key Features**:
- Thread-safe method registry using `Arc<RwLock<HashMap>>`
- Dynamic method registration
- Async method handlers
- Automatic notification handling (no response sent)
- Comprehensive error handling with proper error codes

**Clean Code:**
```rust
// Service manages business logic, not presentation
pub struct JsonRpcService {
    methods: Arc<RwLock<HashMap<String, MethodHandler>>>,
}

// Clear, focused public API
impl JsonRpcService {
    pub fn new() -> Self { ... }
    pub async fn register_method(...) { ... }
    pub async fn handle_request(...) -> Option<Result<...>> { ... }
}
```

**Design Decisions**:
1. **Arc<RwLock>** for thread-safe shared state
2. **Option<Result>** return type clearly indicates notifications vs errors
3. **BoxFuture** for type-erased async handlers
4. Built-in methods registered in constructor for immediate availability

### 3. Presentation Layer (`handlers/websocket.rs`)

**Purpose**: Handle WebSocket protocol and HTTP concerns

**Components**:
- `websocket_handler`: Upgrades HTTP to WebSocket
- `handle_socket`: Manages individual connection lifecycle
- `process_message`: Parses and routes JSON-RPC messages
- Error formatting functions

**Key Features**:
- WebSocket upgrade handling
- Message type routing (Text, Binary, Ping, Pong, Close)
- Proper connection cleanup
- Structured logging for debugging
- Graceful error handling

**Clean Code:**
```rust
// Clear separation: HTTP upgrade -> Socket handling -> Message processing
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(jsonrpc_service): State<JsonRpcService>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, jsonrpc_service))
}

async fn handle_socket(socket: WebSocket, jsonrpc_service: JsonRpcService) {
    // Connection lifecycle management only
}

async fn process_message(text: &str, service: &JsonRpcService) -> Option<String> {
    // Message parsing and routing only
}
```

## JSON-RPC 2.0 Specification Compliance

### Implemented Features

✅ Request/Response pattern
✅ Notification pattern (requests without IDs)
✅ Parameter passing (both named and positional)
✅ Standard error codes (-32700 to -32603)
✅ Error data field support
✅ Version string validation ("2.0")
✅ Reserved method prefix detection ("rpc.*")
✅ Null result handling
✅ ID matching in responses

### Standard Error Codes

| Code   | Name            | Description                                    |
|--------|-----------------|------------------------------------------------|
| -32700 | Parse error     | Invalid JSON was received                      |
| -32600 | Invalid Request | The JSON is not a valid Request object         |
| -32601 | Method not found| The method does not exist                      |
| -32602 | Invalid params  | Invalid method parameter(s)                    |
| -32603 | Internal error  | Internal JSON-RPC error                        |
| -32000 | Server error    | Implementation-defined server-errors           |

## Built-in Methods

### `ping`
- **Purpose**: Health check and connection testing
- **Parameters**: None
- **Returns**: `{ "pong": true, "timestamp": <unix_timestamp> }`
- **Use Case**: Verify server is responsive and measure latency

### `echo`
- **Purpose**: Parameter echo for testing
- **Parameters**: Any JSON value
- **Returns**: The same value that was sent
- **Use Case**: Test parameter serialization and round-trip

### `add`
- **Purpose**: Example method with validation
- **Parameters**: Array of two numbers `[a, b]`
- **Returns**: Sum of the two numbers
- **Use Case**: Demonstrate parameter validation and error handling

### `getServerInfo`
- **Purpose**: Server capability discovery
- **Parameters**: None
- **Returns**: Server name, version, JSON-RPC version, available methods
- **Use Case**: Client can discover what the server supports

## Extensibility

### Adding Custom Methods

The service is designed to be easily extended:

```rust
// In your initialization code
let jsonrpc_service = JsonRpcService::new();

jsonrpc_service.register_method("myMethod".to_string(), |params| async move {
    // 1. Validate parameters
    let params = params.ok_or_else(|| {
        JsonRpcErrorObject::new(JsonRpcErrorCode::InvalidParams, None)
    })?;

    // 2. Execute business logic
    let result = do_something(params).await?;

    // 3. Return result
    Ok(json!(result))
}).await;
```

### Method Handler Signature

```rust
async fn my_handler(params: Option<Value>) -> Result<Value, JsonRpcErrorObject>
```

**Parameters**: Optional `serde_json::Value` (can be object, array, or primitive)
**Returns**: `Result` with either:
- `Ok(Value)`: Success result (any JSON value)
- `Err(JsonRpcErrorObject)`: Error with code and message

### Error Handling Best Practices

```rust
// Use appropriate error codes
JsonRpcErrorCode::InvalidParams  // For bad parameters
JsonRpcErrorCode::InternalError  // For unexpected failures
JsonRpcErrorCode::ServerError    // For business logic errors

// Provide helpful error messages
JsonRpcErrorObject::custom(
    JsonRpcErrorCode::InvalidParams,
    "Expected array of two numbers".to_string(),
    None
)

// Include error data when helpful
JsonRpcErrorObject::new(
    JsonRpcErrorCode::InvalidParams,
    Some(json!({"expected": "array", "received": "object"}))
)
```

## Connection Lifecycle

```
Client                          Server
  |                               |
  |------- HTTP GET /live ------->|
  |<------ 101 Switching ---------|
  |                               |
  |====== WebSocket Connected ====|
  |                               |
  |------ JSON-RPC Request ------>|
  |                               |
  |         (processing)          |
  |                               |
  |<----- JSON-RPC Response ------|
  |                               |
  |------ Notification --------->|
  |                               |
  |      (no response)           |
  |                               |
  |------ Ping -------------->|
  |<----- Pong ----------------|
  |                               |
  |------ Close -------------->|
  |<----- Close ----------------|
  |                               |
  |====== Connection Closed ======|
```

## Testing Strategy

### Unit Tests

Located in each module:
- `models/jsonrpc.rs`: Protocol validation tests
- `services/jsonrpc_service.rs`: Method handling tests
- `handlers/websocket.rs`: Message processing tests

### Integration Tests

Two test clients provided:

1. **HTML Client** (`test_websocket_client.html`)
   - Visual testing interface
   - Pre-built method calls
   - Real-time message inspection
   - Error visualization

2. **Python Client** (`test_websocket_client.py`)
   - Automated test suite
   - Interactive mode for manual testing
   - All built-in methods tested
   - Error handling verification

### Test Coverage

Tests verify:
- ✅ Request validation
- ✅ Notification handling (no response)
- ✅ Error code generation
- ✅ Method registration
- ✅ Parameter validation
- ✅ Response serialization
- ✅ Connection lifecycle

## Performance Considerations

### Async All The Way
- Non-blocking I/O throughout
- Each connection handled in its own task
- Method handlers are async for I/O-heavy operations

### Resource Management
- Automatic connection cleanup on drop
- Bounded message queues to prevent memory issues
- Graceful shutdown support

### Scalability
- Thread-safe service can be cloned and shared
- No global mutable state
- Read-write locks for method registry (many readers, few writers)

## Security Considerations

### Input Validation
- JSON parsing errors caught and reported as Parse error (-32700)
- Request structure validated before processing
- Reserved method names rejected
- Parameter types validated in method handlers

### Error Information Disclosure
- Internal errors logged but not exposed to clients
- Generic error messages for security-sensitive failures
- Error data field only used for client-safe information

### Connection Management
- Proper WebSocket close handshake
- Timeout protection via tower middleware
- Connection limits via server configuration

## Future Enhancements

Potential improvements while maintaining clean architecture:

1. **Batched Requests**: Handle array of requests per JSON-RPC 2.0 spec
2. **Method Middleware**: Add hooks for logging, auth, rate limiting
3. **Streaming Results**: Support for server-sent events or chunked responses
4. **Method Documentation**: Auto-generate API docs from registered methods
5. **Metrics**: Track method call counts, latencies, error rates
6. **Authentication**: Add auth layer before method dispatch

## References

- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [WebSocket Protocol RFC 6455](https://tools.ietf.org/html/rfc6455)
- [Axum Documentation](https://docs.rs/axum/)
- [Clean Architecture by Robert C. Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
