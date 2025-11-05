# Architecture Documentation

## Overview

This project implements a **Modular Clean Architecture** where code is organized by both **vertical slices (features)** and **horizontal slices (layers)**. This approach combines the best of clean architecture principles with feature-driven development.

## Core Principles

### 1. Separation by Feature

Each business capability is encapsulated in its own feature module:
- **health**: Health check functionality
- **users**: User management
- **jsonrpc**: WebSocket JSON-RPC protocol

### 2. Separation by Layer

Within each feature, code is organized into architectural layers:
- **Domain**: Business entities, value objects, business rules
- **Application**: Use cases, business logic orchestration
- **Presentation**: HTTP/WebSocket handlers, request/response mapping

### 3. Infrastructure as Foundation

Cross-cutting concerns are centralized in the infrastructure layer:
- Configuration management
- Error handling
- Logging
- Common utilities

## Directory Structure

```
src/
├── main.rs                    # Application entry point
├── infrastructure/            # Infrastructure layer
└── features/                  # Feature modules
    ├── health/
    ├── users/
    └── jsonrpc/
```

## Layer Details

### Infrastructure Layer

**Location**: `src/infrastructure/`

**Purpose**: Provides foundational services used across all features.

**Components**:
- `config.rs`: Application configuration from environment variables
- `error.rs`: Application-wide error types with HTTP mapping

**Dependencies**: None (depends only on external crates)

**Used By**: All features

```rust
// Example usage
use infrastructure::{AppConfig, AppError};
```

### Feature Modules

Each feature is self-contained with its own layers.

#### Health Feature

**Location**: `src/features/health/`

**Purpose**: Provides a simple health check endpoint.

**Layers**:
- **Domain** (`domain.rs`): `HealthResponse` model
- **Presentation** (`handler.rs`): HTTP handler

**Why no Application layer?** The health check is simple enough that it doesn't require business logic orchestration.

```
health/
├── mod.rs          # Module definition and exports
├── domain.rs       # HealthResponse model
└── handler.rs      # health_check handler
```

#### Users Feature

**Location**: `src/features/users/`

**Purpose**: Manages user-related operations.

**Layers**:
- **Domain** (`domain.rs`): `User`, `CreateUserRequest` with validation
- **Application** (`service.rs`): `UserService` with business logic
- **Presentation** (`handler.rs`): HTTP handlers (list, create, get)

**Flow**:
```
HTTP Request → Handler → Service → Domain Validation → Response
```

```
users/
├── mod.rs          # Module definition and exports
├── domain.rs       # User, CreateUserRequest
├── service.rs      # UserService (business logic)
└── handler.rs      # list_users, create_user, get_user
```

#### JSON-RPC Feature

**Location**: `src/features/jsonrpc/`

**Purpose**: Implements WebSocket-based JSON-RPC 2.0 protocol.

**Layers**:
- **Domain** (`domain/`): Protocol messages and error codes
- **Application** (`application/`): Method registry and dispatcher
- **Presentation** (`presentation/`): WebSocket handler

**Complexity**: This is the most complex feature, hence the deeper layer structure.

```
jsonrpc/
├── mod.rs
├── domain/
│   ├── mod.rs
│   ├── message.rs      # Request, Response, Error types
│   └── error_code.rs   # JSON-RPC error codes
├── application/
│   ├── mod.rs
│   └── service.rs      # JsonRpcService
└── presentation/
    ├── mod.rs
    └── handler.rs      # WebSocket handler
```

## Dependency Rules

### The Dependency Rule

Dependencies point **inward** toward the domain:

```
Presentation → Application → Domain
         ↓
   Infrastructure
```

- **Domain** depends on nothing (pure business logic)
- **Application** depends on Domain
- **Presentation** depends on Application and Domain
- **Infrastructure** is used by all layers but depends on none

### Example

```rust
// ✅ GOOD: Presentation depends on Application
use super::service::UserService;

// ✅ GOOD: Application depends on Domain
use super::domain::{User, CreateUserRequest};

// ✅ GOOD: Both use Infrastructure
use crate::infrastructure::AppError;

// ❌ BAD: Domain should NOT depend on Application
// Domain should be pure business logic
```

## Module Organization Pattern

Each module follows this pattern:

```rust
// mod.rs
/// Feature documentation
///
/// ## Architecture
/// - Domain: ...
/// - Application: ...
/// - Presentation: ...
///
/// ## Usage
/// ```rust
/// // Example usage
/// ```

pub mod domain;
pub mod service;     // If has application layer
pub mod handler;

// Re-export commonly used items
pub use domain::*;
pub use service::*;
pub use handler::*;
```

## Communication Between Features

Features should be **loosely coupled**. Communication happens through:

1. **Shared Infrastructure**: Common error types, configuration
2. **Main Router**: Features are composed in `main.rs`
3. **Direct Service Calls**: If needed, features can depend on other feature services

```rust
// In main.rs - composing features
let app = Router::new()
    .route("/health", get(features::health_check))
    .route("/live", get(features::websocket_handler))
    .nest("/api/v1/users", users_routes);
```

## Adding a New Feature

To add a new feature, follow these steps:

### 1. Create Feature Directory

```bash
mkdir -p src/features/my_feature
```

### 2. Create Layer Files

Depending on complexity:

**Simple Feature** (like health):
```bash
touch src/features/my_feature/mod.rs
touch src/features/my_feature/domain.rs
touch src/features/my_feature/handler.rs
```

**Complex Feature** (like users):
```bash
touch src/features/my_feature/mod.rs
touch src/features/my_feature/domain.rs
touch src/features/my_feature/service.rs
touch src/features/my_feature/handler.rs
```

**Very Complex Feature** (like jsonrpc):
```bash
mkdir -p src/features/my_feature/{domain,application,presentation}
# Create appropriate files in each subdirectory
```

### 3. Implement Layers

**Domain Layer** (`domain.rs`):
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct MyEntity {
    pub id: u64,
    pub name: String,
}
```

**Application Layer** (`service.rs`):
```rust
use crate::infrastructure::AppError;
use super::domain::MyEntity;

#[derive(Clone)]
pub struct MyService {
    // state
}

impl MyService {
    pub fn new() -> Self {
        Self { }
    }

    pub async fn do_something(&self) -> Result<MyEntity, AppError> {
        // business logic
    }
}
```

**Presentation Layer** (`handler.rs`):
```rust
use axum::{extract::State, Json};
use crate::infrastructure::AppError;
use super::{domain::MyEntity, service::MyService};

pub async fn my_handler(
    State(service): State<MyService>,
) -> Result<Json<MyEntity>, AppError> {
    let entity = service.do_something().await?;
    Ok(Json(entity))
}
```

### 4. Register Feature

**Add to `features/mod.rs`**:
```rust
pub mod my_feature;

pub use my_feature::{my_handler, MyEntity, MyService};
```

**Add routes in `main.rs`**:
```rust
let service = features::MyService::new();

let app = Router::new()
    .route("/my-endpoint", get(features::my_handler))
    .with_state(service);
```

## Testing Strategy

### Unit Tests

Each layer should have its own unit tests:

```rust
// In domain.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_logic() {
        // Test business rules
    }
}

// In service.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_logic() {
        // Test use cases
    }
}
```

### Integration Tests

Test features end-to-end in `tests/` directory:

```rust
// tests/health_test.rs
use axum::http::StatusCode;

#[tokio::test]
async fn test_health_endpoint() {
    // Test full feature
}
```

## Benefits of This Architecture

### 1. **Scalability**
- New features don't affect existing ones
- Teams can work on different features independently

### 2. **Maintainability**
- Code location is predictable
- Related code is grouped together
- Easy to understand feature boundaries

### 3. **Testability**
- Each layer can be tested in isolation
- Clear dependencies make mocking easy
- Feature-specific tests are co-located

### 4. **Flexibility**
- Features can have different layer complexity
- Easy to refactor individual features
- Can extract features to microservices later

### 5. **Onboarding**
- New developers can understand one feature at a time
- Clear structure reduces cognitive load
- Self-documenting through organization

## Comparison with Traditional Layered Architecture

### Traditional Layered
```
src/
├── models/        # All domain models
├── services/      # All services
└── handlers/      # All handlers
```

**Problems**:
- Low cohesion (related code scattered)
- High coupling (changes affect many files)
- Hard to find feature-specific code

### Modular Clean Architecture
```
src/
└── features/
    ├── users/     # Everything for users
    └── posts/     # Everything for posts
```

**Benefits**:
- High cohesion (related code together)
- Low coupling (features independent)
- Easy to navigate and understand

## References

- [Clean Architecture by Robert C. Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Vertical Slice Architecture](https://jimmybogard.com/vertical-slice-architecture/)
- [Package by Feature, Not Layer](https://phauer.com/2020/package-by-feature/)
