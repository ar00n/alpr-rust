use crate::{
    error::{AppError, AppErrorResponse},
    state::AppState,
};
use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use tokio::sync::broadcast::error::RecvError;

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
            description = "Unauthorized",
            body = AppErrorResponse
        ),
        (
            status = 500,
            description = "Failed to build stream or internal server error",
            body = AppErrorResponse
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

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(Some(image_bytes)) => {
                    let mut chunk = format!(
                        "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                        image_bytes.len()
                    ).into_bytes();

                    chunk.extend_from_slice(&image_bytes);
                    chunk.extend_from_slice(b"\r\n");

                    yield Ok::<_, std::io::Error>(chunk);
                }
                Ok(None) => break,
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => break,
            }
        }
    };

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "multipart/x-mixed-replace; boundary=frame",
        )
        .body(Body::from_stream(stream))
        .map_err(|e| AppError::internal(format!("Failed to build stream: {}", e)))?;

    Ok(response)
}
