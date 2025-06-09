// src/db.rs
use crate::errors::AppError;
use mongodb::{Client, Database, options::ClientOptions}; // Asegúrate de tener definido AppError

#[derive(Clone)] // Para poder clonar y pasar el estado a los handlers de Axum
pub struct DatabaseState {
    pub client: Client,
    pub db: Database,
}

impl DatabaseState {
    pub async fn init(db_url: &str, db_name: &str) -> Result<Self, AppError> {
        // tracing::info!("Conectando a la base de datos MongoDB en {}...", db_url);
        let mut client_options = ClientOptions::parse(db_url).await.map_err(|e| {
            tracing::error!("Error al parsear la URL de MongoDB: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

        // Opcional: configurar el nombre de la aplicación para monitoreo
        client_options.app_name = Some("JiraCloneBackendRust".to_string());

        let client = Client::with_options(client_options).map_err(|e| {
            tracing::error!("Error al crear el cliente de MongoDB: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

        // Ping para verificar la conexión
        client
            .database("admin")
            .run_command(mongodb::bson::doc! {"ping": 1})
            .await
            .map_err(|e| {
                tracing::error!("Error al hacer ping a la base de datos: {}", e);
                AppError::DatabaseConnectionError(e.to_string())
            })?;

        tracing::info!("Conexión a MongoDB establecida exitosamente.");

        let db = client.database(db_name);
        Ok(Self { client, db })
    }

    pub fn get_db(&self) -> &Database {
        &self.db
    }
}
