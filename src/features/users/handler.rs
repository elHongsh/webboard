use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::infrastructure::AppError;

use super::domain::{CreateUserRequest, User};
use super::service::UserService;

/// Query parameters for list users endpoint
#[derive(Deserialize)]
pub struct ListUsersQuery {
    limit: Option<usize>,
}

/// List users handler
///
/// Presentation layer handler for listing users with optional pagination.
///
/// # Route
/// GET /api/v1/users?limit=10
///
/// # Response
/// ```json
/// [
///   {"id": 1, "username": "user1", "email": "user1@example.com"},
///   {"id": 2, "username": "user2", "email": "user2@example.com"}
/// ]
/// ```
pub async fn list_users(
    State(user_service): State<UserService>,
    Query(params): Query<ListUsersQuery>,
) -> Result<Json<Vec<User>>, AppError> {
    let users = user_service.list_users(params.limit).await?;
    Ok(Json(users))
}

/// Create user handler
///
/// Presentation layer handler for creating a new user.
///
/// # Route
/// POST /api/v1/users
///
/// # Request Body
/// ```json
/// {
///   "username": "john",
///   "email": "john@example.com"
/// }
/// ```
///
/// # Response
/// 201 Created
/// ```json
/// {
///   "id": 1,
///   "username": "john",
///   "email": "john@example.com"
/// }
/// ```
pub async fn create_user(
    State(user_service): State<UserService>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let user = user_service.create_user(payload).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

/// Get user by ID handler
///
/// Presentation layer handler for retrieving a specific user.
///
/// # Route
/// GET /api/v1/users/:id
///
/// # Response
/// ```json
/// {
///   "id": 5,
///   "username": "user5",
///   "email": "user5@example.com"
/// }
/// ```
pub async fn get_user(
    State(user_service): State<UserService>,
    Path(id): Path<u64>,
) -> Result<Json<User>, AppError> {
    let user = user_service.get_user(id).await?;
    Ok(Json(user))
}
