use std::borrow::Cow;

use axum::response::IntoResponse;
use reqwest::StatusCode;

#[derive(utoipa::ToSchema)]
pub struct AppErrorResponse {
    error: String,
}

#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    pub message: Cow<'static, str>,
}

impl AppError {
    pub fn new(status: StatusCode, message: impl Into<Cow<'static, str>>) -> Self {
        AppError { 
            status, 
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    pub fn bad_request(message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn forbidden(message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(StatusCode::FORBIDDEN, message)
    }

    pub fn unauthorized(message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    pub fn not_found(message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let body;
        if self.status.is_server_error() {
            body = serde_json::json!({
                "error": "Internal Server Error",
            });
            tracing::error!("{}", self.message);
        } else {
            body = serde_json::json!({
                "error": self.message,
            });
        }
        (self.status, axum::Json(body)).into_response()
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_status() {
            if let Some(status) = err.status() {
                return AppError::new(status, format!("HTTP request failed with status: {}", status));
            }
        }
        AppError::internal(format!("HTTP request error: {}", err))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::internal(format!("Database error: {}", err))
    }
}