/// Features Module
///
/// Contains all feature modules organized by business capability.
/// Each feature is self-contained with its own layers (domain, application, presentation).
///
/// ## Organization
///
/// Features are organized following clean architecture principles:
/// - Vertical slicing by feature (health, users, jsonrpc)
/// - Horizontal slicing by layer (domain, application, presentation)
///
/// ## Available Features
///
/// ### Auth (`auth/`)
/// Authentication and authorization for verified and anonymous users.
/// - Layers: domain, application (service), middleware
///
/// ### Health (`health/`)
/// Simple health check endpoint to verify service availability.
/// - Layers: domain, presentation
///
/// ### Users (`users/`)
/// User management functionality with CRUD operations.
/// - Layers: domain, application (service), presentation (handlers)
///
/// ### JSON-RPC (`jsonrpc/`)
/// WebSocket-based JSON-RPC 2.0 protocol for real-time communication.
/// - Layers: domain, application (service), presentation (handler)
///
/// ## Benefits of this structure
///
/// 1. **High Cohesion**: Related code is grouped together by feature
/// 2. **Low Coupling**: Features are independent and self-contained
/// 3. **Easy Navigation**: Clear structure makes finding code intuitive
/// 4. **Scalability**: New features can be added without affecting existing ones
/// 5. **Testability**: Each layer can be tested independently

pub mod auth;
pub mod health;
pub mod jsonrpc;
pub mod users;

// Re-export commonly used items for convenience
pub use auth::{
    anonymous_token, auth_middleware, login, me, optional_auth_middleware, register, AuthService,
    AuthenticatedUser,
};
pub use health::{health_check, HealthResponse};
pub use jsonrpc::{websocket_handler, JsonRpcService};
pub use users::{create_user, get_user, list_users, User, UserService};
