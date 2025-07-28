use std::sync::Arc;
use chrono::Utc;
use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::{
        delete::DeleteObjectRequest,
        get::GetObjectRequest,
        upload::{Media, UploadObjectRequest, UploadType},
    },
};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection,
};
use futures::stream::TryStreamExt;
use uuid::Uuid;

use crate::{
    config::Config,
    db::DatabaseState,
    errors::AppError,
    models::image_model::{Image, ImageResponse, UpdateImageSchema},
};

#[derive(Clone)]
pub struct ImageService {
    collection: Collection<Image>,
    gcs_client: Client,
    config: Arc<Config>,
}

impl ImageService {
    pub async fn new(db: Arc<DatabaseState>, config: Arc<Config>) -> Result<Self, AppError> {
        let collection = db.db.collection::<Image>("images");
        
        // En modo test, no creamos cliente GCS real
        let gcs_client = if cfg!(test) {
            // Para tests, devolvemos un error inmediatamente para evitar autenticación
            return Err(AppError::InternalServerError);
        } else {
            // Para producción, intentamos crear el cliente con autenticación
            match Self::create_gcs_client().await {
                Ok(client) => client,
                Err(e) => {
                    tracing::error!("Error configurando GCS: {}", e);
                    return Err(AppError::InternalServerError);
                }
            }
        };

        Ok(Self {
            collection,
            gcs_client,
            config,
        })
    }

    #[cfg(test)]
    pub fn new_mock(db: Arc<DatabaseState>, config: Arc<Config>) -> Self {
        let collection = db.db.collection::<Image>("images");
        // Para tests, creamos un cliente con configuración por defecto que no se usará
        let gcs_client = Client::new(ClientConfig::default());
        Self {
            collection,
            gcs_client,
            config,
        }
    }

    async fn create_gcs_client() -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
        let client_config = ClientConfig::default()
            .with_auth()
            .await?;
        Ok(Client::new(client_config))
    }

    pub async fn upload_image(
        &self,
        file_data: Vec<u8>,
        filename: String,
        content_type: String,
        user_id: ObjectId,
        project_id: Option<ObjectId>,
        task_id: Option<ObjectId>,
        custom_name: Option<String>, // Nuevo parámetro para nombre personalizado
        folder: Option<String>,      // Nuevo parámetro para carpeta
    ) -> Result<ImageResponse, AppError> {
        let file_size = file_data.len() as u64;
        
        // Generar nombre del archivo (personalizado o UUID)
        let file_extension = filename
            .split('.')
            .next_back()
            .unwrap_or("bin");
            
        let unique_filename = if let Some(custom) = custom_name {
            // Si se proporciona nombre personalizado, usarlo
            format!("{}.{}", custom, file_extension)
        } else {
            // Si no, usar UUID como antes
            format!("{}.{}", Uuid::new_v4(), file_extension)
        };
        
        // Crear el path en GCS con carpeta dinámica
        let folder_name = folder.unwrap_or_else(|| "avatar".to_string());
        let gcs_object_name = format!("{}/{}", folder_name, unique_filename);
        
        // Subir el archivo a Google Cloud Storage
        if !cfg!(test) {
            let upload_type = UploadType::Simple(Media::new(gcs_object_name.clone()));
            let upload_request = UploadObjectRequest {
                bucket: self.config.gcs_bucket_name.clone(),
                ..Default::default()
            };

            self.gcs_client
                .upload_object(&upload_request, file_data, &upload_type)
                .await
                .map_err(|e| {
                    tracing::error!("Error subiendo archivo a GCS: {}", e);
                    AppError::InternalServerError
                })?;
        }

        // Crear la URL pública del archivo
        let gcs_url = if cfg!(test) {
            format!("https://test-storage.example.com/{}", gcs_object_name)
        } else {
            format!(
                "https://storage.googleapis.com/{}/{}",
                self.config.gcs_bucket_name,
                gcs_object_name
            )
        };

        // Crear el documento de imagen en la base de datos
        let now = Utc::now();
        let image = Image {
            id: None,
            filename: unique_filename.clone(),
            original_filename: filename,
            content_type,
            size: file_size,
            gcs_url: gcs_url.clone(),
            gcs_bucket: self.config.gcs_bucket_name.clone(),
            gcs_object_name: gcs_object_name.clone(),
            uploaded_by: user_id,
            project_id,
            task_id,
            created_at: now,
            updated_at: now,
        };

        let result = self.collection.insert_one(&image).await
            .map_err(|e| AppError::DatabaseError(format!("Error guardando imagen: {}", e)))?;

        let inserted_id = result.inserted_id.as_object_id()
            .ok_or_else(|| {
                tracing::error!("Error obteniendo ID de imagen insertada");
                AppError::InternalServerError
            })?;

        let mut saved_image = image;
        saved_image.id = Some(inserted_id);

        Ok(ImageResponse::from(saved_image))
    }

    pub async fn get_image(&self, image_id: ObjectId) -> Result<ImageResponse, AppError> {
        let image = self.collection
            .find_one(doc! { "_id": image_id })
            .await
            .map_err(|e| AppError::DatabaseError(format!("Error buscando imagen: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Imagen no encontrada".to_string()))?;

        Ok(ImageResponse::from(image))
    }

    pub async fn get_image_data(&self, image_id: ObjectId) -> Result<Vec<u8>, AppError> {
        let image = self.collection
            .find_one(doc! { "_id": image_id })
            .await
            .map_err(|e| AppError::DatabaseError(format!("Error buscando imagen: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Imagen no encontrada".to_string()))?;

        // En modo test, retornamos datos mock
        if cfg!(test) {
            return Ok(b"test image data".to_vec());
        }

        // Descargar el archivo de Google Cloud Storage
        let get_request = GetObjectRequest {
            bucket: image.gcs_bucket,
            object: image.gcs_object_name,
            ..Default::default()
        };

        let data = self.gcs_client
            .download_object(&get_request, &Default::default())
            .await
            .map_err(|e| {
                tracing::error!("Error descargando archivo de GCS: {}", e);
                AppError::InternalServerError
            })?;

        Ok(data)
    }

    pub async fn update_image(
        &self,
        image_id: ObjectId,
        update_data: UpdateImageSchema,
        user_id: ObjectId,
    ) -> Result<ImageResponse, AppError> {
        // Verificar que la imagen existe y que el usuario tiene permisos
        let image = self.collection
            .find_one(doc! { "_id": image_id })
            .await
            .map_err(|e| AppError::DatabaseError(format!("Error buscando imagen: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Imagen no encontrada".to_string()))?;

        if image.uploaded_by != user_id {
            return Err(AppError::Unauthorized("No tienes permisos para modificar esta imagen".to_string()));
        }

        let mut update_doc = doc! {
            "updated_at": Utc::now()
        };

        if let Some(filename) = update_data.filename {
            update_doc.insert("filename", filename);
        }

        if let Some(project_id) = update_data.project_id {
            if !project_id.is_empty() {
                let project_oid = ObjectId::parse_str(&project_id)
                    .map_err(|_| AppError::ValidationError("ID de proyecto inválido".to_string()))?;
                update_doc.insert("project_id", project_oid);
            } else {
                update_doc.insert("project_id", mongodb::bson::Bson::Null);
            }
        }

        if let Some(task_id) = update_data.task_id {
            if !task_id.is_empty() {
                let task_oid = ObjectId::parse_str(&task_id)
                    .map_err(|_| AppError::ValidationError("ID de tarea inválido".to_string()))?;
                update_doc.insert("task_id", task_oid);
            } else {
                update_doc.insert("task_id", mongodb::bson::Bson::Null);
            }
        }

        self.collection
            .update_one(doc! { "_id": image_id }, doc! { "$set": update_doc })
            .await
            .map_err(|e| AppError::DatabaseError(format!("Error actualizando imagen: {}", e)))?;

        self.get_image(image_id).await
    }

    pub async fn delete_image(&self, image_id: ObjectId, user_id: ObjectId) -> Result<(), AppError> {
        // Verificar que la imagen existe y que el usuario tiene permisos
        let image = self.collection
            .find_one(doc! { "_id": image_id })
            .await
            .map_err(|e| AppError::DatabaseError(format!("Error buscando imagen: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Imagen no encontrada".to_string()))?;

        if image.uploaded_by != user_id {
            return Err(AppError::Unauthorized("No tienes permisos para eliminar esta imagen".to_string()));
        }

        // Eliminar el archivo de Google Cloud Storage
        if !cfg!(test) {
            let delete_request = DeleteObjectRequest {
                bucket: image.gcs_bucket,
                object: image.gcs_object_name,
                ..Default::default()
            };

            self.gcs_client
                .delete_object(&delete_request)
                .await
                .map_err(|e| {
                    tracing::error!("Error eliminando archivo de GCS: {}", e);
                    AppError::InternalServerError
                })?;
        }

        // Eliminar el documento de la base de datos
        self.collection
            .delete_one(doc! { "_id": image_id })
            .await
            .map_err(|e| AppError::DatabaseError(format!("Error eliminando imagen de la base de datos: {}", e)))?;

        Ok(())
    }

    pub async fn list_images_by_project(&self, project_id: ObjectId) -> Result<Vec<ImageResponse>, AppError> {
        let cursor = self.collection
            .find(doc! { "project_id": project_id })
            .await
            .map_err(|e| AppError::DatabaseError(format!("Error buscando imágenes: {}", e)))?;

        let images: Vec<Image> = cursor.try_collect().await
            .map_err(|e| AppError::DatabaseError(format!("Error recolectando imágenes: {}", e)))?;

        Ok(images.into_iter().map(ImageResponse::from).collect())
    }

    pub async fn list_images_by_task(&self, task_id: ObjectId) -> Result<Vec<ImageResponse>, AppError> {
        let cursor = self.collection
            .find(doc! { "task_id": task_id })
            .await
            .map_err(|e| AppError::DatabaseError(format!("Error buscando imágenes: {}", e)))?;

        let images: Vec<Image> = cursor.try_collect().await
            .map_err(|e| AppError::DatabaseError(format!("Error recolectando imágenes: {}", e)))?;

        Ok(images.into_iter().map(ImageResponse::from).collect())
    }

    pub async fn list_user_images(&self, user_id: ObjectId) -> Result<Vec<ImageResponse>, AppError> {
        let cursor = self.collection
            .find(doc! { "uploaded_by": user_id })
            .await
            .map_err(|e| AppError::DatabaseError(format!("Error buscando imágenes: {}", e)))?;

        let images: Vec<Image> = cursor.try_collect().await
            .map_err(|e| AppError::DatabaseError(format!("Error recolectando imágenes: {}", e)))?;

        Ok(images.into_iter().map(ImageResponse::from).collect())
    }
}
