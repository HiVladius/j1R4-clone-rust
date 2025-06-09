use crate::{config::Config, db::DatabaseState};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseState>,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(db: Arc<DatabaseState>, config: Arc<Config>) -> Self {
        Self { db, config }
    }
}

