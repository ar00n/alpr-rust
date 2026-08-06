use axum::{Extension, Json, extract::{Path, State}, http::StatusCode};
use chrono::{Duration, Utc};
use jsonwebtoken::encode;

use crate::{
    error::{AppError, AppErrorResponse}, models::{ChangePasswordPayload, Claims, CreateUserPayload, LoginResponse, User, UserLoginRequest, UserResponse}, state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/users",
    request_body = CreateUserPayload,
    responses(
        (status = 200, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Bad request (e.g. user exists)", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden (Admin only)", body = AppErrorResponse),
        (status = 500, description = "Server error", body = AppErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Users"
)]
pub async fn create_user(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<Json<UserResponse>, AppError> {
    let hash = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::internal(e.to_string()))?;

    let id = sqlx::query!(
        r#"INSERT INTO users (username, password_hash, is_admin) VALUES (?, ?, ?)"#,
        &payload.username,
        &hash,
        payload.is_admin
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError::bad_request(e.to_string()))?
    .last_insert_rowid();

    Ok(Json(UserResponse {
        id,
        username: payload.username,
        is_admin: payload.is_admin,
    }))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = UserLoginRequest,
    responses(
        (status = 200, description = "User login successful", body = LoginResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
pub async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<UserLoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let hash = sqlx::query_scalar!(
        "SELECT password_hash FROM users WHERE username = $1",
        payload.username
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| AppError::unauthorized("Invalid credentials"))?;

    let success = bcrypt::verify(&payload.password, &hash)
        .map_err(|_| AppError::unauthorized("Invalid credentials"))?;

    if !success {
        return Err(AppError::unauthorized("Invalid credentials"));
    }

    let now = Utc::now();
    let expires_at = now + Duration::weeks(1);

    let claims = Claims {
        sub: payload.username.clone(),
        iat: now.timestamp() as usize,
        exp: expires_at.timestamp() as usize,
        is_admin: sqlx::query_scalar!(
            "SELECT is_admin FROM users WHERE username = $1",
            payload.username
        )
        .fetch_one(&state.db)
        .await
        .map_err(|_| AppError::internal("Database error"))?,
    };

    let token = encode(&state.jwt.header, &claims, &state.jwt.encoding_key).map_err(|_| {
        AppError::internal("Error generating authentication token")
    })?;

    Ok(Json(LoginResponse { token }))
}

#[utoipa::path(
    delete,
    path = "/api/users/{id}",
    params(
        ("id" = i64, Path, description = "Database ID of the user")
    ),
    responses(
        (status = 200, description = "User deleted successfully"),
        (status = 400, description = "Bad Request (Cannot delete the last admin)", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden (Admin only)", body = AppErrorResponse),
        (status = 404, description = "User not found", body = AppErrorResponse),
        (status = 500, description = "Server error", body = AppErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Users"
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

    let target_user = sqlx::query!("SELECT is_admin FROM users WHERE id = ?", id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

    let user_to_delete = match target_user {
        Some(user) => user,
        None => return Err(AppError::not_found("User not found")),
    };

    if user_to_delete.is_admin {
        let admin_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE is_admin = 1"
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

        if admin_count <= 1 {
            return Err(AppError::bad_request("Cannot delete the last admin user"));
        }
    }

    sqlx::query!("DELETE FROM users WHERE id = ?", id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    put,
    path = "/api/users/{id}/password",
    request_body = ChangePasswordPayload,
    params(
        ("id" = i64, Path, description = "Database ID of the user")
    ),
    responses(
        (status = 200, description = "Password updated successfully"),
        (status = 400, description = "Bad request", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden (Not admin and not self)", body = AppErrorResponse),
        (status = 404, description = "User not found", body = AppErrorResponse),
        (status = 500, description = "Server error", body = AppErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Users"
)]
pub async fn change_password(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
    Json(payload): Json<ChangePasswordPayload>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin && user.id != Some(id) {
        return Err(AppError::forbidden("You can only change your own password unless you are an admin"));
    }

    let hash = bcrypt::hash(&payload.new_password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::internal(e.to_string()))?;

    let result = sqlx::query!(
        "UPDATE users SET password_hash = ? WHERE id = ?",
        hash,
        id
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found("User not found"));
    }

    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "List of all users retrieved successfully", body = [UserResponse]),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Admin only)", body = AppErrorResponse),
        (status = 500, description = "Server error", body = String)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Users"
)]
pub async fn get_users(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let users = sqlx::query_as!(
        UserResponse,
        r#"SELECT id, username, is_admin FROM users"#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(Json(users))
}