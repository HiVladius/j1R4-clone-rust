use chrono::Utc;
use mongodb::{Collection, bson::{doc, oid::ObjectId}};
use std::sync::Arc;
use validator::Validate;

use crate::{
    config::Config,
    db::DatabaseState,
    errors::AppError,
    models::user_model::{LoginResponse, LoginUserSchema, RegisterUserSchema, UpdateUserSchema, User, UserData, Role},
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
            id: None,
            username: schema.username.clone(),
            email: schema.email.clone(),
            password_hash,
            first_name: Some(schema.first_name.clone()),
            last_name: Some(schema.last_name.clone()),
            bio: schema.bio.clone(),
            role: Some(schema.role.unwrap_or(Role::Member)),
            avatar: schema.avatar.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let insert_result = self
            .user_collection()
            .insert_one(&new_user)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let created_user = User {
            id: insert_result.inserted_id.as_object_id(),
            ..new_user
        };

        Ok(created_user.into())
    }

    pub async fn login_user(&self, schema: LoginUserSchema) -> Result<LoginResponse, AppError> {
        schema
            .validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        let user = self
            .user_collection()
            .find_one(doc! { "email": &schema.email })
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .ok_or(AppError::ValidationError(
                "Usuario no encontrado".to_string(),
            ))?;

        if user.password_hash.is_empty() {
            return Err(AppError::ValidationError(
                "Usuario sin contraseña configurada".to_string(),
            ));
        }

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

    pub async fn update_user_details(&self, user_id: &ObjectId, update_data: UpdateUserSchema) -> Result<UserData, AppError> {
        let mut update_doc = doc! {};

        if let Some(username) = update_data.username {
            update_doc.insert("username", username);
        }
        if let Some(email) = update_data.email {
            update_doc.insert("email", email);
        }
        if let Some(first_name) = update_data.first_name {
            update_doc.insert("first_name", first_name);
        }
        if let Some(last_name) = update_data.last_name {
            update_doc.insert("last_name", last_name);
        }
        if let Some(bio) = update_data.bio {
            update_doc.insert("bio", bio);
        }
        if let Some(role) = update_data.role {
            update_doc.insert("role", serde_json::to_string(&role).unwrap());
        }
        if let Some(avatar) = update_data.avatar {
            update_doc.insert("avatar", avatar);
        }

        if !update_doc.is_empty() {
            update_doc.insert("updated_at", chrono::Utc::now());
            self.user_collection()
                .update_one(doc! { "_id": user_id }, doc! { "$set": update_doc })
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        let updated_user = self.user_collection()
            .find_one(doc! { "_id": user_id })
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .ok_or(AppError::NotFound("Usuario no encontrado después de actualizar".to_string()))?;

        Ok(updated_user.into())
    }
}
