use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use crate::errors::AppError;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(password_hash) => Ok(password_hash.to_string()),
        Err(e) => {
            tracing::error!("Error al hashear la contraseña:{}", e);
            Err(AppError::InternalServerError)
        }
    }
}

pub fn verify_password(hashed_password: &str, password_to_verify: &str) -> Result<bool, AppError> {
    match PasswordHash::new(hashed_password) {
        Ok(parsed_hash) => Ok(Argon2::default()
            .verify_password(password_to_verify.as_bytes(), &parsed_hash)
            .is_ok()),
        Err(e) => {
            tracing::error!("Error al parsear el hash de la contraseña: {}", e);
            Err(AppError::InternalServerError)
        }
    }
}
