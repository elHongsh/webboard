use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

/// Anonymous User Identifier
///
/// Unique identifier for anonymous users based on composite key:
/// {Hospital Code, User ID, User Start Date, Department Code}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnonymousUserIdentifier {
    pub hospital_code: String,
    pub user_id: String,
    pub user_start_date: NaiveDate,
    pub department_code: String,
}

impl AnonymousUserIdentifier {
    /// Validate anonymous user identifier
    pub fn validate(&self) -> Result<(), String> {
        if self.hospital_code.is_empty() {
            return Err("Hospital code cannot be empty".to_string());
        }
        if self.user_id.is_empty() {
            return Err("User ID cannot be empty".to_string());
        }
        if self.department_code.is_empty() {
            return Err("Department code cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Verified User domain model
///
/// Represents an authenticated user with credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedUser {
    pub id: u64,
    pub username: String,
    pub email: String,
}

/// User Identity
///
/// Enum to distinguish between verified and anonymous users.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum UserIdentity {
    Verified(VerifiedUser),
    Anonymous(AnonymousUserIdentifier),
}

impl UserIdentity {
    /// Check if user is verified
    pub fn is_verified(&self) -> bool {
        matches!(self, UserIdentity::Verified(_))
    }

    /// Check if user is anonymous
    pub fn is_anonymous(&self) -> bool {
        matches!(self, UserIdentity::Anonymous(_))
    }

    /// Get verified user if available
    pub fn as_verified(&self) -> Option<&VerifiedUser> {
        match self {
            UserIdentity::Verified(user) => Some(user),
            _ => None,
        }
    }

    /// Get anonymous user identifier if available
    pub fn as_anonymous(&self) -> Option<&AnonymousUserIdentifier> {
        match self {
            UserIdentity::Anonymous(identifier) => Some(identifier),
            _ => None,
        }
    }
}

/// Legacy User domain model (kept for backward compatibility)
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

    #[test]
    fn test_anonymous_user_identifier_validation() {
        let valid_identifier = AnonymousUserIdentifier {
            hospital_code: "H001".to_string(),
            user_id: "U123".to_string(),
            user_start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            department_code: "D001".to_string(),
        };
        assert!(valid_identifier.validate().is_ok());

        let invalid_identifier = AnonymousUserIdentifier {
            hospital_code: "".to_string(),
            user_id: "U123".to_string(),
            user_start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            department_code: "D001".to_string(),
        };
        assert!(invalid_identifier.validate().is_err());
    }

    #[test]
    fn test_user_identity_verified() {
        let verified = UserIdentity::Verified(VerifiedUser {
            id: 1,
            username: "john".to_string(),
            email: "john@example.com".to_string(),
        });

        assert!(verified.is_verified());
        assert!(!verified.is_anonymous());
        assert!(verified.as_verified().is_some());
        assert!(verified.as_anonymous().is_none());
    }

    #[test]
    fn test_user_identity_anonymous() {
        let anonymous = UserIdentity::Anonymous(AnonymousUserIdentifier {
            hospital_code: "H001".to_string(),
            user_id: "U123".to_string(),
            user_start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            department_code: "D001".to_string(),
        });

        assert!(!anonymous.is_verified());
        assert!(anonymous.is_anonymous());
        assert!(anonymous.as_verified().is_none());
        assert!(anonymous.as_anonymous().is_some());
    }
}
