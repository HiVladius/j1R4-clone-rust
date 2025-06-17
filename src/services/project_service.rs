// Este servicio contendrá la lógica de negocio relacionada con los proyectos.
use chrono::Utc;
use futures::{/*stream::StreamExt*/ StreamExt, TryStreamExt};
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
    models::{
        user_model::{User, UserData},

        project_models::{CreateProjectSchema, Project, UpdateProjectSchema, AddMemberSchema}
    },
    services::permission_service::PermissionService,
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
            members: vec![], // El creador es el primer miembro
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
    pub async fn update_project(
        &self,
        project_id: ObjectId,
        owner_id: ObjectId,
        schema: UpdateProjectSchema,
    ) -> Result<Project, AppError> {
        // 1. Validar el esquema de entrada
        schema
            .validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // 2. Buscar el proyecto para asegurarse de que existe y pertenece al usuario
        let project = self
            .projects_collection()
            .find_one(doc! { "_id": project_id })
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Proyecto no encontrado.".to_string()))?;

        // 3. ¡Verificación de permisos!
        if project.owner_id != owner_id {
            return Err(AppError::Unauthorized(
                "No tienes permiso para modificar este proyecto.".to_string(),
            ));
        }

        // 4. Construir el documento de actualización solo con los campos presentes
        let mut update_doc = doc! {};
        if let Some(name) = schema.name {
            update_doc.insert("name", name);
        }
        if let Some(description) = schema.description {
            update_doc.insert("description", description);
        }

        // Si no hay nada que actualizar, devolvemos el proyecto tal cual
        if update_doc.is_empty() {
            return Ok(project);
        }

        update_doc.insert("updated_at", Utc::now());

        // 5. Realizar la actualización
        let update_result = self
            .projects_collection()
            .find_one_and_update(doc! { "_id": project_id }, doc! { "$set": update_doc })
            .with_options(
                mongodb::options::FindOneAndUpdateOptions::builder()
                    .return_document(mongodb::options::ReturnDocument::After)
                    .build(),
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        // Devolver el documento actualizado
        update_result.ok_or_else(|| {
            AppError::NotFound(
                "No se pudo encontrar el proyecto después de actualizar.".to_string(),
            )
        })
    }

    pub async fn delete_project(
        &self,
        project_id: ObjectId,
        owner_id: ObjectId,
    ) -> Result<(), AppError> {
        // //* 1. Buscar para verificar propiedad
        let project = self
            .projects_collection()
            .find_one(doc! { "_id": project_id })
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Proyecto no encontrado.".to_string()))?;

        // //* 2. ¡Verificación de permisos!
        if project.owner_id != owner_id {
            return Err(AppError::Unauthorized(
                "No tienes permiso para eliminar este proyecto.".to_string(),
            ));
        }

        // //* 3. Eliminar el proyecto
        self.projects_collection()
            .delete_one(doc! { "_id": project_id })
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }
    // //! 4. Agregar miembro al proyecto
    // //! Agrega un miembro al proyecto.
    pub async fn add_member(
     &self,
        project_id: ObjectId,
        owner_id: ObjectId,
        schema: AddMemberSchema,   
    ) -> Result<(), AppError> {
        schema.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;

        PermissionService::new(self.db_state.get_db())
            .is_project_owner(project_id, owner_id)
            .await?;

        let user_to_add = self.db_state.get_db().collection::<User>("users")
            .find_one(doc! { "email": &schema.email })
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Usuario no encontrado.".to_string()))?;

        self.projects_collection()
            .update_one(
                doc! { "_id": project_id },
                doc! { "$addToSet": { "members": user_to_add.id.unwrap() } },
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;


        Ok(())
    }

    pub async fn list_members(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
    )-> Result<Vec<UserData>, AppError>{

        let project = PermissionService::new(self.db_state.get_db())
            .can_access_project(project_id, user_id)
            .await?;

        ////! 2. Recopilar todos los ID's (dueño y miembros)        
        let mut user_ids = project.members;
        user_ids.push(project.owner_id);

        ////! 3. Buscar todos los documentos de usuario que coincidan con los IDs
        let users_collection = self.db_state.get_db().collection::<User>("users");
        let filter = doc! { "_id": { "$in": user_ids } };
        let mut cursor = users_collection
            .find(filter)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        ////! 4 Mapear los resultados a UserData para la respuesta de la API
        let mut members_data = Vec::new();
        while let Some(user) = cursor.next().await {
            if let Ok(user) = user {
                members_data.push(user.into());
            }
        }


        Ok(members_data)
    }


    pub async fn remove_member(
        &self,
        project_id: ObjectId,
        owner_id: ObjectId,
        member_id_to_remove: ObjectId,

    ) -> Result<(), AppError>{

        PermissionService::new(self.db_state.get_db())
            .is_project_owner(project_id, owner_id)
            .await?;

        if owner_id == member_id_to_remove{
            return Err(AppError::ValidationError("El dueño del proyecto no puede ser eliminado.".to_string()));
        }

        let update_result = self.projects_collection()
            .update_one(
                doc! { "_id": project_id },
                doc! { "$pull": { "members": member_id_to_remove } },
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        if update_result.modified_count == 0 {
            return Err(AppError::NotFound("Miembro no encontrado en el proyecto.".to_string()));
        }
        Ok(())
    }


}
