use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::infrastructure::AppError;

use super::domain::{CreateUserRequest, User};

/// User service containing business logic
///
/// Application layer service that orchestrates user-related operations.
/// In a real application, this would interact with a database repository.
#[derive(Clone)]
pub struct UserService {
    next_id: Arc<AtomicU64>,
}

impl UserService {
    /// Create a new user service
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Create a new user
    ///
    /// # Business Logic
    /// 1. Validate the request
    /// 2. Generate a unique ID
    /// 3. Create the user entity
    /// 4. (In real app: persist to database)
    /// 5. Return the created user
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
    ///
    /// # Business Logic
    /// 1. Validate the ID
    /// 2. (In real app: fetch from database)
    /// 3. Return the user or error if not found
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
    ///
    /// # Business Logic
    /// 1. Validate and apply limit (max 100 items)
    /// 2. (In real app: fetch from database with pagination)
    /// 3. Return the list of users
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_user_success() {
        let service = UserService::new();
        let request = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        let result = service.create_user(request).await;
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_create_user_invalid() {
        let service = UserService::new();
        let request = CreateUserRequest {
            username: "ab".to_string(), // Too short
            email: "test@example.com".to_string(),
        };

        let result = service.create_user(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_user_valid() {
        let service = UserService::new();
        let result = service.get_user(5).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let service = UserService::new();
        let result = service.get_user(999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_users() {
        let service = UserService::new();
        let result = service.list_users(Some(5)).await;
        assert!(result.is_ok());

        let users = result.unwrap();
        assert_eq!(users.len(), 5);
    }
}
