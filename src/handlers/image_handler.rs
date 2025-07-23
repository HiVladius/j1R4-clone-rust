use std::sync::Arc;

use axum::{
    extract::{Extension, Multipart, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::{
    errors::AppError,
    middleware::auth_middleware::AuthenticatedUser,
    models::image_model::{ImageResponse, ImageUploadQuery, UpdateImageSchema},
    services::image_service::ImageService,
    state::AppState,
};

/// Subir una nueva imagen
pub async fn upload_image_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(query_params): Query<ImageUploadQuery>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ImageResponse>), AppError> {
    let image_service = ImageService::new(app_state.db.clone(), app_state.config.clone()).await?;

    // Parsear project_id y task_id opcionales
    let project_id = if let Some(pid) = query_params.project_id {
        Some(ObjectId::parse_str(&pid)
            .map_err(|_| AppError::ValidationError("ID de proyecto inválido".to_string()))?)
    } else {
        None
    };

    let task_id = if let Some(tid) = query_params.task_id {
        Some(ObjectId::parse_str(&tid)
            .map_err(|_| AppError::ValidationError("ID de tarea inválido".to_string()))?)
    } else {
        None
    };

    // Procesar el archivo del multipart
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart.next_field().await
        .map_err(|e| AppError::ValidationError(format!("Error procesando multipart: {}", e)))? {
        
        let field_name = field.name().unwrap_or("");
        
        if field_name == "file" {
            filename = field.file_name().map(|s| s.to_string());
            content_type = field.content_type().map(|s| s.to_string());
            
            let data = field.bytes().await
                .map_err(|e| AppError::ValidationError(format!("Error leyendo archivo: {}", e)))?;
            
            file_data = Some(data.to_vec());
        }
    }

    // Validar que se recibió un archivo
    let file_data = file_data.ok_or_else(|| 
        AppError::ValidationError("No se encontró archivo en la petición".to_string()))?;
    
    let filename = filename.ok_or_else(|| 
        AppError::ValidationError("Nombre de archivo requerido".to_string()))?;
    
    let content_type = content_type.unwrap_or_else(|| 
        mime_guess::from_path(&filename).first_or_octet_stream().to_string());

    // Validar tamaño del archivo (máximo 10MB)
    if file_data.len() > 10 * 1024 * 1024 {
        return Err(AppError::ValidationError("El archivo es demasiado grande (máximo 10MB)".to_string()));
    }

    // Validar tipo de archivo (solo imágenes)
    if !content_type.starts_with("image/") {
        return Err(AppError::ValidationError("Solo se permiten archivos de imagen".to_string()));
    }

    let image = image_service
        .upload_image(file_data, filename, content_type, auth_user.id, project_id, task_id)
        .await?;

    Ok((StatusCode::CREATED, Json(image)))
}

/// Obtener información de una imagen
pub async fn get_image_info_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(image_id): Path<String>,
) -> Result<Json<ImageResponse>, AppError> {
    let image_service = ImageService::new(app_state.db.clone(), app_state.config.clone()).await?;
    
    let image_oid = ObjectId::parse_str(&image_id)
        .map_err(|_| AppError::ValidationError("ID de imagen inválido".to_string()))?;

    let image = image_service.get_image(image_oid).await?;
    Ok(Json(image))
}

/// Descargar una imagen
pub async fn download_image_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(image_id): Path<String>,
) -> Result<Response, AppError> {
    let image_service = ImageService::new(app_state.db.clone(), app_state.config.clone()).await?;
    
    let image_oid = ObjectId::parse_str(&image_id)
        .map_err(|_| AppError::ValidationError("ID de imagen inválido".to_string()))?;

    // Obtener información de la imagen
    let image_info = image_service.get_image(image_oid).await?;
    
    // Obtener los datos del archivo
    let file_data = image_service.get_image_data(image_oid).await?;

    // Crear headers para la respuesta
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        image_info.content_type.parse()
            .map_err(|_| {
                tracing::error!("Content-Type inválido");
                AppError::InternalServerError
            })?
    );
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        format!("inline; filename=\"{}\"", image_info.original_filename).parse()
            .map_err(|_| {
                tracing::error!("Content-Disposition inválido");
                AppError::InternalServerError
            })?
    );

    Ok((headers, file_data).into_response())
}

/// Actualizar información de una imagen
pub async fn update_image_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(image_id): Path<String>,
    Json(payload): Json<UpdateImageSchema>,
) -> Result<Json<ImageResponse>, AppError> {
    let image_service = ImageService::new(app_state.db.clone(), app_state.config.clone()).await?;
    
    let image_oid = ObjectId::parse_str(&image_id)
        .map_err(|_| AppError::ValidationError("ID de imagen inválido".to_string()))?;

    let updated_image = image_service
        .update_image(image_oid, payload, auth_user.id)
        .await?;

    Ok(Json(updated_image))
}

/// Eliminar una imagen
pub async fn delete_image_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(image_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let image_service = ImageService::new(app_state.db.clone(), app_state.config.clone()).await?;
    
    let image_oid = ObjectId::parse_str(&image_id)
        .map_err(|_| AppError::ValidationError("ID de imagen inválido".to_string()))?;

    image_service.delete_image(image_oid, auth_user.id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Listar imágenes de un proyecto
pub async fn list_project_images_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<ImageResponse>>, AppError> {
    let image_service = ImageService::new(app_state.db.clone(), app_state.config.clone()).await?;
    
    let project_oid = ObjectId::parse_str(&project_id)
        .map_err(|_| AppError::ValidationError("ID de proyecto inválido".to_string()))?;

    let images = image_service.list_images_by_project(project_oid).await?;
    Ok(Json(images))
}

/// Listar imágenes de una tarea
pub async fn list_task_images_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<String>,
) -> Result<Json<Vec<ImageResponse>>, AppError> {
    let image_service = ImageService::new(app_state.db.clone(), app_state.config.clone()).await?;
    
    let task_oid = ObjectId::parse_str(&task_id)
        .map_err(|_| AppError::ValidationError("ID de tarea inválido".to_string()))?;

    let images = image_service.list_images_by_task(task_oid).await?;
    Ok(Json(images))
}

/// Listar todas las imágenes del usuario
pub async fn list_user_images_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<ImageResponse>>, AppError> {
    let image_service = ImageService::new(app_state.db.clone(), app_state.config.clone()).await?;

    let images = image_service.list_user_images(auth_user.id).await?;
    Ok(Json(images))
}
