use chrono::Utc;
use mongodb::{Collection, bson::doc};
use std::sync::Arc;
use validator::Validate;

use crate::{
    config::Config,
    db::DatabaseState,
    errors::AppError,
    models::user_model::{LoginResponse, LoginUserSchema, RegisterUserSchema, User, UserData},
    utils::{jwt_utils::generate_jwt, password_utils},
};

pub struct AuthService {
    db_state: Arc<DatabaseState>,
    config: Arc<Config>,
}

impl AuthService {
    pub fn new(db_state: Arc<DatabaseState>, config: Arc<Config>) -> Self {
        Self { db_state, config }
    }

    fn user_collection(&self) -> Collection<User> {
        self.db_state.get_db().collection("users")
    }

    pub async fn register_user(&self, schema: RegisterUserSchema) -> Result<UserData, AppError> {
        schema
            .validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // Verificar si ya existe un usuario con el mismo email o username
        let existing_user = self
            .user_collection()
            .find_one(doc! {
                "$or": [
                    { "email": &schema.email },
                    { "username": &schema.username }
                ]
            })
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if existing_user.is_some() {
            return Err(AppError::ValidationError(
                "El correo electrónico o nombre de usuario ya está en uso".to_string(),
            ));
        }

        let password_hash = password_utils::hash_password(&schema.password)?;

        let new_user = User {
            id: None, // MongoDB generará automáticamente el _id
            username: schema.username.clone(),
            email: schema.email.clone(),
            password_hash,
            first_name: schema.first_name.clone(),
            last_name: schema.last_name.clone(),
            bio: schema.bio.clone(),
            role: schema.role.clone(),
            avatar: schema.avatar.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let insert_result = self
            .user_collection()
            .insert_one(&new_user)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Crear el usuario con el ID generado por MongoDB
        let created_user = User {
            id: insert_result.inserted_id.as_object_id(),
            username: new_user.username,
            email: new_user.email,
            password_hash: new_user.password_hash,
            first_name: schema.first_name,
            last_name: schema.last_name,
            bio: schema.bio,
            role: schema.role,
            avatar: schema.avatar,
            created_at: new_user.created_at,
            updated_at: new_user.updated_at,
        };

        Ok(created_user.into())
    }

    pub async fn login_user(&self, schema: LoginUserSchema) -> Result<LoginResponse, AppError> {
        schema
            .validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        //Buscar el usuario por correo
        let user = self
            .user_collection()
            .find_one(doc! { "email": &schema.email })
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .ok_or(AppError::ValidationError(
                "Usuario no encontrado".to_string(),
            ))?;

        // Verificar que el usuario tenga un hash de contraseña
        if user.password_hash.is_empty() {
            return Err(AppError::ValidationError(
                "Usuario sin contraseña configurada".to_string(),
            ));
        }
        // //!Verificar la contraseña y generar el token JWT
        if !password_utils::verify_password(&user.password_hash, &schema.password)? {
            return Err(AppError::ValidationError(
                "Contraseña incorrecta".to_string(),
            ));
        }

        let user_id = user.id.ok_or_else(|| AppError::InternalServerError)?;

        let token = generate_jwt(&user_id, &self.config)?;

        Ok(LoginResponse {
            token,
            user: user.into(),
        })
    }
}
