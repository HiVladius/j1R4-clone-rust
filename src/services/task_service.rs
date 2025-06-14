use chrono::Utc;
use futures::StreamExt;
use mongodb::{
    Collection,
    bson::{DateTime, doc, oid::ObjectId},
};
use std::sync::Arc;
use validator::Validate;

use crate::{
    db::DatabaseState,
    errors::AppError,
    models::{
        project_models::Project,
        task_model::{CreateTaskSchema, Task, TaskPriority, TaskStatus, UpdateTaskSchema},
    },
};

use bson::to_bson;

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
    pub async fn get_task_for_project(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<Vec<Task>, AppError> {
        // Verificacion de permisos
        let project = self
            .projects_collection()
            .find_one(doc! {"_id": project_id})
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("El proyuecto no existe".to_string()))?;

        if project.owner_id != user_id {
            return Err(AppError::Unauthorized(
                "No tienes permiso para ver las tareas de este proyecto".to_string(),
            ));
        }

        // Buscar todas las tareas que coincidan con el project_id
        let filter = doc! {"project_id": project_id};
        let mut cursor = self
            .task_collection()
            .find(filter)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let mut tasks = Vec::new();
        while let Some(result) = cursor.next().await {
            if let Ok(task) = result {
                tasks.push(task);
            }
        }
        Ok(tasks)
    }

    pub async fn get_task_by_id(
        &self,
        task_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<Task, AppError> {
        // Buscar la tarea por ID
        let task = self
            .task_collection()
            .find_one(doc! {"_id": task_id})
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Tarea no encontrada".to_string()))?;

        let project = self
            .projects_collection()
            .find_one(doc! {"_id": task.project_id})
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Proyecto no encontrado".to_string()))?;

        if project.owner_id != user_id {
            return Err(AppError::Unauthorized(
                "No tienes permiso para ver esta tarea".to_string(),
            ));
        }

        Ok(task)
    }

    pub async fn update_task(
        &self,
        task_id: ObjectId,
        user_id: ObjectId,
        schema: UpdateTaskSchema,
    ) -> Result<Task, AppError> {
        schema
            .validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // Verificar si la tarea existe
        let task = self.get_task_by_id(task_id, user_id).await?;

        let mut update_doc = doc! {};

        if let Some(title) = schema.title {
            update_doc.insert("title", title);
        }
        if let Some(description) = schema.description {
            update_doc.insert("description", description);
        }
        if let Some(status) = schema.status {
            update_doc.insert("status", to_bson(&status).unwrap());
        }
        if let Some(priority) = schema.priority {
            update_doc.insert("priority", to_bson(&priority).unwrap());
        }
        if let Some(assignee_opt) = schema.assignee_id {
            let assignee_id = match assignee_opt {
                Some(id_str) => Some(ObjectId::parse_str(&id_str).map_err(|_| {
                    AppError::ValidationError("El ID del asignado no es válido".to_string())
                })?),
                None => None,
            };
            update_doc.insert("assignee_id", assignee_id);
        }

        if update_doc.is_empty() {
            return Ok(task);
        }
        update_doc.insert("updated_at", DateTime::from_chrono(Utc::now()));

        self.task_collection()
            .update_one(doc! {"_id": task_id}, doc! {"$set": update_doc})
            .await
            .map_err(|_| AppError::InternalServerError)?;

        self.get_task_by_id(task_id, user_id).await
    }

    pub async fn delete_task(&self, task_id: ObjectId, user_id: ObjectId) -> Result<(), AppError> {

        // 1.- verificar que la tarea existe y que el usuario tiene permisos
        self.get_task_by_id(task_id, user_id).await?;

        // 2.- Eliminar la tarea

        let result = self
            .task_collection()
            .delete_one(doc! {"_id": task_id})
            .await
            .map_err(|_| AppError::InternalServerError)?;

        if result.deleted_count == 0 {
            return Err(AppError::NotFound("Tarea no encontrada".to_string()));
        }


        Ok(())
    }
}
