use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Extension,
};
use std::path::PathBuf;
use tokio::fs;

use crate::{models::User, state::AppState};

#[utoipa::path(
    get,
    path = "/api/snapshot/{id}",
    params(
        ("id" = String, Path, description = "Snapshot file ID or filename")
    ),
    responses(
        (status = 200, description = "Snapshot image", content_type = "image/png"),
        (status = 400, description = "Invalid image ID"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Snapshot not found"),
        (status = 500, description = "Failed to read file")
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
) -> Result<Response, StatusCode> {
    let safe_filename = std::path::Path::new(&id)
        .file_name()
        .ok_or(StatusCode::BAD_REQUEST)?;

    let filepath = PathBuf::from("snapshots").join(safe_filename);

    if !filepath.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let image_bytes = fs::read(&filepath)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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