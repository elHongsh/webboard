use chrono::{Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::features::users::domain::{AnonymousUserIdentifier, UserIdentity, VerifiedUser};

/// JWT Claims for verified users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedUserClaims {
    pub sub: String, // user id
    pub username: String,
    pub email: String,
    pub exp: usize, // expiration timestamp
    pub iat: usize, // issued at timestamp
}

impl VerifiedUserClaims {
    /// Create new claims for a verified user
    pub fn new(user: &VerifiedUser) -> Self {
        let now = Utc::now();
        let expiration = now + Duration::hours(24); // 24 hours expiration

        Self {
            sub: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            iat: now.timestamp() as usize,
            exp: expiration.timestamp() as usize,
        }
    }
}

/// JWT Claims for anonymous users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymousUserClaims {
    pub hospital_code: String,
    pub user_id: String,
    #[serde(with = "naive_date_serde")]
    pub user_start_date: NaiveDate,
    pub department_code: String,
    pub exp: usize, // expiration timestamp
    pub iat: usize, // issued at timestamp
}

impl AnonymousUserClaims {
    /// Create new claims for an anonymous user
    pub fn new(identifier: &AnonymousUserIdentifier) -> Self {
        let now = Utc::now();
        let expiration = now + Duration::hours(12); // 12 hours expiration for anonymous users

        Self {
            hospital_code: identifier.hospital_code.clone(),
            user_id: identifier.user_id.clone(),
            user_start_date: identifier.user_start_date,
            department_code: identifier.department_code.clone(),
            iat: now.timestamp() as usize,
            exp: expiration.timestamp() as usize,
        }
    }

    /// Convert to AnonymousUserIdentifier
    pub fn to_identifier(&self) -> AnonymousUserIdentifier {
        AnonymousUserIdentifier {
            hospital_code: self.hospital_code.clone(),
            user_id: self.user_id.clone(),
            user_start_date: self.user_start_date,
            department_code: self.department_code.clone(),
        }
    }
}

/// Custom serializer/deserializer for NaiveDate
mod naive_date_serde {
    use chrono::NaiveDate;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&date.format("%Y-%m-%d").to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)
    }
}

/// Token type to distinguish between verified and anonymous user tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TokenClaims {
    Verified(VerifiedUserClaims),
    Anonymous(AnonymousUserClaims),
}

impl TokenClaims {
    /// Get expiration timestamp
    pub fn exp(&self) -> usize {
        match self {
            TokenClaims::Verified(claims) => claims.exp,
            TokenClaims::Anonymous(claims) => claims.exp,
        }
    }

    /// Convert to UserIdentity
    pub fn to_user_identity(&self) -> UserIdentity {
        match self {
            TokenClaims::Verified(claims) => UserIdentity::Verified(VerifiedUser {
                id: claims.sub.parse().unwrap_or(0),
                username: claims.username.clone(),
                email: claims.email.clone(),
            }),
            TokenClaims::Anonymous(claims) => {
                UserIdentity::Anonymous(claims.to_identifier())
            }
        }
    }
}

/// Authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub token_type: String, // "Bearer"
}

impl AuthToken {
    /// Create a new Bearer token
    pub fn bearer(token: String) -> Self {
        Self {
            token,
            token_type: "Bearer".to_string(),
        }
    }
}

/// Login request for verified users
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

impl LoginRequest {
    /// Validate login request
    pub fn validate(&self) -> Result<(), String> {
        if self.username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if self.password.is_empty() {
            return Err("Password cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Register request for verified users
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl RegisterRequest {
    /// Validate register request
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
        if self.password.len() < 8 {
            return Err("Password must be at least 8 characters".to_string());
        }
        Ok(())
    }
}
