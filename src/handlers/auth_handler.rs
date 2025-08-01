use axum::{
    Json, debug_handler,
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use std::sync::Arc;
use crate::{
    errors::AppError,
    middleware::auth_middleware::AuthenticatedUser,
    models::user_model::{LoginResponse, LoginUserSchema, RegisterUserSchema, UpdateUserSchema, UserData},
    services::auth_service::AuthService,
    state::AppState,
};

#[derive(Debug)]
pub enum AuthHandlerError {
    ServiceError(String),
    ValidationError(String),
    NotFound,
    Unauthorized,
}

impl IntoResponse for AuthHandlerError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthHandlerError::ServiceError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AuthHandlerError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AuthHandlerError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            AuthHandlerError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
            }
        };

        let body = Json(serde_json::json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

impl From<AppError> for AuthHandlerError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::NotFound(_) => AuthHandlerError::NotFound,
            AppError::ValidationError(msg) => AuthHandlerError::ValidationError(msg),
            _ => AuthHandlerError::ServiceError(err.to_string()),
        }
    }
}

pub async fn register_handler(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<RegisterUserSchema>,
) -> Result<(StatusCode, Json<UserData>), AuthHandlerError> {
    let auth_service = AuthService::new(app_state.db.clone(), app_state.config.clone());
    let new_user_data = auth_service.register_user(payload).await?;
    Ok((StatusCode::CREATED, Json(new_user_data)))
}

pub async fn login_handler(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<LoginUserSchema>,
) -> Result<Json<LoginResponse>, AuthHandlerError> {
    let auth_service = AuthService::new(app_state.db.clone(), app_state.config.clone());
    let login_response = auth_service.login_user(payload).await?;
    Ok(Json(login_response))
}

#[debug_handler]
pub async fn get_me_handler(
    Extension(authenticated_user): Extension<AuthenticatedUser>,
) -> Json<UserData> {
    Json(authenticated_user.0.into())
}

#[debug_handler]
pub async fn update_me_handler(
    State(app_state): State<Arc<AppState>>,
    Extension(authenticated_user): Extension<AuthenticatedUser>,
    Json(payload): Json<UpdateUserSchema>,
) -> Result<Json<UserData>, AuthHandlerError> {
    let auth_service = AuthService::new(app_state.db.clone(), app_state.config.clone());
    let user_id = authenticated_user.0.id.as_ref().unwrap();
    let updated_user = auth_service.update_user_details(user_id, payload).await?;
    Ok(Json(updated_user))
}
