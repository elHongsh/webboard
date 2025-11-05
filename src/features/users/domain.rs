use serde::{Deserialize, Serialize};

/// User domain model
///
/// Core business entity representing a user in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
}

/// Request payload for creating a user
///
/// Value object for user creation with built-in validation.
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
}

impl CreateUserRequest {
    /// Validate user creation request
    ///
    /// Enforces business rules:
    /// - Username must not be empty
    /// - Username must be at least 3 characters
    /// - Email must contain '@' symbol
    pub fn validate(&self) -> Result<(), String> {
        if self.username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if self.username.len() < 3 {
            return Err("Username must be at least 3 characters".to_string());
        }
        if !self.email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_request() {
        let request = CreateUserRequest {
            username: "john".to_string(),
            email: "john@example.com".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_invalid_username_empty() {
        let request = CreateUserRequest {
            username: "".to_string(),
            email: "test@example.com".to_string(),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_invalid_username_too_short() {
        let request = CreateUserRequest {
            username: "ab".to_string(),
            email: "test@example.com".to_string(),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_invalid_email() {
        let request = CreateUserRequest {
            username: "john".to_string(),
            email: "invalid".to_string(),
        };
        assert!(request.validate().is_err());
    }
}
