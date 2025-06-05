use axum::{Router, routing::get};
use dotenvy::dotenv;
use jira_clone_backend::config::Config;
use jira_clone_backend::db::DatabaseState;
use jira_clone_backend::errors::AppError;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().ok();

    let env_filter = EnvFilter::from_default_env().add_directive(
        "jira_clone_backend=debug"
            .parse()
            .expect("Error al parsear la directiva de filtro"),
    );

    //Configurar tracing
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .init();

    tracing::info!("Iniciando la aplicación...");

    // Cargar la configuración
    let config = Config::from_env().expect("Error al cargar la configuración");
    tracing::info!("Configuración cargada: {:?}", config);

    let db = DatabaseState::init(&config.database_url, &config.database_name).await?;
    let db_state = Arc::new(db);

    //Definir una ruta de prueba
    let app = Router::new()
        .route("/", get(root_handler))
        .with_state(db_state);

    // Iniciar el servidor
    let addr_str = config
        .server_address
        .strip_prefix("http://")
        .or_else(|| config.server_address.strip_prefix("https://"))
        .unwrap_or(&config.server_address);

    // Convertir localhost a 127.0.0.1 para que sea compatible con SocketAddr
    let addr_str = if addr_str.starts_with("localhost:") {
        addr_str.replace("localhost", "127.0.0.1")
    } else {
        addr_str.to_string()
    };

    let addr: SocketAddr = addr_str.parse().map_err(|e| {
        tracing::error!(
            "Error parseando dirección del servidor '{}': {}",
            addr_str,
            e
        );
        AppError::InternalServerError
    })?;

    tracing::info!("Servidor escuchando en {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(AppError::from)?;
    Ok(())
}

async fn root_handler() -> &'static str {
    "¡Bienvenido al Backend del clon de Jira en Rust!"
}
