/// Users Feature Module
///
/// Manages user-related functionality with clear layer separation.
///
/// ## Architecture
///
/// This module follows clean architecture principles with three distinct layers:
///
/// ### Domain Layer (`domain.rs`)
/// - `User`: Core business entity
/// - `CreateUserRequest`: Value object with validation
/// - Contains business rules and validations
/// - No dependencies on other layers
///
/// ### Application Layer (`service.rs`)
/// - `UserService`: Business logic orchestration
/// - Coordinates operations between domain and infrastructure
/// - In a real app, would interact with repository/database
///
/// ### Presentation Layer (`handler.rs`)
/// - HTTP request handlers
/// - Request/response mapping
/// - Route handling for user endpoints
///
/// ## Usage
/// ```rust
/// use features::users;
///
/// // Initialize service
/// let user_service = users::UserService::new();
///
/// // Build routes
/// Router::new()
///     .route("/users", get(users::list_users).post(users::create_user))
///     .route("/users/:id", get(users::get_user))
///     .with_state(user_service)
/// ```

pub mod domain;
pub mod handler;
pub mod service;

// Re-export commonly used items
pub use domain::{CreateUserRequest, User};
pub use handler::{create_user, get_user, list_users};
pub use service::UserService;
