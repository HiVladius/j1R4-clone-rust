use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Image {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub filename: String,
    pub original_filename: String,
    pub content_type: String,
    pub size: u64,
    pub gcs_url: String,
    pub gcs_bucket: String,
    pub gcs_object_name: String,
    pub uploaded_by: ObjectId,
    pub project_id: Option<ObjectId>,
    pub task_id: Option<ObjectId>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ImageUploadQuery {
    pub project_id: Option<String>,
    pub task_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ImageResponse {
    pub id: String,
    pub filename: String,
    pub original_filename: String,
    pub content_type: String,
    pub size: u64,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Image> for ImageResponse {
    fn from(image: Image) -> Self {
        Self {
            id: image.id.unwrap().to_hex(),
            filename: image.filename,
            original_filename: image.original_filename,
            content_type: image.content_type,
            size: image.size,
            url: image.gcs_url,
            created_at: image.created_at,
            updated_at: image.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateImageSchema {
    pub filename: Option<String>,
    pub project_id: Option<String>,
    pub task_id: Option<String>,
}
