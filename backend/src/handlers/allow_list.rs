use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use chrono::{DateTime, Utc};

use crate::{
    error::{AppError, AppErrorResponse},
    models::{AllowListEntry, User},
    state::AppState,
};

fn validate_and_sanitize_plate(plate: &str) -> Result<String, AppError> {
    let sanitized: String = plate.chars().filter(|c| !c.is_whitespace()).collect();
    let sanitized = sanitized.to_uppercase();

    if sanitized.is_empty() {
        return Err(AppError::bad_request("License plate cannot be empty"));
    }

    if !sanitized.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(AppError::bad_request(
            "License plate contains invalid symbols. Only alphanumeric characters and hyphens are allowed.",
        ));
    }

    Ok(sanitized)
}

#[utoipa::path(
    post,
    path = "/api/allow-list",
    request_body = AllowListEntry,
    responses(
        (status = 200, description = "Plate added or updated in allow list", body = AllowListEntry),
        (status = 400, description = "Bad Request (e.g. invalid symbols in plate)", body = AppErrorResponse),
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
    Json(mut payload): Json<AllowListEntry>,
) -> Result<Json<AllowListEntry>, AppError> {
    // Sanitize and validate the plate payload
    payload.plate = validate_and_sanitize_plate(&payload.plate)?;

    sqlx::query!(
        r#"INSERT INTO allow_list (plate, expiry_date, name, metadata)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(plate)
        DO UPDATE SET 
            expiry_date = excluded.expiry_date,
            name = excluded.name,
            metadata = excluded.metadata
        "#,
        &payload.plate,
        &payload.expiry_date,
        &payload.name,
        &payload.metadata
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

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
        r#"SELECT 
            plate, 
            expiry_date AS "expiry_date: DateTime<Utc>",
            name,
            metadata
        FROM allow_list"#
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
        (status = 400, description = "Invalid plate format", body = AppErrorResponse),
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
    let result = sqlx::query!(r#"DELETE FROM allow_list WHERE plate = ?"#, plate)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!("Plate '{}' not found", plate)));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/allow-list/export",
    responses(
        (status = 200, description = "CSV file of the allow list", content_type = "text/csv"),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Allow List"
)]
pub async fn export_allow_list_csv(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<impl IntoResponse, AppError> {
    let list = sqlx::query_as!(
        AllowListEntry,
        r#"SELECT 
            plate, 
            expiry_date AS "expiry_date: DateTime<Utc>",
            name,
            metadata
        FROM allow_list"#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    let mut wtr = csv::Writer::from_writer(vec![]);
    for record in list {
        wtr.serialize(record)
            .map_err(|e| AppError::internal(format!("CSV Serialization error: {}", e)))?;
    }
    let data = wtr
        .into_inner()
        .map_err(|e| AppError::internal(format!("CSV Writer error: {}", e)))?;

    let headers = [
        (header::CONTENT_TYPE, "text/csv"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"allow_list.csv\"",
        ),
    ];

    Ok((headers, data))
}

#[utoipa::path(
    post,
    path = "/api/allow-list/import",
    request_body(content = String, description = "CSV file content", content_type = "text/csv"),
    responses(
        (status = 200, description = "CSV successfully imported"),
        (status = 400, description = "Bad CSV format or invalid plate data", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Allow List"
)]
pub async fn import_allow_list_csv(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
    body: String,
) -> Result<StatusCode, AppError> {
    let mut rdr = csv::Reader::from_reader(body.as_bytes());

    // Using a transaction to ensure atomic bulk uploads
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

    for result in rdr.deserialize::<AllowListEntry>() {
        let mut record =
            result.map_err(|e| AppError::bad_request(format!("Invalid CSV format: {}", e)))?;

        // Sanitize and validate plate inside CSV. Will fail and rollback the transaction if any are invalid.
        record.plate = validate_and_sanitize_plate(&record.plate)?;

        sqlx::query!(
            r#"INSERT INTO allow_list (plate, expiry_date, name, metadata)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(plate)
            DO UPDATE SET 
                expiry_date = excluded.expiry_date,
                name = excluded.name,
                metadata = excluded.metadata"#,
            record.plate,
            record.expiry_date,
            record.name,
            record.metadata
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(StatusCode::OK)
}
