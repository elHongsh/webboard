use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::features::users::domain::{AnonymousUserIdentifier, UserIdentity, VerifiedUser};
use crate::infrastructure::error::AppError;

use super::domain::{
    AnonymousUserClaims, AuthToken, LoginRequest, RegisterRequest, TokenClaims,
    VerifiedUserClaims,
};

/// Authentication Service
///
/// Handles authentication and token management for both verified and anonymous users.
#[derive(Clone)]
pub struct AuthService {
    jwt_secret: String,
    user_id_counter: Arc<AtomicU64>,
}

impl AuthService {
    /// Create a new AuthService
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret,
            user_id_counter: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Register a new verified user (mock implementation)
    ///
    /// In production, this would:
    /// 1. Hash the password with bcrypt
    /// 2. Save the user to the database
    /// 3. Return the created user
    pub async fn register(&self, request: RegisterRequest) -> Result<VerifiedUser, AppError> {
        // Validate request
        request
            .validate()
            .map_err(|e| AppError::BadRequest(e))?;

        // In production, hash the password:
        // let password_hash = bcrypt::hash(&request.password, bcrypt::DEFAULT_COST)
        //     .map_err(|e| AppError::InternalError(format!("Failed to hash password: {}", e)))?;

        // Create user (mock implementation)
        let user = VerifiedUser {
            id: self.user_id_counter.fetch_add(1, Ordering::SeqCst),
            username: request.username,
            email: request.email,
        };

        Ok(user)
    }

    /// Login a verified user (mock implementation)
    ///
    /// In production, this would:
    /// 1. Query the database for the user by username
    /// 2. Verify the password against the stored hash
    /// 3. Generate and return a JWT token
    pub async fn login(&self, request: LoginRequest) -> Result<AuthToken, AppError> {
        // Validate request
        request
            .validate()
            .map_err(|e| AppError::BadRequest(e))?;

        // Mock user lookup and password verification
        // In production, query database and verify password:
        // let user = user_repository.find_by_username(&request.username).await?;
        // bcrypt::verify(&request.password, &user.password_hash)
        //     .map_err(|_| AppError::Unauthorized("Invalid credentials".to_string()))?;

        let mock_user = VerifiedUser {
            id: 1,
            username: request.username.clone(),
            email: format!("{}@example.com", request.username),
        };

        // Generate token
        let token = self.generate_verified_user_token(&mock_user)?;
        Ok(AuthToken::bearer(token))
    }

    /// Generate a token for a verified user
    pub fn generate_verified_user_token(&self, user: &VerifiedUser) -> Result<String, AppError> {
        let claims = VerifiedUserClaims::new(user);

        encode(
            &Header::default(),
            &TokenClaims::Verified(claims),
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::InternalError(format!("Failed to generate token: {}", e)))
    }

    /// Generate a token for an anonymous user
    pub fn generate_anonymous_user_token(
        &self,
        identifier: &AnonymousUserIdentifier,
    ) -> Result<String, AppError> {
        // Validate identifier
        identifier
            .validate()
            .map_err(|e| AppError::BadRequest(e))?;

        let claims = AnonymousUserClaims::new(identifier);

        encode(
            &Header::default(),
            &TokenClaims::Anonymous(claims),
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::InternalError(format!("Failed to generate token: {}", e)))
    }

    /// Verify and decode a token
    pub fn verify_token(&self, token: &str) -> Result<UserIdentity, AppError> {
        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims.to_user_identity())
    }

    /// Extract user identity from Authorization header
    pub fn extract_user_from_header(&self, auth_header: &str) -> Result<UserIdentity, AppError> {
        // Check if header starts with "Bearer "
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Invalid authorization header".to_string()))?;

        self.verify_token(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[tokio::test]
    async fn test_register_valid_user() {
        let service = AuthService::new("test_secret".to_string());
        let request = RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = service.register(request).await;
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_register_invalid_user() {
        let service = AuthService::new("test_secret".to_string());
        let request = RegisterRequest {
            username: "ab".to_string(), // Too short
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = service.register(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login() {
        let service = AuthService::new("test_secret".to_string());
        let request = LoginRequest {
            username: "testuser".to_string(),
            password: "password123".to_string(),
        };

        let result = service.login(request).await;
        assert!(result.is_ok());

        let token = result.unwrap();
        assert_eq!(token.token_type, "Bearer");
        assert!(!token.token.is_empty());
    }

    #[test]
    fn test_generate_and_verify_verified_user_token() {
        let service = AuthService::new("test_secret".to_string());
        let user = VerifiedUser {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        let token = service.generate_verified_user_token(&user).unwrap();
        let identity = service.verify_token(&token).unwrap();

        assert!(identity.is_verified());
        let verified_user = identity.as_verified().unwrap();
        assert_eq!(verified_user.username, "testuser");
    }

    #[test]
    fn test_generate_and_verify_anonymous_user_token() {
        let service = AuthService::new("test_secret".to_string());
        let identifier = AnonymousUserIdentifier {
            hospital_code: "H001".to_string(),
            user_id: "U123".to_string(),
            user_start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            department_code: "D001".to_string(),
        };

        let token = service.generate_anonymous_user_token(&identifier).unwrap();
        let identity = service.verify_token(&token).unwrap();

        assert!(identity.is_anonymous());
        let anonymous_id = identity.as_anonymous().unwrap();
        assert_eq!(anonymous_id.hospital_code, "H001");
        assert_eq!(anonymous_id.user_id, "U123");
    }

    #[test]
    fn test_extract_user_from_header() {
        let service = AuthService::new("test_secret".to_string());
        let user = VerifiedUser {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        let token = service.generate_verified_user_token(&user).unwrap();
        let header = format!("Bearer {}", token);

        let identity = service.extract_user_from_header(&header).unwrap();
        assert!(identity.is_verified());
    }

    #[test]
    fn test_extract_user_from_invalid_header() {
        let service = AuthService::new("test_secret".to_string());
        let result = service.extract_user_from_header("Invalid header");
        assert!(result.is_err());
    }
}
