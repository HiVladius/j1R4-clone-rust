use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId; // Solo importamos ObjectId
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub password_hash: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Validate, Debug)]
pub struct RegisterUserSchema {
    #[validate(length(
        min = 5,
        message = "El nombre de usuario debe tener al menos 5 caracteres."
    ))]
    pub username: String,
    #[validate(email(message = "El correo electrónico no es válido."))]
    pub email: String,
    #[validate(length(min = 8, message = "La contraseña debe tener al menos 8 caracteres."))]
    pub password: String,
}

#[derive(Deserialize, Validate, Debug)]
pub struct LoginUserSchema {
    #[validate(email(message = "El correo electrónico no es válido."))]
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Debug)]
pub struct UserData {
    pub id: String,
    pub username: String,
    pub email: String,
}

impl From<User> for UserData {
    fn from(user: User) -> Self {
        UserData {
            id: user.id.map_or_else(String::new, |oid| oid.to_hex()),
            username: user.username,
            email: user.email,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserData,
}
