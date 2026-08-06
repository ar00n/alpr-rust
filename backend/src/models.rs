use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Username
    pub iat: usize,  // Issued at timestamp
    pub exp: usize,  // Expiration timestamp
    pub is_admin: bool, // Admin flag
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct PlateRead {
    pub id: Option<i64>,
    pub plate: String,
    pub confidence: f64,
    pub snapshot_image: Option<String>,
    pub was_allowed: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct AllowListEntry {
    pub plate: String,
    pub expiry_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub password_hash: String,
    pub is_admin: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateUserPayload {
    pub username: String,
    pub password: String,
    pub is_admin: bool,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ChangePasswordPayload {
    pub new_password: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct UpdateFrameratePayload {
    pub framerate: u32,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct UpdateRTSPUrlPayload {
    pub rtsp_url: Option<String>,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateTrimSnapshotsPayload {
    pub trim_snapshots_mb: Option<u64>,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateTrimHistoryPayload {
    pub trim_history_days: Option<u64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UserLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Clone, Debug)]
pub struct VideoFrame {
    pub data: Arc<[u8]>,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug)]
pub struct PipelineConfig {
    pub fps: u32,
    pub rtsp_url: Option<String>,
    pub trim_snapshots_mb: Option<u64>,
    pub trim_history_days: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct PlateEvent {
    pub plate: String,
    pub confidence: f32,
    pub frame: VideoFrame,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateCustomAction {
    pub name: String,
    #[schema(example = "POST")]
    pub method: String,
    #[schema(example = "http://192.168.1.10/open-gate")]
    pub url: String,
    #[schema(example = "BASIC")]
    pub auth_type: String, 
    /// Sensitive credentials (e.g., {"username": "admin", "password": "123"})
    pub auth_data: Option<serde_json::Value>, 
    /// Optional custom headers
    pub headers: Option<serde_json::Value>, 
    #[schema(example = "{\"plate\": \"${LICENCE_PLATE}\"}")]
    pub body_template: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CustomActionResponse {
    pub id: i64,
    pub name: String,
    pub method: String,
    pub url: String,
    pub auth_type: String,
    // Note: auth_data is intentionally omitted for security!
    pub headers: Option<serde_json::Value>,
    pub body_template: Option<String>,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct TestActionResponse {
    pub status: u16,
    pub body: String,
}