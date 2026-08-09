use axum::extract::{
    ws::{Message, WebSocket, WebSocketUpgrade},
    Query, State,
};
use axum::response::IntoResponse;
use jsonwebtoken::{decode, Algorithm, Validation};
use serde::Deserialize;

use crate::{
    error::{AppError, AppErrorResponse},
    models::{Claims, User},
    state::AppState,
};

#[derive(Deserialize)]
pub struct WsAuthQuery {
    token: String,
}

#[utoipa::path(
    get,
    path = "/api/ws",
    params(
        ("token" = String, Query, description = "JWT Bearer Token for WebSocket Auth")
    ),
    responses(
        (status = 101, description = "Switching protocols to WebSocket connection"),
        (status = 401, description = "Unauthorized", body = AppErrorResponse)
    ),
    tag = "WebSocket"
)]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsAuthQuery>,
    State(state): State<AppState>,
) -> Result<axum::response::Response, AppError> {
    let mut validation = Validation::default();
    validation.algorithms = vec![Algorithm::EdDSA];

    let token_data = decode::<Claims>(&query.token, &state.jwt.decoding_key, &validation)
        .map_err(|e| AppError::unauthorized(e.to_string()))?;

    let claims = token_data.claims;

    let _user = sqlx::query_as!(User, "SELECT * FROM users WHERE username = ?", &claims.sub)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?
        .ok_or(AppError::unauthorized("User not found"))?;

    Ok(ws
        .on_upgrade(move |socket| handle_websocket(socket, state))
        .into_response())
}

async fn handle_websocket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.plate_tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        if let Ok(json_text) = serde_json::to_string(&msg) {
            if socket.send(Message::Text(json_text.into())).await.is_err() {
                break;
            }
        }
    }
}
