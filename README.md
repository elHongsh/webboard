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

- **axum**: Web application framework
- **tokio**: Async runtime
- **tower**: Service middleware
- **tower-http**: HTTP-specific middleware
- **serde**: Serialization/deserialization
- **tracing**: Structured logging
- **anyhow**: Error handling utilities
- **thiserror**: Error trait derivation

## License

This project is provided as a clean code example for Axum web servers.
