use crate::{error::{AppError, AppErrorResponse}, state::AppState};
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
                    let encode_task = tokio::task::spawn_blocking(move || {
                        let img = image::load_from_memory(&image_bytes)
                            .map_err(|_| "Failed to decode image")?;

                        let mut config = webp::WebPConfig::new()
                            .map_err(|_| "Failed to initialize WebPConfig")?;
                        
                        config.method = 1; // 0 = fastest, 1 = fast (low CPU), 6 = slowest (max compression)
                        config.quality = 10.0; // Quality factor from 0.0 to 100.0

                        let encoder = webp::Encoder::from_image(&img)
                            .map_err(|_| "Failed to create WebP encoder")?;

                        let webp_mem = encoder.encode_advanced(&config)
                            .map_err(|_| "WebP encoding failed")?;

                        Ok::<Vec<u8>, &'static str>(webp_mem.to_vec())
                    });

                    match encode_task.await {
                        Ok(Ok(webp_bytes)) => {
                            let mut chunk = format!(
                                "--frame\r\nContent-Type: image/webp\r\nContent-Length: {}\r\n\r\n",
                                webp_bytes.len()
                            ).into_bytes();

                            chunk.extend_from_slice(&webp_bytes);
                            chunk.extend_from_slice(b"\r\n");

                            yield Ok::<_, std::io::Error>(chunk);
                        }
                        _ => {
                            // Skip frame on decode/encode failure or thread panic
                            continue;
                        }
                    }
                }
                Ok(None) => break,
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => break,
            }
        }
    };

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "multipart/x-mixed-replace; boundary=frame")
        .body(Body::from_stream(stream))
        .map_err(|e| AppError::internal(format!("Failed to build stream: {}", e)))?;

    Ok(response)
}