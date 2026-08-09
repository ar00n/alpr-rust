use axum::{extract::State, Extension, Json};

use crate::{
    error::{AppError, AppErrorResponse},
    models::{
        UpdateFrameratePayload, UpdateMinConfidencePayload, UpdateRTSPUrlPayload,
        UpdateTrimHistoryPayload, UpdateTrimSnapshotsPayload, User,
    },
    rtsp::validate_rtsp_url,
    state::AppState,
};

async fn update_db_setting(
    db: &sqlx::Pool<sqlx::Sqlite>,
    key: &str,
    value: Option<String>,
) -> Result<(), AppError> {
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
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(())
}

#[utoipa::path(
    put,
    path = "/api/settings/framerate",
    request_body = UpdateFrameratePayload,
    responses(
        (status = 200, description = "Framerate updated successfully", body = Object),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden (Admin only)", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
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
) -> Result<Json<serde_json::Value>, AppError> {
    update_db_setting(
        &state.db,
        "processing_framerate",
        Some(payload.framerate.to_string()),
    )
    .await?;

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
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden (Admin only)", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn get_framerate(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<Json<UpdateFrameratePayload>, AppError> {
    let framerate = state.pipeline_config_tx.borrow().fps;
    Ok(Json(UpdateFrameratePayload { framerate }))
}

#[utoipa::path(
    put,
    path = "/api/settings/rtsp_url",
    request_body = UpdateRTSPUrlPayload,
    responses(
        (status = 200, description = "RTSP URL updated successfully", body = Object),
        (status = 400, description = "Invalid or unreachable RTSP URL", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden (Admin only)", body = AppErrorResponse),
        (status = 500, description = "Database or Internal Error", body = AppErrorResponse)
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
) -> Result<Json<serde_json::Value>, AppError> {
    let url_to_check = payload.rtsp_url.clone();

    if let Some(url) = url_to_check {
        validate_rtsp_url(url, 5)
            .await
            .map_err(|e| AppError::bad_request(format!("RTSP Validation Failed: {}", e)))?;
    }

    update_db_setting(&state.db, "rtsp_url", payload.rtsp_url.clone()).await?;

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
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden (Admin only)", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn get_rtsp_url(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<Json<UpdateRTSPUrlPayload>, AppError> {
    let rtsp_url = state.pipeline_config_tx.borrow().rtsp_url.clone();
    Ok(Json(UpdateRTSPUrlPayload { rtsp_url }))
}

#[utoipa::path(
    put,
    path = "/api/settings/trim_snapshots",
    request_body = UpdateTrimSnapshotsPayload,
    responses(
        (status = 200, description = "Snapshot MB trim limit updated successfully", body = Object),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden (Admin only)", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
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
) -> Result<Json<serde_json::Value>, AppError> {
    let value_str = payload.trim_snapshots_mb.map(|v| v.to_string());

    update_db_setting(&state.db, "trim_snapshots_mb", value_str).await?;

    state.pipeline_config_tx.send_modify(|config| {
        config.trim_snapshots_mb = payload.trim_snapshots_mb;
    });

    Ok(Json(serde_json::json!({
        "status": "success",
        "trim_snapshots_mb": payload.trim_snapshots_mb
    })))
}

#[utoipa::path(
    get,
    path = "/api/settings/trim_snapshots",
    responses(
        (status = 200, description = "Current snapshot MB trim limit", body = UpdateTrimSnapshotsPayload),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden (Admin only)", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn get_trim_snapshots(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<Json<UpdateTrimSnapshotsPayload>, AppError> {
    let trim_snapshots_mb = state.pipeline_config_tx.borrow().trim_snapshots_mb;
    Ok(Json(UpdateTrimSnapshotsPayload { trim_snapshots_mb }))
}

#[utoipa::path(
    put,
    path = "/api/settings/trim_history",
    request_body = UpdateTrimHistoryPayload,
    responses(
        (status = 200, description = "History days trim limit updated successfully", body = Object),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden (Admin only)", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
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
) -> Result<Json<serde_json::Value>, AppError> {
    let value_str = payload.trim_history_days.map(|v| v.to_string());

    update_db_setting(&state.db, "trim_history_days", value_str).await?;

    state.pipeline_config_tx.send_modify(|config| {
        config.trim_history_days = payload.trim_history_days;
    });

    Ok(Json(serde_json::json!({
        "status": "success",
        "trim_history_days": payload.trim_history_days
    })))
}

#[utoipa::path(
    get,
    path = "/api/settings/trim_history",
    responses(
        (status = 200, description = "Current history days trim limit", body = UpdateTrimHistoryPayload),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Admin only)", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn get_trim_history(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<Json<UpdateTrimHistoryPayload>, AppError> {
    let trim_history_days = state.pipeline_config_tx.borrow().trim_history_days;
    Ok(Json(UpdateTrimHistoryPayload { trim_history_days }))
}

#[utoipa::path(
    put,
    path = "/api/settings/min-confidence",
    request_body = UpdateMinConfidencePayload,
    responses(
        (status = 200, description = "Minimum confidence updated successfully", body = Object),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden (Admin only)", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn update_min_confidence(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
    Json(payload): Json<UpdateMinConfidencePayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    update_db_setting(
        &state.db,
        "min_confidence",
        Some(payload.min_confidence.to_string()),
    )
    .await?;

    state.pipeline_config_tx.send_modify(|config| {
        config.min_confidence = payload.min_confidence;
    });

    Ok(Json(
        serde_json::json!({"status": "success", "min_confidence": payload.min_confidence}),
    ))
}

#[utoipa::path(
    get,
    path = "/api/settings/min-confidence",
    responses(
        (status = 200, description = "Current processing framerate", body = UpdateMinConfidencePayload),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden (Admin only)", body = AppErrorResponse),
        (status = 500, description = "Database error", body = AppErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn get_min_confidence(
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<Json<UpdateMinConfidencePayload>, AppError> {
    let min_confidence = state.pipeline_config_tx.borrow().min_confidence;
    Ok(Json(UpdateMinConfidencePayload { min_confidence }))
}
