use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TaskStatus {
    ToDo,
    InProgress,
    Done,
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub project_id: ObjectId,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_id: Option<ObjectId>,
    pub reporter_id: ObjectId,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Validate, Debug)]
pub struct CreateTaskSchema {
    #[validate(length(min = 3, message = "El titulo debe tener al menos 3 caracteres"))]
    pub title: String,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub assignee_id: Option<String>,
}

#[derive(Deserialize, Validate, Debug, Default)]
pub struct UpdateTaskSchema {
    #[validate(length(min = 3, message = "El titulo debe tener al menos 3 caracteres"))]
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub assignee_id: Option<Option<String>>,
}
