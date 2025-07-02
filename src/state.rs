use crate::{config::Config, db::DatabaseState};
use std::sync::Arc;
use tokio::sync::broadcast;

// Importar WebSocketMessage desde main.rs o definirlo aquí
#[derive(Clone, Debug)]
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseState>,
    pub config: Arc<Config>,
    pub ws_tx: broadcast::Sender<String>,
}

impl AppState {
    pub fn new(
        db: Arc<DatabaseState>,
        config: Arc<Config>,
        ws_tx: broadcast::Sender<String>,
    ) -> Self {
        Self { db, config, ws_tx }
    }
}
