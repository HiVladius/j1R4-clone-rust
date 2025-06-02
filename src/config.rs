use serde::Deserialize;
use std::env;


#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub database_name: String,
    pub jwt_secret: String,
    pub server_address: String,
}


impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            database_name: env::var("DATABASE_NAME")?,
            jwt_secret: env::var("JWT_SECRET")?,
            server_address: env::var("SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0:1:8000".to_string()),
        })
    }
}