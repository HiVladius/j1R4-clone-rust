use mongodb::{bson::doc, Collection};
use std::sync::Arc;
use chrono::Utc;
use validator::Validate;

use crate::{
    db::DatabaseState,
    models::user_model::{User, RegisterUserSchema, LoginUserSchema, UserData, LoginResponse},
    utils::{password_utils, jwt_utils::{generate_jwt}},
    errors::AppError,
    config::Config,
};

pub struct AuthService {
    db_state: Arc<DatabaseState>,
    config: Arc<Config>,
}

impl AuthService{

    pub fn new(db_state: Arc<DatabaseState>, config: Arc<Config>) -> Self {
        Self {
            db_state,
            config,
        }
    }

    fn user_collection(&self) -> Collection<User> {
        self.db_state.get_db().collection("users")
    }

    pub async fn register_user(&self, schema: RegisterUserSchema) -> Result<UserData, AppError> {

        schema.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;

        let existing_user = self.user_collection()
            .find_one(doc! { "email": &schema.email })
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if existing_user.is_some() {
            return Err(AppError::ValidationError("El correo / nombre de usuario ya esta en uso".to_string()));
        }

        let password_hash = password_utils::hash_password(&schema.password)?;

        let new_user = User {
            id: None,
            username: schema.username.clone(),
            email: schema.email.clone(),
            password_hash,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let insert_result = self.user_collection()
            .insert_one(new_user.clone())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let create_user = User {
            id: insert_result.inserted_id.as_object_id(),
            ..new_user
        };
        Ok(create_user.into())
    }

    pub async fn login_user(&self, schema: LoginUserSchema) -> Result<LoginResponse, AppError> {
        schema.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;

        //Buscar el usuario por correo
        let user = self.user_collection()
            .find_one(doc! { "email": &schema.email })
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .ok_or(AppError::ValidationError("Usuario no encontrado".to_string()))?;

        if !password_utils::verify_password(&schema.password, &user.password_hash)? {
            return Err(AppError::ValidationError("Contraseña incorrecta".to_string()));
        }

        let user_id = user.id.ok_or_else(|| AppError::InternalServerError)?;

        let token = generate_jwt(&user_id, &self.config)?;

        Ok(LoginResponse {
            token,
            user: user.into(),
        })
    }

}