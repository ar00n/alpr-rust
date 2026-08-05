use axum::{Extension, Json, extract::{Path, State}, http::StatusCode};
use chrono::{Duration, Utc};
use jsonwebtoken::encode;

use crate::{
    models::{ChangePasswordPayload, Claims, CreateUserPayload, LoginResponse, User, UserLoginRequest, UserResponse}, state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/users",
    request_body = CreateUserPayload,
    responses(
        (status = 200, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Bad request (e.g. user exists)", body = String),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Admin only)"),
        (status = 500, description = "Server error", body = String)
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
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    let hash = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let id = sqlx::query!(
        r#"INSERT INTO users (username, password_hash, is_admin) VALUES (?, ?, ?)"#,
        &payload.username,
        &hash,
        payload.is_admin
    )
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
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
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let hash = sqlx::query_scalar!(
        "SELECT password_hash FROM users WHERE username = $1",
        payload.username
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".into()))?;

    let success = bcrypt::verify(&payload.password, &hash)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".into()))?;

    if !success {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into()));
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
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into()))?,
    };

    let token = encode(&state.jwt.header, &claims, &state.jwt.encoding_key).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error generating authentication token".into(),
        )
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
        (status = 400, description = "Bad Request (Cannot delete the last admin)"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Admin only)"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Server error")
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
) -> Result<StatusCode, (StatusCode, String)> {
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let target_user = sqlx::query!("SELECT is_admin FROM users WHERE id = ?", id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user_to_delete = match target_user {
        Some(user) => user,
        None => return Err((StatusCode::NOT_FOUND, "User not found".into())),
    };

    if user_to_delete.is_admin {
        let admin_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE is_admin = 1"
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if admin_count <= 1 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Cannot delete the last admin user".into(),
            ));
        }
    }

    sqlx::query!("DELETE FROM users WHERE id = ?", id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Not admin and not self)"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Server error")
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
) -> Result<StatusCode, (StatusCode, String)> {
    if !user.is_admin && user.id != Some(id) {
        return Err((
            StatusCode::FORBIDDEN,
            "You can only change your own password unless you are an admin".into(),
        ));
    }

    let hash = bcrypt::hash(&payload.new_password, bcrypt::DEFAULT_COST)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = sqlx::query!(
        "UPDATE users SET password_hash = ? WHERE id = ?",
        hash,
        id
    )
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "User not found".into()));
    }

    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "List of all users retrieved successfully", body = [UserResponse]),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Admin only)"),
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
) -> Result<Json<Vec<UserResponse>>, (StatusCode, String)> {
    let users = sqlx::query_as!(
        UserResponse,
        r#"SELECT id, username, is_admin FROM users"#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(users))
}