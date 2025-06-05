use crate::config::Config; // Para acceder al JWT_SECRET
use crate::errors::AppError;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject, typically the user ID
    pub exp: i64,           // Expiration time as a timestamp
    pub iat: i64,           // Issued at time as a timestamp
    pub roles: Vec<String>, // Roles associated with the user
}


pub fn generate_jwt(user_id: &ObjectId, config: &Config) -> Result<String, AppError> {
    let now = Utc::now();
    let iat = now.timestamp(); // i64
    let exp = (now + Duration::days(1)).timestamp(); // i64

    let claims = Claims{
        sub: user_id.to_hex(),
        exp,
        iat,
        roles: vec!["user".to_string()], // Default role, can be extended later
    };

    let header = Header::new(Algorithm::HS256);
    encode(&header, &claims, &EncodingKey::from_secret(config.jwt_secret.as_ref()))
        .map_err(|e| {
            tracing::error!("Error al generar JWT: {}", e);
            AppError::JwtError(e)
        })
}

pub fn verify_jwt(token: &str, config: &Config) -> Result<Claims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true; // Validar expiración del token

    decode::<Claims>(token, &DecodingKey::from_secret(config.jwt_secret.as_ref()), &validation)
        .map(|data| data.claims)
        .map_err(|e| {
            tracing::error!("Error al verificar JWT: {}", e);
            AppError::JwtError(e)
        })

}