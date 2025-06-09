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
    // Más adelante podríamos añadir 'members: Vec<ObjectId>'
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
