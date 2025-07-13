use crate::{
    errors::AppError,
    middleware::auth_middleware::AuthenticatedUser,
    models::task_model::{CreateDateRangeSchema, DateRange, UpdateDateRangeSchema},
    services::date_range_service::DateRangeService,
    state::AppState,
};
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
};
use mongodb::bson::oid::ObjectId;
use std::sync::Arc;

/// Establecer o actualizar el rango de fechas para una tarea
pub async fn set_task_date_range_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<String>,
    Json(date_range_schema): Json<CreateDateRangeSchema>,
) -> Result<Json<DateRange>, AppError> {
    let task_id = ObjectId::parse_str(&task_id)
        .map_err(|_| AppError::ValidationError("ID de tarea inválido".to_string()))?;

    let date_range_data = DateRange {
        task_id,
        start_date: date_range_schema.start_date,
        end_date: date_range_schema.end_date,
    };

    let date_range_service = DateRangeService::new(app_state.db.clone());

    let date_range = date_range_service
        .set_task_date_range(task_id, date_range_data, auth_user.id)
        .await?;

    Ok(Json(date_range))
}

/// Obtener el rango de fechas para una tarea específica
pub async fn get_task_date_range_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<String>,
) -> Result<Json<Option<DateRange>>, AppError> {
    let task_id = ObjectId::parse_str(&task_id)
        .map_err(|_| AppError::ValidationError("ID de tarea inválido".to_string()))?;

    let date_range_service = DateRangeService::new(app_state.db.clone());

    let date_range = date_range_service
        .get_task_date_range(task_id, auth_user.id)
        .await?;

    Ok(Json(date_range))
}

/// Obtener todos los rangos de fechas para las tareas de un proyecto
pub async fn get_project_date_ranges_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<DateRange>>, AppError> {
    let project_id = ObjectId::parse_str(&project_id)
        .map_err(|_| AppError::ValidationError("ID de proyecto inválido".to_string()))?;

    let date_range_service = DateRangeService::new(app_state.db.clone());

    let date_ranges = date_range_service
        .get_project_date_ranges(project_id, auth_user.id)
        .await?;

    Ok(Json(date_ranges))
}

/// Eliminar el rango de fechas para una tarea
pub async fn delete_task_date_range_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let task_id = ObjectId::parse_str(&task_id)
        .map_err(|_| AppError::ValidationError("ID de tarea inválido".to_string()))?;

    let date_range_service = DateRangeService::new(app_state.db.clone());

    date_range_service
        .delete_task_date_range(task_id, auth_user.id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Actualizar parcialmente el rango de fechas para una tarea
pub async fn update_task_date_range_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<String>,
    Json(update_data): Json<UpdateDateRangeSchema>,
) -> Result<Json<DateRange>, AppError> {
    let task_id = ObjectId::parse_str(&task_id)
        .map_err(|_| AppError::ValidationError("ID de tarea inválido".to_string()))?;

    let date_range_service = DateRangeService::new(app_state.db.clone());
    
    let updated_date_range = date_range_service
        .update_task_date_range(task_id, update_data, auth_user.id)
        .await?;

    Ok(Json(updated_date_range))
}
