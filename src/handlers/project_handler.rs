use std::sync::Arc;

use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};

use crate::{
    errors::AppError,
    middleware::auth_middleware::AuthenticatedUser,
    models::project_models::{CreateProjectSchema, Project},
    services::project_service::ProjectService,
    state::AppState,
};

pub async fn create_project_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(payload): Json<CreateProjectSchema>,
) -> Result<(StatusCode, Json<Project>), AppError> {
    let project_service = ProjectService::new(app_state.db.clone());
    let new_project = project_service
        .create_project(payload, auth_user.id)
        .await
        .map_err(|e| AppError::from(e))?;

    Ok((StatusCode::CREATED, Json(new_project)))
}

pub async fn get_project_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<Project>>, AppError> {
    let project_service = ProjectService::new(app_state.db.clone());
    let project = project_service.get_projects_for_user(auth_user.id).await?;

    Ok(Json(project))
}
