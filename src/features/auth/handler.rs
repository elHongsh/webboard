use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::features::users::domain::AnonymousUserIdentifier;
use crate::infrastructure::error::AppError;

use super::{
    domain::{AuthToken, LoginRequest, RegisterRequest},
    service::AuthService,
};

/// Register a new verified user
///
/// POST /api/v1/auth/register
///
/// Request body:
/// ```json
/// {
///   "username": "john",
///   "email": "john@example.com",
///   "password": "password123"
/// }
/// ```
///
/// Response (201 Created):
/// ```json
/// {
///   "id": 1,
///   "username": "john",
///   "email": "john@example.com"
/// }
/// ```
pub async fn register(
    State(auth_service): State<AuthService>,
    Json(request): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_service.register(request).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

/// Login as a verified user
///
/// POST /api/v1/auth/login
///
/// Request body:
/// ```json
/// {
///   "username": "john",
///   "password": "password123"
/// }
/// ```
///
/// Response (200 OK):
/// ```json
/// {
///   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
///   "token_type": "Bearer"
/// }
/// ```
pub async fn login(
    State(auth_service): State<AuthService>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let token = auth_service.login(request).await?;
    Ok(Json(token))
}

/// Get an authentication token for an anonymous user
///
/// POST /api/v1/auth/anonymous
///
/// Request body:
/// ```json
/// {
///   "hospital_code": "H001",
///   "user_id": "U123",
///   "user_start_date": "2024-01-01",
///   "department_code": "D001"
/// }
/// ```
///
/// Response (200 OK):
/// ```json
/// {
///   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
///   "token_type": "Bearer"
/// }
/// ```
pub async fn anonymous_token(
    State(auth_service): State<AuthService>,
    Json(identifier): Json<AnonymousUserIdentifier>,
) -> Result<impl IntoResponse, AppError> {
    let token = auth_service.generate_anonymous_user_token(&identifier)?;
    Ok(Json(AuthToken::bearer(token)))
}

/// Get current authenticated user info
///
/// GET /api/v1/auth/me
///
/// Requires authentication via Authorization header.
///
/// Response (200 OK) for verified user:
/// ```json
/// {
///   "type": "verified",
///   "id": 1,
///   "username": "john",
///   "email": "john@example.com"
/// }
/// ```
///
/// Response (200 OK) for anonymous user:
/// ```json
/// {
///   "type": "anonymous",
///   "hospital_code": "H001",
///   "user_id": "U123",
///   "user_start_date": "2024-01-01",
///   "department_code": "D001"
/// }
/// ```
pub async fn me(
    user: super::middleware::AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(user.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::{get, post},
        Router,
    };
    use chrono::NaiveDate;
    use tower::util::ServiceExt;

    fn create_test_app() -> Router {
        let auth_service = AuthService::new("test_secret".to_string());

        Router::new()
            .route("/auth/register", post(register))
            .route("/auth/login", post(login))
            .route("/auth/anonymous", post(anonymous_token))
            .route(
                "/auth/me",
                get(me).layer(middleware::from_fn_with_state(
                    auth_service.clone(),
                    super::super::middleware::auth_middleware,
                )),
            )
            .with_state(auth_service)
    }

    #[tokio::test]
    async fn test_register_endpoint() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/auth/register")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"username":"testuser","email":"test@example.com","password":"password123"}"#,
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_login_endpoint() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/auth/login")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"username":"testuser","password":"password123"}"#,
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_anonymous_token_endpoint() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/auth/anonymous")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"hospital_code":"H001","user_id":"U123","user_start_date":"2024-01-01","department_code":"D001"}"#,
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_me_endpoint_with_auth() {
        let auth_service = AuthService::new("test_secret".to_string());
        let identifier = AnonymousUserIdentifier {
            hospital_code: "H001".to_string(),
            user_id: "U123".to_string(),
            user_start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            department_code: "D001".to_string(),
        };
        let token = auth_service
            .generate_anonymous_user_token(&identifier)
            .unwrap();

        let app = create_test_app();

        let request = Request::builder()
            .uri("/auth/me")
            .method("GET")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
