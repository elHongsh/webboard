pub mod jsonrpc_service;

use crate::error::AppError;
use crate::models::{CreateUserRequest, User};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub use jsonrpc_service::JsonRpcService;

/// User service containing business logic
#[derive(Clone)]
pub struct UserService {
    next_id: Arc<AtomicU64>,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Create a new user
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, AppError> {
        // Validate request
        request
            .validate()
            .map_err(|msg| AppError::BadRequest(msg))?;

        // Generate unique ID
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        // Create user (in real app, this would save to database)
        let user = User {
            id,
            username: request.username,
            email: request.email,
        };

        tracing::info!("Created user: {:?}", user);
        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user(&self, id: u64) -> Result<User, AppError> {
        // In real app, fetch from database
        // For demo, return mock user or error
        if id == 0 {
            return Err(AppError::BadRequest("Invalid user ID".to_string()));
        }

        if id > 100 {
            return Err(AppError::NotFound(format!("User {} not found", id)));
        }

        Ok(User {
            id,
            username: format!("user{}", id),
            email: format!("user{}@example.com", id),
        })
    }

    /// List all users (paginated)
    pub async fn list_users(&self, limit: Option<usize>) -> Result<Vec<User>, AppError> {
        let limit = limit.unwrap_or(10).min(100); // Max 100 items

        // In real app, fetch from database with pagination
        // For demo, return mock data
        let users: Vec<User> = (1..=limit)
            .map(|i| User {
                id: i as u64,
                username: format!("user{}", i),
                email: format!("user{}@example.com", i),
            })
            .collect();

        Ok(users)
    }
}

impl Default for UserService {
    fn default() -> Self {
        Self::new()
    }
}