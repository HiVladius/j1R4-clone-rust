use std::sync::Arc;

use crate::models::{project_models::UpdateProjectSchema, user_model::UserData};
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
};
use mongodb::bson::oid::ObjectId;

use crate::{
    errors::AppError,
    middleware::auth_middleware::AuthenticatedUser,
    models::project_models::{CreateProjectSchema, Project, AddMemberSchema},
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

pub async fn update_project_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
    Json(payload): Json<UpdateProjectSchema>,
) -> Result<Json<Project>, AppError> {
    let project_id = ObjectId::parse_str(&project_id)
        .map_err(|_| AppError::ValidationError("ID de proyecto inválido".to_string()))?;

    let project_service = ProjectService::new(app_state.db.clone());
    let update_project = project_service
        .update_project(project_id, auth_user.id, payload)
        .await?;

    Ok(Json(update_project))
}

pub async fn delete_project_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let project_id = ObjectId::parse_str(&project_id)
        .map_err(|_| AppError::ValidationError("ID de proyecto invalido. ".to_string()))?;

    let project_service = ProjectService::new(app_state.db.clone());
    project_service
        .delete_project(project_id, auth_user.id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_member_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
    Json(payload): Json<AddMemberSchema>,
) -> Result<StatusCode, AppError> {
    let project_id = ObjectId::parse_str(&project_id)
        .map_err(|_| AppError::ValidationError("ID de proyecto inválido".to_string()))?;

    let project_service = ProjectService::new(app_state.db.clone());
    project_service
        .add_member(project_id, auth_user.id, payload)
        .await?;

        Ok(StatusCode::OK)
}

pub async fn list_members_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>
) -> Result<Json<Vec<UserData>>, AppError>{
    let project_id = ObjectId::parse_str(&project_id)
        .map_err(|_| AppError::ValidationError("ID de proyecto inválido".to_string()))?;

    let project_service = ProjectService::new(app_state.db.clone());
    let members = project_service
        .list_members(project_id, auth_user.id)
        .await?;

    Ok(Json(members))    
}


pub async fn remove_member_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path((project_id, user_id)): Path<(String, String)>,
)-> Result<StatusCode, AppError>{ 
    let project_id = ObjectId::parse_str(&project_id)
        .map_err(|_| AppError::ValidationError("ID de proyecto inválido".to_string()))?;
    let user_id = ObjectId::parse_str(&user_id)
        .map_err(|_| AppError::ValidationError("ID de usuario inválido".to_string()))?;

    let project_service = ProjectService::new(app_state.db.clone());
    project_service
        .remove_member(project_id, auth_user.id, user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}