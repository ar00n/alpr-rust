use axum::{
    extract::{Path, State},
    http::header,
    response::{IntoResponse, Response},
    Extension,
};
use std::path::PathBuf;
use tokio::fs;

use crate::{
    error::{AppError, AppErrorResponse},
    models::User,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/snapshot/{id}",
    params(
        ("id" = String, Path, description = "Snapshot file ID or filename")
    ),
    responses(
        (status = 200, description = "Snapshot image", content_type = "image/png"),
        (status = 400, description = "Invalid image ID", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 404, description = "Snapshot not found", body = AppErrorResponse),
        (status = 500, description = "Failed to read file", body = AppErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "History"
)]
pub async fn get_snapshot(
    State(_state): State<AppState>,
    Extension(_user): Extension<User>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let safe_filename = std::path::Path::new(&id)
        .file_name()
        .ok_or(AppError::bad_request("Missing filename"))?;

    let filepath = PathBuf::from("snapshots").join(safe_filename);

    if !filepath.exists() {
        return Err(AppError::not_found("File not found"));
    }

    let image_bytes = fs::read(&filepath).await.map_err(|_| {
        AppError::internal(format!("Failed to read file at {}", filepath.display()))
    })?;

    let content_type = "image/webp";

    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        image_bytes,
    )
        .into_response())
}
