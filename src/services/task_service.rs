use bson::to_bson;
use chrono::Utc;
use futures::StreamExt;
use mongodb::{
    Collection,
    bson::{DateTime, doc, oid::ObjectId},
};
use std::sync::Arc;
use tokio::sync::broadcast;
use validator::Validate;

use crate::{
    db::DatabaseState,
    errors::AppError,
    models::{
        project_models::Project,
        task_model::{CreateTaskSchema, Task, TaskPriority, TaskStatus, UpdateTaskSchema},
    },
    services::permission_service::PermissionService,
};

pub struct TaskService {
    db_state: Arc<DatabaseState>,
    ws_tx: broadcast::Sender<String>,
}

impl TaskService {
    pub fn new(db_state: Arc<DatabaseState>, ws_tx: broadcast::Sender<String>) -> Self {
        Self { db_state, ws_tx }
    }

    fn task_collection(&self) -> Collection<Task> {
        self.db_state.get_db().collection::<Task>("tasks")
    }

    fn projects_collection(&self) -> Collection<Project> {
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

        // Validar fechas

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

        // Usar fechas proporcionadas o generar automáticamente si no se proporcionan
        let now = Utc::now();
        let created_at = schema.created_at.unwrap_or(now);
        let updated_at = schema.updated_at.unwrap_or(now);

        let mut new_task = Task {
            id: None,
            project_id,
            title: schema.title,
            description: schema.description,
            status: schema.status.unwrap_or(TaskStatus::ToDo),
            priority: schema.priority.unwrap_or(TaskPriority::Medium),
            assignee_id,
            reporter_id,
            created_at,
            updated_at,
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

        // Emitir mensaje WebSocket para nueva tarea creada
        let broadcast_message = serde_json::json!({
            "event_type": "TASK_CREATED",
            "task": new_task,
        })
        .to_string();

        if let Err(e) = self.ws_tx.send(broadcast_message) {
            tracing::warn!("Error enviando mensaje WebSocket para tarea creada: {}", e);
        } else {
            tracing::info!(
                "Mensaje WebSocket enviado para tarea creada: {}",
                new_task.id.unwrap()
            );
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
        let task = self
            .task_collection()
            .find_one(doc! {"_id": task_id})
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Tarea no encontrada".to_string()))?;

        // Validar fechas con el contexto de la tarea actual

        // Verificar si el usuario puede acceder al proyecto (es dueño o miembro)
        PermissionService::new(self.db_state.get_db())
            .can_access_project(task.project_id, user_id)
            .await?;

        // No es necesario verificar si es dueño o asignado específicamente
        // Cualquier miembro del proyecto puede actualizar tareas

        // Capturar información sobre los cambios antes de mover los valores
        let title_changed = schema.title.is_some();
        let description_changed = schema.description.is_some();
        let status_changed = schema.status.is_some();
        let priority_changed = schema.priority.is_some();
        let assignee_changed = schema.assignee_id.is_some();

        let previous_status = if status_changed {
            Some(task.status.clone())
        } else {
            None
        };

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

        // Usar fecha proporcionada o generar automáticamente
        let updated_at = schema.updated_at.unwrap_or_else(|| Utc::now());
        update_doc.insert("updated_at", DateTime::from_chrono(updated_at));

        self.task_collection()
            .update_one(doc! {"_id": task_id}, doc! {"$set": update_doc})
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let updated_task = self.get_task_by_id(task_id, user_id).await?;

        // nueva logica de broadcast con información detallada de cambios
        let broadcast_message = serde_json::json!({
            "event_type": "TASK_UPDATED",
            "task": updated_task,
            "changes": {
                "status_changed": status_changed,
                "previous_status": previous_status,
                "updated_fields": {
                    "title": title_changed,
                    "description": description_changed,
                    "status": status_changed,
                    "priority": priority_changed,
                    "assignee_id": assignee_changed,
                }
            }
        })
        .to_string();

        if let Err(e) = self.ws_tx.send(broadcast_message) {
            tracing::warn!(
                "Error enviando mensaje WebSocket para tarea actualizada: {}",
                e
            );
        } else {
            tracing::info!(
                "Mensaje WebSocket enviado para tarea actualizada: {} (status_changed: {})",
                task_id,
                status_changed
            );
        }

        Ok(updated_task)
    }

    // //* Delete a task
    // //* Deletes a task by its ID, ensuring the user has permission to delete it.
    pub async fn delete_task(&self, task_id: ObjectId, user_id: ObjectId) -> Result<(), AppError> {
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

        if project.owner_id != user_id && task.assignee_id != Some(user_id) {
            return Err(AppError::Unauthorized(
                "No tienes permiso para eliminar esta tarea".to_string(),
            ));
        }

        // 2.- Eliminar la tarea

        let result = self
            .task_collection()
            .delete_one(doc! {"_id": task_id})
            .await
            .map_err(|_| AppError::InternalServerError)?;

        if result.deleted_count == 0 {
            return Err(AppError::NotFound(
                "No se pudo eliminar la tarea, es posible que ya haya sido eliminada".to_string(),
            ));
        }

        // Emitir mensaje WebSocket para tarea eliminada
        let broadcast_message = serde_json::json!({
            "event_type": "TASK_DELETED",
            "task_id": task_id.to_hex(),
            "project_id": task.project_id.to_hex(),
        })
        .to_string();

        if let Err(e) = self.ws_tx.send(broadcast_message) {
            tracing::warn!(
                "Error enviando mensaje WebSocket para tarea eliminada: {}",
                e
            );
        } else {
            tracing::info!(
                "Mensaje WebSocket enviado para tarea eliminada: {}",
                task_id
            );
        }

        Ok(())
    }
}
