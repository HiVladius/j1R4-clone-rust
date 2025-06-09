// Este servicio contendrá la lógica de negocio relacionada con los proyectos.
use chrono::Utc;
use futures::{/*stream::StreamExt*/ TryStreamExt};
use mongodb::{
    Collection,
    bson::{doc, oid::ObjectId},
    // options::FindOptions,
};
use std::sync::Arc;
use validator::Validate;

use crate::{
    db::DatabaseState,
    errors::AppError,
    models::project_models::{CreateProjectSchema, Project},
};

pub struct ProjectService {
    db_state: Arc<DatabaseState>,
}

impl ProjectService {
    pub fn new(db_state: Arc<DatabaseState>) -> Self {
        Self { db_state }
    }

    fn projects_collection(&self) -> Collection<Project> {
        self.db_state.get_db().collection("projects")
    }

    pub async fn create_project(
        &self,
        schema: CreateProjectSchema,
        owner_id: ObjectId,
    ) -> Result<Project, AppError> {
        // 1. Validar el esquema
        schema
            .validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // 2. Verificar si la clave del proyecto ya existe
        let key_exists = self
            .projects_collection()
            .find_one(doc! { "key": &schema.key })
            .await
            .map_err(|_| AppError::InternalServerError)?
            .is_some();

        if key_exists {
            return Err(AppError::ValidationError(
                "La clave del proyecto ya está en uso.".to_string(),
            ));
        }

        // 3. Crear y guardar el nuevo proyecto
        let new_project = Project {
            id: None,
            name: schema.name,
            project_key: schema.key,
            description: schema.description,
            owner_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = self
            .projects_collection()
            .insert_one(&new_project)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(Project {
            id: result.inserted_id.as_object_id(),
            ..new_project
        })
    }

    pub async fn get_projects_for_user(&self, user_id: ObjectId) -> Result<Vec<Project>, AppError> {
        let filter = doc! { "owner_id": user_id };
        // let options = FindOptions::builder()
        // .sort(doc! { "created_at": -1 })
        // .build();

        let cursor = self.projects_collection().find(filter).await.map_err(|e| {
            tracing::error!("Error al buscar proyectos en la base de datos: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

        let projects = cursor.try_collect().await.map_err(|e| {
            tracing::error!("Error al recolectar proyectos del cursor: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

        Ok(projects)
    }
}
