//! Bearer token authentication middleware and constant-time compare helpers.

use axum::Json;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use subtle::ConstantTimeEq;

use ai_brains_contracts::response::ApiError;

/// Auth configuration held in application state.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Opaque bearer token (never log the full value).
    pub bearer_token: String,
}

/// Constant-time string compare for bearer tokens.
///
/// When lengths differ, still performs a self-compare on the expected token
/// so the early reject path is not a pure length oracle on the secret alone.
pub fn tokens_equal(provided: &str, expected: &str) -> bool {
    token_bytes_equal(provided.as_bytes(), expected.as_bytes())
}

/// Constant-time byte compare (public for unit tests).
pub fn token_bytes_equal(provided: &[u8], expected: &[u8]) -> bool {
    if provided.len() != expected.len() {
        // Touch expected so missing/short tokens still do constant-time work.
        let _ = expected.ct_eq(expected);
        return false;
    }
    bool::from(provided.ct_eq(expected))
}

/// Extractor: requires `Authorization: Bearer <token>` matching state.
pub struct Authenticated;

/// Rejection body for auth failures (structured, no stack traces).
#[derive(Debug)]
pub struct AuthRejection {
    pub status: StatusCode,
    pub error: ApiError,
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        let body = Json(self.error);
        (self.status, body).into_response()
    }
}

impl AuthRejection {
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            error: ApiError::new("UNAUTHORIZED", message),
        }
    }
}

impl<S> FromRequestParts<S> for Authenticated
where
    S: Send + Sync,
    AuthConfig: axum::extract::FromRef<S>,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = AuthConfig::from_ref(state);
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let Some(raw) = header else {
            return Err(AuthRejection::unauthorized(
                "missing Authorization bearer token",
            ));
        };

        let Some(token) = raw
            .strip_prefix("Bearer ")
            .or_else(|| raw.strip_prefix("bearer "))
        else {
            return Err(AuthRejection::unauthorized(
                "Authorization header must be Bearer <token>",
            ));
        };

        let token = token.trim();
        if token.is_empty() {
            return Err(AuthRejection::unauthorized("empty bearer token"));
        }

        if !tokens_equal(token, &auth.bearer_token) {
            // Never include provided/expected token material in the message.
            return Err(AuthRejection::unauthorized("invalid bearer token"));
        }

        Ok(Authenticated)
    }
}
