use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};

use crate::state::AppState;
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    let mut rx = state.ws_tx.subscribe();

    tracing::info!("Nueva conexión WebSocket establecida");

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            tracing::debug!("Enviando mensaje WebSocket: {}", msg);
            if sender.send(Message::Text(msg.into())).await.is_err() {
                tracing::warn!("Error enviando mensaje WebSocket, cerrando conexión");
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    tracing::debug!("Mensaje recibido del cliente: {}", text);
                    // Aquí puedes manejar mensajes del cliente si es necesario
                }
                Ok(Message::Close(_)) => {
                    tracing::info!("Cliente cerró la conexión WebSocket");
                    break;
                }
                Ok(Message::Ping(_data)) => {
                    tracing::debug!("Ping recibido");
                    // Los pings se manejan automáticamente
                }
                Ok(Message::Pong(_)) => {
                    tracing::debug!("Pong recibido");
                }
                Err(e) => {
                    tracing::error!("Error en WebSocket: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => {
            tracing::info!("Send task terminada, cerrando recv task");
            recv_task.abort();
        },
        _ = (&mut recv_task) => {
            tracing::info!("Recv task terminada, cerrando send task");
            send_task.abort();
        },
    }
    tracing::info!("WebSocket connection closed");
}
