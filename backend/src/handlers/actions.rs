use axum::{extract::{Path, State}, Extension, Json};
use reqwest::{Method, Url};
use std::net::IpAddr;

use crate::{
    actions::{execute_action, is_ip_allowed}, crypto::encrypt_data, error::{AppError, AppErrorResponse}, models::{CreateCustomAction, CustomActionResponse, TestActionResponse, User}, state::AppState
};

#[utoipa::path(
    post,
    path = "/api/custom-actions",
    request_body = CreateCustomAction,
    responses(
        (status = 200, description = "Custom action created successfully", body = CustomActionResponse),
        (status = 400, description = "Invalid input", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "Custom Actions"
)]
pub async fn add_custom_action(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
    Json(payload): Json<CreateCustomAction>,
) -> Result<Json<CustomActionResponse>, AppError> {
    let url = Url::parse(&payload.url).map_err(|_| AppError::bad_request("Invalid URL format"))?;
    
    if let Some(host_str) = url.host_str() {
        if let Ok(ip) = host_str.parse::<IpAddr>() {
            if !is_ip_allowed(ip) {
                return Err(AppError::forbidden("URL points to a restricted IP address"));
            }
        }
    }

    Method::from_bytes(payload.method.to_uppercase().as_bytes())
        .map_err(|_| AppError::bad_request("Invalid HTTP method"))?;

    // --- ENCRYPTION ---
    // Convert and encrypt auth_data if it is provided
    let auth_data_str = match payload.auth_data.as_ref() {
        Some(v) => {
            let json_str = v.to_string();
            let encrypted = encrypt_data(&json_str, &state.encryption_key)?;
            Some(encrypted)
        }
        None => None,
    };

    let headers_str = payload.headers.as_ref().map(|v| v.to_string());
    
    let delay = payload.delay_seconds.unwrap_or(0);

    let action_id = sqlx::query_scalar!(
        r#"
        INSERT INTO custom_actions (
            name, method, url, auth_type, auth_data, headers, body_template, delay_seconds
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
        payload.name,
        payload.method,
        payload.url,
        payload.auth_type,
        auth_data_str,
        headers_str,
        payload.body_template,
        delay
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(CustomActionResponse {
        id: action_id,
        name: payload.name,
        method: payload.method,
        url: payload.url,
        auth_type: payload.auth_type,
        headers: payload.headers, 
        body_template: payload.body_template,
        delay_seconds: Some(delay),
    }))
}

#[utoipa::path(
    post,
    path = "/api/custom-actions/test",
    request_body = CreateCustomAction,
    responses(
        (status = 200, description = "Action executed successfully", body = TestActionResponse),
        (status = 400, description = "Invalid input or Execution failed", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "Custom Actions"
)]
pub async fn test_custom_action(
    Extension(_user): Extension<User>,
    Json(payload): Json<CreateCustomAction>,
) -> Result<Json<TestActionResponse>, AppError> {
    
    // Support sleeping on tests too so users know it works
    if let Some(delay) = payload.delay_seconds {
        if delay > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(delay as u64)).await;
        }
    }

    let (status, body) = execute_action(
        &payload.url,
        &payload.method,
        payload.body_template.as_deref(),
        payload.headers.as_ref(),
        &payload.auth_type,
        payload.auth_data.as_ref(),
        "TEST_PLATE_123"
    ).await?;

    Ok(Json(TestActionResponse { status, body }))
}

#[utoipa::path(
    get,
    path = "/api/custom-actions",
    responses(
        (status = 200, description = "Retrieved list of custom actions", body = [CustomActionResponse]),
        (status = 401, description = "Unauthorized", body = AppErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "Custom Actions"
)]
pub async fn get_custom_actions(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<Json<Vec<CustomActionResponse>>, AppError> {
    let records = sqlx::query!(
        r#"
        SELECT id, name, method, url, auth_type, headers, body_template, delay_seconds 
        FROM custom_actions
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let actions = records.into_iter().map(|rec| {
        let headers_val = rec.headers.and_then(|h| serde_json::from_str(&h).ok());
        
        CustomActionResponse {
            id: rec.id,
            name: rec.name,
            method: rec.method,
            url: rec.url,
            auth_type: rec.auth_type,
            headers: headers_val,
            body_template: rec.body_template,
            delay_seconds: rec.delay_seconds,
        }
    }).collect();

    Ok(Json(actions))
}

#[utoipa::path(
    delete,
    path = "/api/custom-actions/{id}",
    params(
        ("id" = i64, Path, description = "Custom Action ID")
    ),
    responses(
        (status = 200, description = "Custom action deleted successfully"),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "Custom Actions"
)]
pub async fn delete_custom_action(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query!("DELETE FROM custom_actions WHERE id = ?", id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "success": true, "message": "Action deleted" })))
}