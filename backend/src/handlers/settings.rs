use axum::{extract::State, http::StatusCode, Extension, Json};

use crate::{
    models::{
        UpdateFrameratePayload, UpdateRTSPUrlPayload, UpdateTrimHistoryPayload, 
        UpdateTrimSnapshotsPayload, User
    }, 
    rtsp::validate_rtsp_url, 
    state::AppState,
};

/// Helper function to reduce boilerplate when updating a single setting in the DB
async fn update_db_setting(
    db: &sqlx::Pool<sqlx::Sqlite>,
    key: &str,
    value: Option<String>,
) -> Result<(), (StatusCode, String)> {
    sqlx::query!(
        r#"
            INSERT INTO settings (key, value)
            VALUES (?, ?)
            ON CONFLICT(key)
            DO UPDATE SET value = excluded.value
        "#,
        key,
        value
    )
    .execute(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(())
}

#[utoipa::path(
    put,
    path = "/api/settings/framerate",
    request_body = UpdateFrameratePayload,
    responses(
        (status = 200, description = "Framerate updated successfully", body = Object),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Admin only)"),
        (status = 500, description = "Database error", body = String)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn update_framerate(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
    Json(payload): Json<UpdateFrameratePayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    update_db_setting(
        &state.db, 
        "processing_framerate", 
        Some(payload.framerate.to_string())
    ).await?;

    state.pipeline_config_tx.send_modify(|config| {
        config.fps = payload.framerate;
    });

    Ok(Json(
        serde_json::json!({"status": "success", "framerate": payload.framerate}),
    ))
}

#[utoipa::path(
    get,
    path = "/api/settings/framerate",
    responses(
        (status = 200, description = "Current processing framerate", body = UpdateFrameratePayload),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Admin only)"),
        (status = 500, description = "Database error", body = String)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn get_framerate(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<Json<UpdateFrameratePayload>, (StatusCode, String)> {
    let framerate = state.pipeline_config_tx.borrow().fps;
    Ok(Json(UpdateFrameratePayload { framerate }))
}

#[utoipa::path(
    put,
    path = "/api/settings/rtsp_url",
    request_body = UpdateRTSPUrlPayload,
    responses(
        (status = 200, description = "RTSP URL updated successfully", body = Object),
        (status = 400, description = "Invalid or unreachable RTSP URL", body = String),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Admin only)"),
        (status = 500, description = "Database or Internal Error", body = String)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn update_rtsp_url(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
    Json(payload): Json<UpdateRTSPUrlPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let url_to_check = payload.rtsp_url.clone();
    
    if let Some(ref url) = url_to_check {
        let url_clone = url.clone();
        tokio::task::spawn_blocking(move || validate_rtsp_url(&url_clone, 5))
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("RTSP Validation Failed: {}", e)))?;
    }

    // 2. Save to DB if validation succeeded
    update_db_setting(&state.db, "rtsp_url", payload.rtsp_url.clone()).await?;

    // 3. Update background pipeline config
    state.pipeline_config_tx.send_modify(|config| {
        config.rtsp_url = payload.rtsp_url.clone();
    });

    Ok(Json(
        serde_json::json!({"status": "success", "rtsp_url": payload.rtsp_url}),
    ))
}

#[utoipa::path(
    get,
    path = "/api/settings/rtsp_url",
    responses(
        (status = 200, description = "Current RTSP URL", body = UpdateRTSPUrlPayload),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Admin only)"),
        (status = 500, description = "Database error", body = String)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn get_rtsp_url(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<Json<UpdateRTSPUrlPayload>, (StatusCode, String)> {
    let rtsp_url = state.pipeline_config_tx.borrow().rtsp_url.clone();
    Ok(Json(UpdateRTSPUrlPayload { rtsp_url }))
}

#[utoipa::path(
    put,
    path = "/api/settings/trim_snapshots",
    request_body = UpdateTrimSnapshotsPayload,
    responses(
        (status = 200, description = "Snapshot MB trim limit updated successfully", body = Object),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Admin only)"),
        (status = 500, description = "Database error", body = String)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn update_trim_snapshots(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
    Json(payload): Json<UpdateTrimSnapshotsPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let value_str = payload.trim_snapshots_mb.map(|v| v.to_string());

    update_db_setting(&state.db, "trim_snapshots_mb", value_str).await?;

    state.pipeline_config_tx.send_modify(|config| {
        config.trim_snapshots_mb = payload.trim_snapshots_mb;
    });

    Ok(Json(
        serde_json::json!({
            "status": "success",
            "trim_snapshots_mb": payload.trim_snapshots_mb
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/settings/trim_snapshots",
    responses(
        (status = 200, description = "Current snapshot MB trim limit", body = UpdateTrimSnapshotsPayload),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Admin only)"),
        (status = 500, description = "Database error", body = String)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn get_trim_snapshots(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<Json<UpdateTrimSnapshotsPayload>, (StatusCode, String)> {
    let trim_snapshots_mb = state.pipeline_config_tx.borrow().trim_snapshots_mb;
    Ok(Json(UpdateTrimSnapshotsPayload { trim_snapshots_mb }))
}

#[utoipa::path(
    put,
    path = "/api/settings/trim_history",
    request_body = UpdateTrimHistoryPayload,
    responses(
        (status = 200, description = "History days trim limit updated successfully", body = Object),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Admin only)"),
        (status = 500, description = "Database error", body = String)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn update_trim_history(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
    Json(payload): Json<UpdateTrimHistoryPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let value_str = payload.trim_history_days.map(|v| v.to_string());

    update_db_setting(&state.db, "trim_history_days", value_str).await?;

    state.pipeline_config_tx.send_modify(|config| {
        config.trim_history_days = payload.trim_history_days;
    });

    Ok(Json(
        serde_json::json!({
            "status": "success",
            "trim_history_days": payload.trim_history_days
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/settings/trim_history",
    responses(
        (status = 200, description = "Current history days trim limit", body = UpdateTrimHistoryPayload),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Admin only)"),
        (status = 500, description = "Database error", body = String)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn get_trim_history(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<Json<UpdateTrimHistoryPayload>, (StatusCode, String)> {
    let trim_history_days = state.pipeline_config_tx.borrow().trim_history_days;
    Ok(Json(UpdateTrimHistoryPayload { trim_history_days }))
}