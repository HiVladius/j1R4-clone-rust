use chrono::Utc;
use mongodb::{
    Collection,
    bson::{doc, oid::ObjectId},
};
use std::sync::Arc;
use validator::Validate;

use crate::{
    db::DatabaseState,
    errors::AppError,
    models::{
        project_models::Project,
        task_model::{CreateTaskSchema, Task, TaskPriority, TaskStatus},
    },
};

pub struct TaskService {
    db_state: Arc<DatabaseState>,
}

impl TaskService {
    pub fn new(db_state: Arc<DatabaseState>) -> Self {
        Self { db_state }
    }

    fn task_collection(&self) -> Collection<Task> {
        self.db_state.get_db().collection::<Task>("tasks")
    }

    fn projects_collection(&self) -> Collection<Project> {
        self.db_state.get_db().collection::<Project>("projects")
    }

    pub async fn create_task(
        &self,
        schema: CreateTaskSchema,
        project_id: ObjectId,
        reporter_id: ObjectId,
    ) -> Result<Task, AppError> {
        schema
            .validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        let project = self
            .projects_collection()
            .find_one(doc! {"_id": project_id})
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        if project.owner_id != reporter_id {
            return Err(AppError::Unauthorized(
                "No tienes permiso para crear tareas en este proyecto".to_string(),
            ));
        }

        let assignee_id = schema
            .assignee_id
            .map(|id_str| ObjectId::parse_str(&id_str))
            .transpose()
            .map_err(|_| {
                AppError::ValidationError("El ID del asignado no es válido".to_string())
            })?;

        let mut new_task = Task {
            id: None,
            project_id,
            title: schema.title,
            description: schema.description,
            status: schema.status.unwrap_or(TaskStatus::ToDo),
            priority: schema.priority.unwrap_or(TaskPriority::Medium),
            assignee_id,
            reporter_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = self
            .task_collection()
            .insert_one(&new_task)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        new_task.id = result.inserted_id.as_object_id();

        if new_task.id.is_none() {
            return Err(AppError::InternalServerError);
        }

        Ok(new_task)
    }
}
