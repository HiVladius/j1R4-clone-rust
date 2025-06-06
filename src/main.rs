use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Router, Extension,
};
use dotenvy::dotenv;
use jira_clone_backend::config::Config;
use jira_clone_backend::db::DatabaseState;
use jira_clone_backend::errors::AppError;
use jira_clone_backend::handlers::auth_handler::get_me_handler;
use jira_clone_backend::handlers::auth_handler::{login_handler, register_handler};
use jira_clone_backend::middleware::auth_middleware::auth_guard;
use jira_clone_backend::state::AppState;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive("jira_clone_backend=debug".parse()?))
        .with(fmt::layer())
        .init();

    tracing::info!("Starting Jira Clone Backend...");

    let config = Arc::new(Config::from_env().expect("Error al cargar la configuración"));
    tracing::info!("Configuración cargada: {:?}", config);

    let server_address = config.server_address.clone();

    let db = DatabaseState::init(&config.database_url, &config.database_name).await?;
    let db_state = Arc::new(db);

    // Crear el estado compartido de la aplicación
    let app_state = Arc::new(AppState::new(db_state.clone(), config.clone()));

    // Define a middleware layer for auth_guard using the app_state
    let auth_middleware = middleware::from_fn_with_state(
        app_state.clone(),
        auth_guard
    );

    // Define routes that require authentication
    let protected_routes = Router::new()
        .route("/me", get(get_me_handler))
        .layer(auth_middleware)
        .with_state(app_state.clone());

    let auth_routes = Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler));

    let app = Router::new()
        .route("/", get(root_handler))
        .nest("/api/auth", auth_routes)
        .nest("/api", protected_routes)
        .with_state(app_state);

    let add_str = &server_address;
    let addr: SocketAddr = add_str
        .parse()
        .expect("No se pudo parsear la dirección del servidor");

    tracing::info!("Servidor escuchando en {}", addr);

    let listener: TcpListener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|e| AppError::from(e))?;

    Ok(())
}

async fn root_handler() -> &'static str {
    "¡Bienvenido al Backend del clon de Jira en Rust!"
}
