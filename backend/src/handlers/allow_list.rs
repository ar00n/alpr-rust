use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::{DateTime, Utc};

use crate::{
    error::{AppError, AppErrorResponse}, models::{AllowListEntry, User}, state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/allow-list",
    request_body = AllowListEntry,
    responses(
        (status = 200, description = "Plate added or updated in allow list", body = AllowListEntry),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Allow List"
)]
pub async fn add_allow_list(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
    Json(payload): Json<AllowListEntry>,
) -> Result<Json<AllowListEntry>, AppError> {
    sqlx::query!(
        r#"INSERT INTO allow_list (plate, expiry_date)
        VALUES (?, ?)
        ON CONFLICT(plate)
        DO UPDATE SET expiry_date = excluded.expiry_date
        "#,
        &payload.plate,
        &payload.expiry_date
    )
    .execute(&state.db)
    .await?;

    Ok(Json(payload))
}

#[utoipa::path(
    get,
    path = "/api/allow-list",
    responses(
        (status = 200, description = "List of allow-listed plates", body = [AllowListEntry]),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Allow List"
)]
pub async fn get_allow_list(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<Json<Vec<AllowListEntry>>, AppError> {
    let list = sqlx::query_as!(
        AllowListEntry, 
        r#"SELECT plate, expiry_date AS "expiry_date: DateTime<Utc>" FROM allow_list"#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(Json(list))
}

#[utoipa::path(
    delete,
    path = "/api/allow-list/{plate}",
    params(
        ("plate" = String, Path, description = "The license plate to remove")
    ),
    responses(
        (status = 204, description = "Plate successfully deleted"),
        (status = 404, description = "Plate not found in the allow list", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Allow List"
)]
pub async fn delete_allow_list(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
    Path(plate): Path<String>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query!(
        r#"DELETE FROM allow_list WHERE plate = ?"#,
        plate
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!("Plate '{}' not found", plate)));
    }

    Ok(StatusCode::NO_CONTENT)
}