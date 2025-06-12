use crate::{
    errors::AppError,
    middleware::auth_middleware::AuthenticatedUser,
    models::task_model::{CreateTaskSchema, Task},
    services::task_service::TaskService,
    state::AppState,
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
