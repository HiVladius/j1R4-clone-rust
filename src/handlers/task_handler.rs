use crate::{
    errors::AppError,
    middleware::auth_middleware::AuthenticatedUser,
    models::task_model::UpdateTaskSchema,
    models::task_model::{CreateTaskSchema, DateRange, Task},
    services::date_range_service::DateRangeService,
    services::task_service::TaskService,
    state::AppState,
};
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskWithDateRange {
    pub task: Task,
    pub date_range: Option<DateRange>,
}

pub async fn create_task_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
    Json(task_data): Json<CreateTaskSchema>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    let project_id = ObjectId::parse_str(&project_id)
        .map_err(|_| AppError::ValidationError("ID de proyecto invalido".to_string()))?;

    let task_service = TaskService::new(app_state.db.clone(), app_state.ws_tx.clone());

    let new_task = task_service
        .create_task(task_data, project_id, auth_user.0.id.unwrap())
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

    let task_service = TaskService::new(app_state.db.clone(), app_state.ws_tx.clone());
    let tasks = task_service
        .get_task_for_project(project_id, auth_user.0.id.unwrap())
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

    let task_service = TaskService::new(app_state.db.clone(), app_state.ws_tx.clone());
    let task = task_service.get_task_by_id(task_id, auth_user.0.id.unwrap()).await?;

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

    let task_service = TaskService::new(app_state.db.clone(), app_state.ws_tx.clone());
    let update_task = task_service
        .update_task(task_id, auth_user.0.id.unwrap(), payload)
        .await?;

    Ok(Json(update_task))
}

pub async fn delete_task_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let task_id = ObjectId::parse_str(&task_id)
        .map_err(|_| AppError::ValidationError("ID de tarea invalido".to_string()))?;

    let task_service = TaskService::new(app_state.db.clone(), app_state.ws_tx.clone());
    task_service.delete_task(task_id, auth_user.0.id.unwrap()).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_task_with_date_range_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskWithDateRange>, AppError> {
    let task_id = ObjectId::parse_str(&task_id)
        .map_err(|_| AppError::ValidationError("ID de tarea invalido".to_string()))?;

    let task_service = TaskService::new(app_state.db.clone(), app_state.ws_tx.clone());
    let date_range_service = DateRangeService::new(app_state.db.clone());

    let task = task_service.get_task_by_id(task_id, auth_user.0.id.unwrap()).await?;
    let date_range = date_range_service
        .get_task_date_range(task_id, auth_user.0.id.unwrap())
        .await?;

    Ok(Json(TaskWithDateRange { task, date_range }))
}
