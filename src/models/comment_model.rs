use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use validator::Validate;


use super::user_model::UserData;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Comment {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub task_id: ObjectId,
    pub user_id: ObjectId,
    pub content: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Validate, Debug)]
pub struct CreateCommentSchema {
    #[validate(length(min = 1, message = "El comentario no puede estar vacío"))]
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommentData {
    pub id: String,
    pub task_id: String,
    pub author: UserData,
    pub content: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Validate, Debug)]
pub struct UpdateCommentSchema {
    #[validate(length(min = 1, message = "El comentario no puede estar vacío"))]
    pub content: String,
}