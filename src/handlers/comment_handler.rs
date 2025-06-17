use std::sync::Arc;
use axum::{
    extract::{Path, State}, http::StatusCode, Extension, Json
};
use mongodb::bson::oid::ObjectId;

use crate::{
    errors::AppError,
    middleware::auth_middleware::AuthenticatedUser,
    models::comment_model::{CreateCommentSchema, CommentData},
    services::comment_service::CommentService,
    state::AppState,
};


pub async fn create_comment_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<String>,
    Json(payload): Json<CreateCommentSchema>,
) -> Result<(StatusCode, Json<CommentData>), AppError> {
    let task_id = ObjectId::parse_str(&task_id)
        .map_err(|_| AppError::ValidationError("ID de tarea inválido.".to_string()))?;

    let comment_service = CommentService::new(app_state.db.clone());
    let new_comment = comment_service
        .create_comment(task_id, auth_user.id, payload)
        .await?;
    
    Ok((StatusCode::CREATED, Json(new_comment)))
}


pub async fn get_comments_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<String>,
)-> Result<Json<Vec<CommentData>>, AppError> {
    let task_id = ObjectId::parse_str(&task_id)
        .map_err(|_| AppError::ValidationError("ID de tarea inválido.".to_string()))?;

    let comment_service = CommentService::new(app_state.db.clone());

    let comments = comment_service
        .get_comments_for_task(task_id, auth_user.id, None)
        .await?;
    
    Ok(Json(comments))
}