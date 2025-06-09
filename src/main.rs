use dotenvy::dotenv;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use jira_clone_backend::config::Config;
use jira_clone_backend::db::DatabaseState;
use jira_clone_backend::errors::AppError;
use jira_clone_backend::router::router::get_app;
use jira_clone_backend::state::AppState;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive("jira_clone_backend=debug".parse()?))
        .with(fmt::layer())
        .init();

    tracing::info!("Starting Jira Clone Backend...");

    let config = Arc::new(Config::from_env().expect("Error al cargar la configuración"));
    // tracing::info!("Configuración cargada: {:?}", config);

    let server_address = config.server_address.clone();

    let db = DatabaseState::init(&config.database_url, &config.database_name).await?;
    let db_state = Arc::new(db);

    // Crear el estado compartido de la aplicación
    let app_state = Arc::new(AppState::new(db_state.clone(), config.clone()));

    // Define a middleware layer for auth_guard using the app_state
   let app = get_app(app_state);

   // Parse the server address and bind the listener
    tracing::info!("Escuchando en: {}", server_address);
   let addr: SocketAddr = server_address
        .parse()
        .expect("No se pudo parsear la direccion del servidor");

    let listener: TcpListener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(())
}


