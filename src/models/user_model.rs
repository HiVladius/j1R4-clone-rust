use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Role {
    Admin,
    ProjectManager,
    Member,
    Viewer,
}

impl Default for Role {
    fn default() -> Self {
        Role::Member
    }
}

fn default_none_string() -> Option<String> {
    None
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub password_hash: String,
    #[serde(default = "default_none_string")]
    pub first_name: Option<String>,
    #[serde(default = "default_none_string")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Validate, Debug)]
pub struct RegisterUserSchema {
    #[validate(length(min = 5,message = "El nombre de usuario debe tener al menos 5 caracteres."))]
    pub username: String,
    #[validate(email(message = "El correo electrónico no es válido."))]
    pub email: String,
    #[validate(length(min = 8, message = "La contraseña debe tener al menos 8 caracteres."))]
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub bio: Option<String>,
    pub role: Option<Role>,
    pub avatar: Option<String>,
}

#[derive(Deserialize, Validate, Debug, Default)]
pub struct UpdateUserSchema {
    pub username: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub bio: Option<String>,
    pub role: Option<Role>,
    pub avatar: Option<String>,
}


#[derive(Deserialize, Validate, Debug)]
pub struct LoginUserSchema {
    #[validate(email(message = "El correo electrónico no es válido."))]
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserData {
    pub id: String,
    pub username: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub bio: Option<String>,
    pub role: Option<Role>,
    pub avatar: Option<String>,
}

impl From<User> for UserData {
    fn from(user: User) -> Self {
        UserData {
            id: user.id.map_or_else(String::new, |oid| oid.to_hex()),
            username: user.username,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            bio: user.bio,
            role: user.role,
            avatar: user.avatar,
        }
    }
}

#[derive(Serialize, Debug, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserData,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct UserLoginResponseTest {
    pub token: String,
    pub user: UserData,
}
