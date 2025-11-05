# Webboard - Clean Axum Web Server

A production-ready web server built with Axum following clean code principles and web server best practices.

## Architecture

### Clean Architecture Layers

```
┌─────────────────────────────────────┐
│      Presentation Layer             │
│      (handlers/)                    │
│  HTTP request/response handling     │
├─────────────────────────────────────┤
│      Application Layer              │
│      (services/)                    │
│  Business logic & orchestration     │
├─────────────────────────────────────┤
│      Domain Layer                   │
│      (models/)                      │
│  Core business entities & rules     │
├─────────────────────────────────────┤
│      Infrastructure Layer           │
│   (config.rs, error.rs, main.rs)   │
│  Configuration, logging, errors     │
└─────────────────────────────────────┘
```

### Project Structure

```
src/
├── main.rs              # Entry point, server setup, middleware
├── config.rs            # Environment-based configuration
├── error.rs             # Custom error types with IntoResponse
├── handlers/            # HTTP handlers (presentation layer)
│   └── mod.rs           # Health check, CRUD endpoints
├── services/            # Business logic (application layer)
│   └── mod.rs           # UserService with validation
└── models/              # Domain models
    └── mod.rs           # User, request/response types
```

## Features

### Core Capabilities
- RESTful API with clean routing structure
- **WebSocket with JSON-RPC 2.0 protocol** at `/live` endpoint
- Environment-based configuration
- Structured logging with tracing
- Graceful shutdown handling
- Request timeout protection
- Request body size limits
- CORS support

### Best Practices Implemented

**SOLID Principles:**
- Single Responsibility: Each module has one clear purpose
- Open/Closed: Extensible via middleware and modular routing
- Liskov Substitution: Error types properly implement IntoResponse
- Interface Segregation: Handlers depend only on needed extractors
- Dependency Inversion: Services injected via State

**Clean Code:**
- Descriptive naming conventions
- Small, focused functions
- Clear separation of concerns
- Type-driven design
- Comprehensive error handling

**Web Server Best Practices:**
- Structured error responses
- Input validation
- Security middleware (CORS, body limits, timeouts)
- Request/response logging
- Health check endpoint

## API Endpoints

### Health Check
```
GET /health
Response: {"status": "healthy", "version": "0.1.0"}
```

### WebSocket JSON-RPC Endpoint
```
WebSocket: ws://127.0.0.1:3000/live
Protocol: JSON-RPC 2.0
```

The `/live` endpoint provides a WebSocket connection that uses the JSON-RPC 2.0 protocol for real-time bidirectional communication.

### Users API

**List Users**
```
GET /api/v1/users?limit=10
Response: [{"id": 1, "username": "user1", "email": "user1@example.com"}, ...]
```

**Create User**
```
POST /api/v1/users
Content-Type: application/json
Body: {"username": "john", "email": "john@example.com"}
Response: {"id": 1, "username": "john", "email": "john@example.com"}
```

**Get User by ID**
```
GET /api/v1/users/{id}
Response: {"id": 5, "username": "user5", "email": "user5@example.com"}
```

### Error Responses

All errors return JSON with consistent structure:
```json
{
  "error": "ERROR_TYPE",
  "message": "Human-readable error message"
}
```

Error types:
- `NOT_FOUND` (404): Resource not found
- `BAD_REQUEST` (400): Invalid input or validation error
- `INTERNAL_SERVER_ERROR` (500): Server-side error

## WebSocket JSON-RPC API

### Overview

The WebSocket endpoint at `/live` implements the JSON-RPC 2.0 specification, providing a standardized protocol for real-time communication.

### JSON-RPC 2.0 Protocol

**Request Format:**
```json
{
  "jsonrpc": "2.0",
  "method": "method_name",
  "params": {...},
  "id": 1
}
```

**Success Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {...},
  "id": 1
}
```

**Error Response:**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32601,
    "message": "Method not found"
  },
  "id": 1
}
```

**Notification (no response expected):**
```json
{
  "jsonrpc": "2.0",
  "method": "method_name",
  "params": {...}
}
```
*Note: Notifications omit the `id` field and don't receive a response.*

### Built-in JSON-RPC Methods

#### `ping`
Health check method that returns server timestamp.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "ping",
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "pong": true,
    "timestamp": 1699564800
  },
  "id": 1
}
```

#### `echo`
Echoes back the parameters sent.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "echo",
  "params": {"message": "Hello, World!"},
  "id": 2
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {"message": "Hello, World!"},
  "id": 2
}
```

#### `add`
Adds two numbers together.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "add",
  "params": [5, 3],
  "id": 3
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": 8,
  "id": 3
}
```

#### `getServerInfo`
Returns information about the server and its capabilities.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "getServerInfo",
  "id": 4
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "name": "webboard",
    "version": "0.1.0",
    "jsonrpc_version": "2.0",
    "capabilities": ["echo", "ping", "add", "getServerInfo"]
  },
  "id": 4
}
```

### JSON-RPC Error Codes

Standard JSON-RPC 2.0 error codes:

| Code   | Message           | Meaning                                    |
|--------|-------------------|--------------------------------------------|
| -32700 | Parse error       | Invalid JSON received                      |
| -32600 | Invalid Request   | JSON is not a valid Request object         |
| -32601 | Method not found  | The method does not exist                  |
| -32602 | Invalid params    | Invalid method parameters                  |
| -32603 | Internal error    | Internal JSON-RPC error                    |
| -32000 | Server error      | Implementation-defined server error        |

### Testing the WebSocket API

#### Using the HTML Test Client

Open `test_websocket_client.html` in your browser:

```bash
# Start the server
cargo run

# Open the HTML file in your browser
open test_websocket_client.html  # macOS
# or
xdg-open test_websocket_client.html  # Linux
# or simply drag and drop it into your browser
```

The HTML client provides:
- Connection management
- Pre-built method calls (ping, echo, add, getServerInfo)
- Custom request builder
- Real-time message log
- Error handling

#### Using the Python Test Client

```bash
# Install dependencies
pip install websockets

# Run automated tests
python test_websocket_client.py

# Run in interactive mode
python test_websocket_client.py --interactive
```

#### Using wscat (Command Line)

```bash
# Install wscat
npm install -g wscat

# Connect to the WebSocket
wscat -c ws://127.0.0.1:3000/live

# Send a ping request
> {"jsonrpc":"2.0","method":"ping","id":1}

# Send an echo request
> {"jsonrpc":"2.0","method":"echo","params":{"message":"Hello"},"id":2}

# Send an add request
> {"jsonrpc":"2.0","method":"add","params":[10,20],"id":3}
```

#### Using JavaScript (Browser Console or Node.js)

```javascript
const ws = new WebSocket('ws://127.0.0.1:3000/live');

ws.onopen = () => {
    console.log('Connected');

    // Send a ping request
    ws.send(JSON.stringify({
        jsonrpc: '2.0',
        method: 'ping',
        id: 1
    }));
};

ws.onmessage = (event) => {
    console.log('Received:', JSON.parse(event.data));
};

ws.onerror = (error) => {
    console.error('WebSocket error:', error);
};
```

### Extending with Custom Methods

The JSON-RPC service can be easily extended with custom methods. Example:

```rust
// In main.rs or a service file
jsonrpc_service.register_method("custom_method".to_string(), |params| async move {
    // Your business logic here
    let result = process_params(params)?;
    Ok(json!(result))
}).await;
```

The architecture follows clean code principles:
- **Single Responsibility**: Each component has one clear purpose
- **Open/Closed**: Easy to add new methods without modifying existing code
- **Type Safety**: Strongly typed messages and error handling
- **Separation of Concerns**: Clear layers (models, services, handlers)

## Configuration

Create a `.env` file (see `.env.example`):

```env
HOST=127.0.0.1
PORT=3000
LOG_LEVEL=info
REQUEST_TIMEOUT_SECS=30
MAX_BODY_SIZE=2097152
```

## Running the Server

```bash
# Development
cargo run

# Production build
cargo build --release
./target/release/webboard
```

The server will start on `http://127.0.0.1:3000` by default.

## Testing

```bash
# Health check
curl http://127.0.0.1:3000/health

# List users
curl http://127.0.0.1:3000/api/v1/users?limit=3

# Create user
curl -X POST http://127.0.0.1:3000/api/v1/users \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com"}'

# Get user by ID
curl http://127.0.0.1:3000/api/v1/users/1
```

## Middleware Stack

The application uses the following middleware layers (executed in order):

1. **TraceLayer**: Request/response logging
2. **CorsLayer**: Cross-origin resource sharing
3. **TimeoutLayer**: Request timeout protection (30s default)
4. **DefaultBodyLimit**: Request body size limit (2MB default)

## Graceful Shutdown

The server handles shutdown signals gracefully:
- CTRL+C (SIGINT)
- SIGTERM (Unix systems)

In-flight requests are allowed to complete before shutdown.

## Dependencies

- **axum**: Web application framework (with WebSocket support)
- **tokio**: Async runtime
- **tower**: Service middleware
- **tower-http**: HTTP-specific middleware
- **serde**: Serialization/deserialization
- **serde_json**: JSON serialization
- **tracing**: Structured logging
- **anyhow**: Error handling utilities
- **thiserror**: Error trait derivation
- **futures**: Async utilities for WebSocket handling
- **chrono**: Date/time utilities for timestamps

## License

This project is provided as a clean code example for Axum web servers.
