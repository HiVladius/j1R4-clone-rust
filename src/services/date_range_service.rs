use mongodb::{
    Collection,
    bson::{doc, oid::ObjectId},
};
use std::sync::Arc;
use validator::Validate;

use crate::{
    db::DatabaseState, errors::AppError, models::task_model::DateRange,
    services::permission_service::PermissionService,
};
use crate::models::task_model::Task;

pub struct DateRangeService {
    db_state: Arc<DatabaseState>,
}

impl DateRangeService {
    pub fn new(db_state: Arc<DatabaseState>) -> Self {
        Self { db_state }
    }

    fn date_range_collection(&self) -> Collection<DateRange> {
        self.db_state
            .get_db()
            .collection::<DateRange>("task_date_ranges")
    }

    /// Crear o actualizar el rango de fechas para una tarea
    pub async fn set_task_date_range(
        &self,
        task_id: ObjectId,
        date_range: DateRange,
        user_id: ObjectId,
    ) -> Result<DateRange, AppError> {
        date_range
            .validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // Verificar que el usuario tiene permisos para acceder a la tarea
        let task_collection = self
            .db_state
            .get_db()
            .collection::<Task>("tasks");

        let task = task_collection
            .find_one(doc! {"_id": task_id})
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Tarea no encontrada".to_string()))?;

        // Verificar permisos del proyecto
        PermissionService::new(self.db_state.get_db())
            .can_access_project(task.project_id, user_id)
            .await?;

        // Verificar si ya existe un rango de fechas para esta tarea
        let existing_range = self
            .date_range_collection()
            .find_one(doc! {"task_id": task_id})
            .await
            .map_err(|_| AppError::InternalServerError)?;

        if let Some(_) = existing_range {
            // Actualizar el rango existente
            let update_doc = doc! {
                "$set": {
                    "start_date": date_range.start_date,
                    "end_date": date_range.end_date,
                }
            };

            self.date_range_collection()
                .update_one(doc! {"task_id": task_id}, update_doc)
                .await
                .map_err(|_| AppError::InternalServerError)?;
        } else {
            // Crear nuevo rango
            self.date_range_collection()
                .insert_one(&date_range)
                .await
                .map_err(|_| AppError::InternalServerError)?;
        }

        Ok(date_range)
    }

    /// Obtener el rango de fechas para una tarea específica
    pub async fn get_task_date_range(
        &self,
        task_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<Option<DateRange>, AppError> {
        // Verificar que el usuario tiene permisos para acceder a la tarea
        let task_collection = self
            .db_state
            .get_db()
            .collection::<Task>("tasks");

        let task = task_collection
            .find_one(doc! {"_id": task_id})
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Tarea no encontrada".to_string()))?;

        // Verificar permisos del proyecto
        PermissionService::new(self.db_state.get_db())
            .can_access_project(task.project_id, user_id)
            .await?;

        // Buscar el rango de fechas
        let date_range = self
            .date_range_collection()
            .find_one(doc! {"task_id": task_id})
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(date_range)
    }

    /// Obtener todos los rangos de fechas para las tareas de un proyecto
    pub async fn get_project_date_ranges(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<Vec<DateRange>, AppError> {
        // Verificar permisos del proyecto
        PermissionService::new(self.db_state.get_db())
            .can_access_project(project_id, user_id)
            .await?;

        // Obtener todas las tareas del proyecto
        let task_collection = self
            .db_state
            .get_db()
            .collection::<Task>("tasks");
        let mut cursor = task_collection
            .find(doc! {"project_id": project_id})
            .await
            .map_err(|_| AppError::InternalServerError)?;

        use futures::StreamExt;
        let mut task_ids = Vec::new();
        while let Some(result) = cursor.next().await {
            if let Ok(task) = result {
                if let Some(task_id) = task.id {
                    task_ids.push(task_id);
                }
            }
        }

        // Obtener los rangos de fechas para estas tareas
        let mut date_ranges = Vec::new();
        if !task_ids.is_empty() {
            let filter = doc! {"task_id": {"$in": task_ids}};
            let mut cursor = self
                .date_range_collection()
                .find(filter)
                .await
                .map_err(|_| AppError::InternalServerError)?;

            while let Some(result) = cursor.next().await {
                if let Ok(date_range) = result {
                    date_ranges.push(date_range);
                }
            }
        }

        Ok(date_ranges)
    }

    /// Eliminar el rango de fechas para una tarea
    pub async fn delete_task_date_range(
        &self,
        task_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<(), AppError> {
        // Verificar que el usuario tiene permisos para acceder a la tarea
        let task_collection = self
            .db_state
            .get_db()
            .collection::<Task>("tasks");

        let task = task_collection
            .find_one(doc! {"_id": task_id})
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Tarea no encontrada".to_string()))?;

        // Verificar permisos del proyecto
        PermissionService::new(self.db_state.get_db())
            .can_access_project(task.project_id, user_id)
            .await?;

        // Eliminar el rango de fechas
        self.date_range_collection()
            .delete_one(doc! {"task_id": task_id})
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }

    /// Actualizar parcialmente el rango de fechas para una tarea
    pub async fn update_task_date_range(
        &self,
        task_id: ObjectId,
        update_data: crate::models::task_model::UpdateDateRangeSchema,
        user_id: ObjectId,
    ) -> Result<DateRange, AppError> {
        // Verificar que el usuario tiene permisos para acceder a la tarea
        let task_collection = self
            .db_state
            .get_db()
            .collection::<Task>("tasks");

        let task = task_collection
            .find_one(doc! {"_id": task_id})
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Tarea no encontrada".to_string()))?;

        // Verificar permisos del proyecto
        PermissionService::new(self.db_state.get_db())
            .can_access_project(task.project_id, user_id)
            .await?;

        // Verificar si existe un rango de fechas para esta tarea
        let existing_range = self
            .date_range_collection()
            .find_one(doc! {"task_id": task_id})
            .await
            .map_err(|_| AppError::InternalServerError)?;

        if existing_range.is_none() {
            return Err(AppError::NotFound("Rango de fechas no encontrado para esta tarea".to_string()));
        }

        // Construir el documento de actualización
        let mut update_doc = doc! {};
        
        if let Some(start_date) = update_data.start_date {
            update_doc.insert("start_date", start_date);
        }
        
        if let Some(end_date) = update_data.end_date {
            update_doc.insert("end_date", end_date);
        }

        if update_doc.is_empty() {
            return Err(AppError::ValidationError("No se proporcionaron campos para actualizar".to_string()));
        }

        // Actualizar el documento
        self.date_range_collection()
            .update_one(
                doc! {"task_id": task_id},
                doc! {"$set": update_doc}
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        // Obtener y retornar el documento actualizado
        let updated_range = self
            .date_range_collection()
            .find_one(doc! {"task_id": task_id})
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::InternalServerError)?;

        Ok(updated_range)
    }
}
