use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade},

    State,},
    response::IntoResponse,
};

use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use crate::state::AppState;


pub async fn websocket_handler(
   ws: WebSocketUpgrade,
   State(state): State<Arc<AppState>>, 
) -> impl IntoResponse  {
    ws.on_upgrade( |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>){
    let (mut sender, mut receiver) = socket.split();

    let mut rx = state.ws_tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            } 
        }
    });

    let mut recv_task = tokio::spawn(async move {

        while let Some(Ok(Message::Close(_))) = receiver.next().await {
            break;
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
    tracing::info!("WebSocket connection closed");


}