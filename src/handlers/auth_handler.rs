use axum::{Json, extract::State, http::StatusCode};
use std::sync::Arc;
// use validator::Validate; // //!optional, si necesitas validación de datos
use crate::{
    errors::AppError,
    models::user_model::{LoginResponse, LoginUserSchema, RegisterUserSchema, UserData},
    services::auth_service::AuthService,
    state::AppState,
};

pub async fn register_handler(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<RegisterUserSchema>,
) -> Result<(StatusCode, Json<UserData>), AppError> {
    let auth_service = AuthService::new(app_state.db.clone(), app_state.config.clone());
    let new_user_data = auth_service.register_user(payload).await?;
    Ok((StatusCode::CREATED, Json(new_user_data)))
}

pub async fn login_handler(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<LoginUserSchema>,
) -> Result<Json<LoginResponse>, AppError> {
    let auth_service = AuthService::new(app_state.db.clone(), app_state.config.clone());
    let login_response = auth_service.login_user(payload).await?;
    Ok(Json(login_response))
}
