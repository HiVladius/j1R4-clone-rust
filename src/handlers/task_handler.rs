use crate::{
    errors::AppError,
    middleware::auth_middleware::AuthenticatedUser,
    models::task_model::{CreateTaskSchema, Task},
    services::task_service::TaskService,
    state::AppState,
    models::task_model::UpdateTaskSchema,
};
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
};
use mongodb::bson::oid::ObjectId;
use std::sync::Arc;

pub async fn create_task_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
    Json(task_data): Json<CreateTaskSchema>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    let project_id = ObjectId::parse_str(&project_id)
        .map_err(|_| AppError::ValidationError("ID de proyecto invalido".to_string()))?;

    let task_service = TaskService::new(app_state.db.clone());

    let new_task = task_service
        .create_task(task_data, project_id, auth_user.id)
        .await?;

    Ok((StatusCode::CREATED, Json(new_task)))
}

pub async fn get_task_for_project_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<Task>>, AppError> {
    let project_id = ObjectId::parse_str(&project_id)
        .map_err(|_| AppError::ValidationError("ID de proyecto invalido".to_string()))?;

    let task_service = TaskService::new(app_state.db.clone());
    let tasks = task_service
        .get_task_for_project(project_id, auth_user.id)
        .await?;

    Ok(Json(tasks))
}

pub async fn get_task_by_id_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<String>,
) -> Result<Json<Task>, AppError> {
    let task_id = ObjectId::parse_str(&task_id)
        .map_err(|_| AppError::ValidationError("ID de tarea invalido".to_string()))?;

    let task_service = TaskService::new(app_state.db.clone());
    let task = task_service.get_task_by_id(task_id, auth_user.id).await?;

    Ok(Json(task))
}

pub async fn update_task_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<String>,
    Json(payload): Json<UpdateTaskSchema>,
) -> Result<Json<Task>, AppError> {
    let task_id = ObjectId::parse_str(&task_id)
        .map_err(|_| AppError::ValidationError("ID de tarea invalido".to_string()))?;

    let task_service = TaskService::new(app_state.db.clone());
    let update_task = task_service
        .update_task(task_id, auth_user.id, payload)
        .await?;


    Ok(Json(update_task))
}


pub async fn delete_task_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<String>,
) -> Result<StatusCode, AppError>{

    let task_id = ObjectId::parse_str(&task_id)
        .map_err(|_| AppError::ValidationError("ID de tarea invalido".to_string()))?;

    let task_service = TaskService::new(app_state.db.clone());
    task_service
        .delete_task(task_id, auth_user.id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}