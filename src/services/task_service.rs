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
    services::permission_service::PermissionService
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

    fn _projects_collection(&self) -> Collection<Project> {
        self.db_state.get_db().collection::<Project>("projects")
    }

    // //* Create a new task
    // //* Creates a new task in a project, ensuring the user has permission to do so.
    pub async fn create_task(
        &self,
        schema: CreateTaskSchema,
        project_id: ObjectId,
        reporter_id: ObjectId,
    ) -> Result<Task, AppError> {
        schema
            .validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;


        PermissionService::new(self.db_state.get_db())
            .can_access_project(project_id, reporter_id)
            .await?;

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

    // //* Get all tasks for a project
    // //* Retrieves all tasks associated with a project, ensuring the user has permission to access the project.
    pub async fn get_task_for_project(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<Vec<Task>, AppError> {
        
        // Verificacion de permisos

        PermissionService::new(self.db_state.get_db())
            .can_access_project(project_id, user_id)
            .await?;

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

    // //* Get a task by ID
    // //* Retrieves a task by its ID, ensuring the user has permission to access it.
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

            PermissionService::new(self.db_state.get_db())
            .can_access_project(task.project_id, user_id)
            .await?;

        Ok(task)
    }

    // //* Update a task
    // //* Updates a task by its ID, ensuring the user has permission to update it.
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


    // //* Delete a task
    // //* Deletes a task by its ID, ensuring the user has permission to delete it.
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
