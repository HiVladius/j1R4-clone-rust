use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use mongodb::bson::oid::ObjectId;
use std::sync::Arc;

use crate::{
    errors::AppError,
    middleware::auth_middleware::AuthenticatedUser,
    models::comment_model::{CommentData, CreateCommentSchema, UpdateCommentSchema},
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
) -> Result<Json<Vec<CommentData>>, AppError> {
    let task_id = ObjectId::parse_str(&task_id)
        .map_err(|_| AppError::ValidationError("ID de tarea inválido.".to_string()))?;

    let comment_service = CommentService::new(app_state.db.clone());

    let comments = comment_service
        .get_comments_for_task(task_id, auth_user.id, None)
        .await?;

    Ok(Json(comments))
}

pub async fn update_comment_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path((_task_id, comment_id)): Path<(String, String)>, // ← Aquí está el cambio
    Json(payload): Json<UpdateCommentSchema>,
) -> Result<Json<CommentData>, AppError> {
    let comment_id = ObjectId::parse_str(&comment_id)
        .map_err(|_| AppError::ValidationError("ID de comentario inválido.".to_string()))?;

    let comment_service = CommentService::new(app_state.db.clone());

    let updated_comment = comment_service
        .update_comment(comment_id, auth_user.id, payload)
        .await?;

    Ok(Json(updated_comment))
}

pub async fn delete_comment_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path((_task_id, comment_id)): Path<(String, String)>, // ← Y aquí también
) -> Result<StatusCode, AppError> {
    let comment_id = ObjectId::parse_str(&comment_id)
        .map_err(|_| AppError::ValidationError("ID de comentario inválido.".to_string()))?;

    let comment_service = CommentService::new(app_state.db.clone());
    comment_service
        .delete_comment(comment_id, auth_user.id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}