use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use validator::Validate;

// Representación de un Proyecto en la base de datos
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(rename = "key")] // Usar 'key' en la DB
    pub project_key: String, // 'key' es una palabra reservada en Rust
    pub description: Option<String>,
    pub owner_id: ObjectId,
    #[serde(default)]
    pub members: Vec<ObjectId>, // Lista de IDs de miembros del proyecto
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Validate, Debug)]
pub struct CreateProjectSchema {
    #[validate(length(
        min = 3,
        message = "El nombre del proyecto debe tener al menos 3 caracteres."
    ))]
    pub name: String,

    #[validate(length(
        min = 2,
        max = 10,
        message = "La clave del proyecto debe tener entre 2 y 10 caracteres."
    ))]
    #[validate(regex(
        path = "crate::utils::validation::KEY_REGEX",
        message = "La clave solo puede contener letras mayúsculas y números."
    ))]
    pub key: String,

    pub description: Option<String>,
}

#[derive(Deserialize, Validate, Debug, Default)]
pub struct UpdateProjectSchema {
    #[validate(length(
        min = 3,
        message = "El nombre del proyecto debe de tener al menos 3 caracteres. "
    ))]
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Validate, Debug)]
pub struct AddMemberSchema {
    #[validate(email(message = "El correo electrónico no es valido"))]
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UserRole {
    #[serde(rename = "owner")]
    Owner,
    #[serde(rename = "member")]
    Member,
}

// Proyecto con información del rol del usuario autenticado
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectWithRole {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(rename = "key")]
    pub project_key: String,
    pub description: Option<String>,
    pub owner_id: ObjectId,
    #[serde(default)]
    pub members: Vec<ObjectId>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
    pub user_role: UserRole,
}

impl ProjectWithRole {
    pub fn from_project(project: Project, user_id: ObjectId) -> Self {
        let user_role = if project.owner_id == user_id {
            UserRole::Owner
        } else {
            UserRole::Member
        };

        Self {
            id: project.id,
            name: project.name,
            project_key: project.project_key,
            description: project.description,
            owner_id: project.owner_id,
            members: project.members,
            created_at: project.created_at,
            updated_at: project.updated_at,
            user_role,
        }
    }
}
