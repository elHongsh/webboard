/// Authentication feature module
///
/// Provides authentication and authorization functionality for both
/// verified and anonymous users.
///
/// ## Features
///
/// - JWT-based authentication
/// - Support for verified users (with credentials)
/// - Support for anonymous users (identified by composite key)
/// - Authentication middleware for request validation
/// - Token generation and verification
///
/// ## Usage
///
/// ```rust,ignore
/// use crate::features::auth::{AuthService, middleware::auth_middleware};
///
/// // Create auth service
/// let auth_service = AuthService::new("your-secret-key".to_string());
///
/// // Generate token for verified user
/// let token = auth_service.generate_verified_user_token(&user)?;
///
/// // Apply authentication middleware to routes
/// let protected_routes = Router::new()
///     .route("/protected", get(handler))
///     .layer(middleware::from_fn_with_state(
///         auth_service.clone(),
///         auth_middleware,
///     ));
/// ```

pub mod domain;
pub mod handler;
pub mod middleware;
pub mod service;

pub use domain::*;
pub use handler::{anonymous_token, login, me, register};
pub use middleware::{auth_middleware, optional_auth_middleware, AuthenticatedUser};
pub use service::AuthService;
