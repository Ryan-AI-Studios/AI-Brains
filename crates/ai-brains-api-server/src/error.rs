//! HTTP status mapping for structured `ApiError` / `DaemonResponse`.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use ai_brains_contracts::response::ApiError;
use ai_brains_daemon_api::DaemonResponse;

use crate::dispatch::DispatchError;

/// Map a stable `ApiError.code` to an HTTP status.
pub fn http_status_for_api_error_code(code: &str) -> StatusCode {
    let upper = code.to_ascii_uppercase();
    match upper.as_str() {
        "UNAUTHORIZED" | "UNAUTHENTICATED" => StatusCode::UNAUTHORIZED,
        "POLICY_DENIED" | "FORBIDDEN" => StatusCode::FORBIDDEN,
        "NOT_FOUND" => StatusCode::NOT_FOUND,
        c if c.starts_with("INVALID_") => StatusCode::BAD_REQUEST,
        "PAYLOAD_TOO_LARGE" => StatusCode::PAYLOAD_TOO_LARGE,
        "UNSUPPORTED_OPERATION" | "NOT_IMPLEMENTED" => StatusCode::NOT_IMPLEMENTED,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Map a successful dispatch result (including domain `Error` variants) to HTTP.
pub fn map_daemon_response(resp: DaemonResponse) -> Response {
    match &resp {
        DaemonResponse::Error(err) => {
            let status = http_status_for_api_error_code(&err.code);
            // Prefer raw DaemonResponse JSON for IPC parity (type/payload tag).
            (status, Json(resp)).into_response()
        }
        _ => (StatusCode::OK, Json(resp)).into_response(),
    }
}

/// Errors raised by the HTTP adapter itself (not domain).
#[derive(Debug)]
pub enum ApiHttpError {
    BadRequest(String),
    Unauthorized(String),
    PayloadTooLarge,
    Dispatch(DispatchError),
    Internal(String),
}

impl ApiHttpError {
    pub fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Dispatch(DispatchError::NotSupported(_)) => StatusCode::NOT_IMPLEMENTED,
            Self::Dispatch(DispatchError::Internal(_)) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn to_api_error(&self) -> ApiError {
        match self {
            Self::BadRequest(msg) => ApiError::new("INVALID_REQUEST", msg.clone()),
            Self::Unauthorized(msg) => ApiError::new("UNAUTHORIZED", msg.clone()),
            Self::PayloadTooLarge => {
                ApiError::new("PAYLOAD_TOO_LARGE", "request body exceeds 1 MiB limit")
            }
            Self::Dispatch(DispatchError::NotSupported(msg)) => {
                ApiError::new("UNSUPPORTED_OPERATION", msg.clone())
            }
            Self::Dispatch(DispatchError::Internal(msg)) => ApiError::new("INTERNAL", msg.clone()),
            Self::Internal(msg) => ApiError::new("INTERNAL", msg.clone()),
        }
    }
}

impl IntoResponse for ApiHttpError {
    fn into_response(self) -> Response {
        let status = self.status();
        let err = self.to_api_error();
        // Keep adapter errors as DaemonResponse::Error for body shape parity.
        let body = DaemonResponse::Error(err);
        (status, Json(body)).into_response()
    }
}

impl From<DispatchError> for ApiHttpError {
    fn from(value: DispatchError) -> Self {
        Self::Dispatch(value)
    }
}
