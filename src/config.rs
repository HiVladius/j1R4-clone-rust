use serde::Deserialize;
use shuttle_runtime::SecretStore;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub database_name: String,
    pub jwt_secret: String,
    pub server_address: String,
    pub cors_origins: Vec<String>,
    pub gcs_bucket_name: String,
    pub gcs_project_id: String,
    pub google_application_credentials: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://127.0.0.1:3000,http://localhost:5173,http://127.0.0.1:5173".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            database_name: env::var("DATABASE_NAME")?,
            jwt_secret: env::var("JWT_SECRET")?,
            server_address: env::var("SERVER_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1:8000".to_string()),
            cors_origins,
            gcs_bucket_name: env::var("GCS_BUCKET_NAME")?,
            gcs_project_id: env::var("GCS_PROJECT_ID")?,
            google_application_credentials: env::var("GOOGLE_APPLICATION_CREDENTIALS").ok(),
        })
    }

    pub fn from_secrets(secrets: &SecretStore) -> Result<Self, env::VarError> {
        let cors_origins = secrets
            .get("CORS_ORIGINS")
            .unwrap_or_else(|| "http://localhost:3000,http://127.0.0.1:3000,http://localhost:5173,http://127.0.0.1:5173".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Self {
            database_url: secrets
                .get("DATABASE_URL")
                .ok_or(env::VarError::NotPresent)?,
            database_name: secrets
                .get("DATABASE_NAME")
                .ok_or(env::VarError::NotPresent)?,
            jwt_secret: secrets.get("JWT_SECRET").ok_or(env::VarError::NotPresent)?,
            server_address: secrets
                .get("SERVER_ADDRESS")
                .unwrap_or_else(|| "127.0.0.1:8000".to_string()),
            cors_origins,
            gcs_bucket_name: secrets.get("GCS_BUCKET_NAME").ok_or(env::VarError::NotPresent)?,
            gcs_project_id: secrets.get("GCS_PROJECT_ID").ok_or(env::VarError::NotPresent)?,
            google_application_credentials: secrets.get("GOOGLE_APPLICATION_CREDENTIALS"),
        })
    }
}
