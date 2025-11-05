pub mod websocket;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::error::AppError;
use crate::models::{CreateUserRequest, HealthResponse, User};
use crate::services::UserService;

pub use websocket::websocket_handler;

/// Health check handler
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Query parameters for list users endpoint
#[derive(Deserialize)]
pub struct ListUsersQuery {
    limit: Option<usize>,
}

/// List users handler
pub async fn list_users(
    State(user_service): State<UserService>,
    Query(params): Query<ListUsersQuery>,
) -> Result<Json<Vec<User>>, AppError> {
    let users = user_service.list_users(params.limit).await?;
    Ok(Json(users))
}

/// Create user handler
pub async fn create_user(
    State(user_service): State<UserService>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let user = user_service.create_user(payload).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

/// Get user by ID handler
pub async fn get_user(
    State(user_service): State<UserService>,
    Path(id): Path<u64>,
) -> Result<Json<User>, AppError> {
    let user = user_service.get_user(id).await?;
    Ok(Json(user))
}
