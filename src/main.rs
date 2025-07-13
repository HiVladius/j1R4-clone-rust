use dotenvy::dotenv;
use std::sync::Arc;
use tokio::sync::broadcast;
use shuttle_runtime::SecretStore;

use jira_clone_backend::config::Config;
use jira_clone_backend::db::DatabaseState;
use jira_clone_backend::router::router::get_app;
use jira_clone_backend::state::AppState;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    dotenv().ok();

    tracing::info!("Starting Jira Clone Backend...");

    let config = Arc::new(Config::from_secrets(&secrets).expect("Error al cargar la configuración"));

    let db = DatabaseState::init(&config.database_url, &config.database_name).await
        .map_err(|e| shuttle_runtime::Error::Custom(anyhow::Error::msg(format!("Database error: {}", e))))?;
    let db_state = Arc::new(db);

    let (ws_tx, _) = broadcast::channel(100);

    // Crear el estado compartido de la aplicación
    let app_state = Arc::new(AppState::new(db_state.clone(), config.clone(), ws_tx));

    // Define a middleware layer for auth_guard using the app_state
    let app = get_app(app_state);

    tracing::info!("Aplicación configurada correctamente para Shuttle");

    Ok(app.into())
}
