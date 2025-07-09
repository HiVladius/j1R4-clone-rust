use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use validator::Validate;

// Helper module para serialización de Option<DateTime>
mod datetime_option {
    use super::*;
    use mongodb::bson::DateTime as BsonDateTime;

    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(dt) => {
                let bson_dt = BsonDateTime::from_chrono(*dt);
                bson_dt.serialize(serializer)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<BsonDateTime> = Option::deserialize(deserializer)?;
        Ok(opt.map(|bson_dt| bson_dt.to_chrono()))
    }
}

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
    // Nuevos campos para fechas
    #[serde(
        with = "datetime_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(
        with = "datetime_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub end_date: Option<DateTime<Utc>>,
    pub has_due_date: bool, // Campo para activar/desactivar fecha de finalización
}

#[derive(Deserialize, Validate, Debug)]
pub struct CreateTaskSchema {
    #[validate(length(min = 3, message = "El titulo debe tener al menos 3 caracteres"))]
    pub title: String,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub assignee_id: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub has_due_date: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,    // Fecha de creación manual
    pub updated_at: Option<DateTime<Utc>>,    // Fecha de actualización manual
}

#[derive(Deserialize, Validate, Debug, Default)]
pub struct UpdateTaskSchema {
    #[validate(length(min = 3, message = "El titulo debe tener al menos 3 caracteres"))]
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub assignee_id: Option<Option<String>>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<Option<DateTime<Utc>>>, // Option<Option<>> permite remover la fecha
    pub has_due_date: Option<bool>,
    pub updated_at: Option<DateTime<Utc>>,        // Fecha de actualización manual
}

impl CreateTaskSchema {
    /// Valida que las fechas sean consistentes
    pub fn validate_dates(&self) -> Result<(), String> {
        // Si has_due_date es true, debe haber una end_date
        if self.has_due_date.unwrap_or(false) && self.end_date.is_none() {
            return Err("La fecha de finalización es requerida cuando has_due_date es true".to_string());
        }

        // Si has_due_date es false, no debe haber end_date
        if !self.has_due_date.unwrap_or(false) && self.end_date.is_some() {
            return Err("No se puede establecer fecha de finalización cuando has_due_date es false".to_string());
        }

        // Si ambas fechas están presentes, start_date debe ser anterior a end_date
        if let (Some(start), Some(end)) = (&self.start_date, &self.end_date) {
            if start >= end {
                return Err("La fecha de inicio debe ser anterior a la fecha de finalización".to_string());
            }
        }

        Ok(())
    }
}

impl UpdateTaskSchema {
    /// Valida que las fechas sean consistentes en una actualización
    pub fn validate_dates(&self, current_task: &Task) -> Result<(), String> {
        let has_due_date = self.has_due_date.unwrap_or(current_task.has_due_date);
        let start_date = self.start_date.or(current_task.start_date);
        let end_date = self.end_date.as_ref().map_or(current_task.end_date, |opt| *opt);

        // Si has_due_date es true, debe haber una end_date
        if has_due_date && end_date.is_none() {
            return Err("La fecha de finalización es requerida cuando has_due_date es true".to_string());
        }

        // Si has_due_date es false, no debe haber end_date
        if !has_due_date && end_date.is_some() {
            return Err("No se puede establecer fecha de finalización cuando has_due_date es false".to_string());
        }

        // Si ambas fechas están presentes, start_date debe ser anterior a end_date
        if let (Some(start), Some(end)) = (start_date, end_date) {
            if start >= end {
                return Err("La fecha de inicio debe ser anterior a la fecha de finalización".to_string());
            }
        }

        Ok(())
    }
}

/// Funciones auxiliares básicas para fechas
impl Task {
    /// Verifica si la tarea tiene fechas válidas
    pub fn has_valid_dates(&self) -> bool {
        // Si tiene fecha de inicio y fin, la de inicio debe ser anterior
        if let (Some(start), Some(end)) = (self.start_date, self.end_date) {
            return start < end;
        }
        
        // Si has_due_date es true, debe tener end_date
        if self.has_due_date && self.end_date.is_none() {
            return false;
        }
        
        // Si has_due_date es false, no debe tener end_date
        if !self.has_due_date && self.end_date.is_some() {
            return false;
        }
        
        true
    }
}
