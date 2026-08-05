use axum::extract::{
    ws::{Message, WebSocket, WebSocketUpgrade},
    State,
};

use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/ws",
    responses(
        (status = 101, description = "Switching protocols to WebSocket connection")
    ),
    tag = "WebSocket"
)]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
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
