use crate::{error::AppError, state::AppState};
use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

#[utoipa::path(
    get,
    path = "/api/stream",
    responses(
        (
            status = 200,
            description = "MJPEG Stream initialized successfully",
            content_type = "multipart/x-mixed-replace; boundary=frame",
            body = [u8]
        ),
        (
            status = 401,
            description = "Unauthorized"
        ),
        (
            status = 500,
            description = "Failed to build stream or internal server error",
            body = String
        )
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn mjpeg_stream_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let mut rx = state.rtsp_tx.subscribe();

    // Use async-stream to yield chunks as they arrive from GStreamer
    let stream = async_stream::stream! {
        while let Ok(image_bytes) = rx.recv().await {
            let mut chunk = format!(
                "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                image_bytes.len()
            ).into_bytes();

            chunk.extend_from_slice(&image_bytes);
            chunk.extend_from_slice(b"\r\n");

            yield Ok::<_, std::io::Error>(chunk);
        }
    };

    let response = Response::builder()
        .status(StatusCode::OK)
        // x-mixed-replace tells the browser to replace the previous image
        .header(
            header::CONTENT_TYPE,
            "multipart/x-mixed-replace; boundary=frame",
        )
        .body(Body::from_stream(stream))
        .map_err(|e| {
            AppError::internal(
                format!("Failed to build stream: {}", e),
            )
        })?;

    Ok(response)
}
