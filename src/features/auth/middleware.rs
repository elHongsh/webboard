use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::features::users::domain::UserIdentity;

use super::service::AuthService;

/// Extension type for storing authenticated user in request
#[derive(Clone, Debug)]
pub struct AuthenticatedUser(pub UserIdentity);

/// Authentication middleware
///
/// Extracts and validates JWT token from Authorization header.
/// Adds UserIdentity to request extensions if authentication succeeds.
pub async fn auth_middleware(
    State(auth_service): State<AuthService>,
    mut request: Request,
    next: Next,
) -> Response {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    // If no authorization header, return unauthorized
    let Some(auth_header) = auth_header else {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
                "error": "Missing authorization header"
            })),
        )
            .into_response();
    };

    // Extract user from header
    match auth_service.extract_user_from_header(auth_header) {
        Ok(user_identity) => {
            // Add user to request extensions
            request.extensions_mut().insert(AuthenticatedUser(user_identity));
            next.run(request).await
        }
        Err(e) => {
            (
                StatusCode::UNAUTHORIZED,
                axum::Json(json!({
                    "error": format!("Authentication failed: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// Optional authentication middleware
///
/// Similar to auth_middleware but doesn't fail if no authorization header is present.
/// Useful for endpoints that work for both authenticated and unauthenticated users.
pub async fn optional_auth_middleware(
    State(auth_service): State<AuthService>,
    mut request: Request,
    next: Next,
) -> Response {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    // Try to extract user if header is present
    if let Some(auth_header) = auth_header {
        if let Ok(user_identity) = auth_service.extract_user_from_header(auth_header) {
            request.extensions_mut().insert(AuthenticatedUser(user_identity));
        }
    }

    next.run(request).await
}

/// Extractor for authenticated user
///
/// Use this in handlers to get the authenticated user.
/// This will fail with 401 if the user is not authenticated.
#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({
                        "error": "Authentication required"
                    })),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        middleware,
        routing::get,
        Router,
    };
    use tower::util::ServiceExt;
    use crate::features::users::domain::VerifiedUser;

    async fn test_handler(user: AuthenticatedUser) -> impl IntoResponse {
        axum::Json(json!({
            "authenticated": true,
            "is_verified": user.0.is_verified(),
            "is_anonymous": user.0.is_anonymous(),
        }))
    }

    async fn test_optional_handler(user: Option<AuthenticatedUser>) -> impl IntoResponse {
        match user {
            Some(auth_user) => axum::Json(json!({
                "authenticated": true,
                "is_verified": auth_user.0.is_verified(),
            })),
            None => axum::Json(json!({
                "authenticated": false,
            })),
        }
    }

    #[tokio::test]
    async fn test_auth_middleware_with_valid_token() {
        let auth_service = AuthService::new("test_secret".to_string());
        let user = VerifiedUser {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };
        let token = auth_service.generate_verified_user_token(&user).unwrap();

        let app = Router::new()
            .route("/protected", get(test_handler))
            .layer(middleware::from_fn_with_state(
                auth_service.clone(),
                auth_middleware,
            ))
            .with_state(auth_service);

        let request = Request::builder()
            .uri("/protected")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_middleware_without_token() {
        let auth_service = AuthService::new("test_secret".to_string());

        let app = Router::new()
            .route("/protected", get(test_handler))
            .layer(middleware::from_fn_with_state(
                auth_service.clone(),
                auth_middleware,
            ))
            .with_state(auth_service);

        let request = Request::builder()
            .uri("/protected")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_optional_auth_middleware_without_token() {
        let auth_service = AuthService::new("test_secret".to_string());

        let app = Router::new()
            .route("/optional", get(test_optional_handler))
            .layer(middleware::from_fn_with_state(
                auth_service.clone(),
                optional_auth_middleware,
            ))
            .with_state(auth_service);

        let request = Request::builder()
            .uri("/optional")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
