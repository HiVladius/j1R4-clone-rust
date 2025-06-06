// src/errors.rs
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Error de base de datos: {0}")]
    DatabaseError(String),

    #[error("Error de conexión a la base de datos: {0}")]
    DatabaseConnectionError(String),

    #[error("Error de configuración: {0}")]
    ConfigError(#[from] std::env::VarError),

    #[error("Error de parsing de directiva de log: {0}")]
    LogDirectiveParseError(#[from] tracing_subscriber::filter::ParseError),

    #[error("Error al iniciar el listener TCP: {0}")]
    TcpListenerError(#[from] std::io::Error), // Cubre errores de bind y serve

    #[error("Error de validación: {0}")]
    ValidationError(String),

    #[error("Recurso no encontrado: {0}")]
    NotFound(String),

    #[error("Error de autenticación: {0}")]
    AuthError(String),

    #[error("No autorizado: {0}")]
    Unauthorized(String),

    #[error("Error interno del servidor")]
    InternalServerError,

    #[error("Error de JWT: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
}

// Cómo Axum convierte AppError en una respuesta HTTP
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(msg) | AppError::DatabaseConnectionError(msg) => {
                tracing::error!("Error de base de datos: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error interno del servidor (DB)".to_string(),
                )
            }
            AppError::ConfigError(e) => {
                tracing::error!("Error de configuración: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error de configuración del servidor".to_string(),
                )
            }
            AppError::LogDirectiveParseError(e) => {
                tracing::error!("Error de parsing de directiva de log: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error de logging del servidor".to_string(),
                )
            }
            AppError::TcpListenerError(e) => {
                tracing::error!("Error de listener TCP: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error al iniciar el servidor".to_string(),
                )
            }
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::AuthError(msg) => (StatusCode::UNAUTHORIZED, msg), // O BAD_REQUEST dependiendo del contexto
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error interno del servidor".to_string(),
            ),
            AppError::JwtError(e) => {
                tracing::error!("Error de JWT: {}", e);
                (
                    StatusCode::UNAUTHORIZED,
                    "Token inválido o expirado".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

// Si usas anyhow directamente en main.rs y quieres un helper:
// pub type Result<T> = std::result::Result<T, anyhow::Error>;
// Si usas AppError:
pub type Result<T, E = AppError> = std::result::Result<T, E>;
