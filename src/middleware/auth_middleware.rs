use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
    body::Body,
};
use std::sync::Arc;
use mongodb::bson::{doc, oid::ObjectId};

use crate::{
    errors::AppError, 
    models::user_model::User, 
    utils::jwt_utils,
    state::AppState,
};

// Struc que guardará la información del usiario autenticado
#[derive(Debug, Clone, serde::Serialize)]
pub struct AuthenticatedUser {
    pub id: ObjectId,
    pub username: String,
    pub email: String,
}

pub async fn auth_guard(
    State(app_state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, AppError> {
    tracing::debug!("Ejecutando middleware de autenticación");

    // //* 1.- Extraer el token del encabezado 'Authorization'

    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|auth_value| auth_value.strip_prefix("Bearer "));
    let token = token.ok_or_else(|| {
        tracing::error!("Token de autorización no encontrado");
        AppError::Unauthorized("Token de autorización no encontrado".to_string())
    })?;

    //* 2.- Validar el token JWT

    let claims = jwt_utils::verify_jwt(token, &app_state.config)?;

    let user_id = ObjectId::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("ID de usuario inválido en el token.".to_string()))?;

    let users_collection: mongodb::Collection<User> = app_state.db.get_db().collection("users");

    let user = users_collection
        .find_one(doc! { "_id": user_id })
        .await
        .map_err(|_| AppError::InternalServerError)? // Error de base deatos
        .ok_or_else(|| AppError::Unauthorized("El usuario del token ya no existe.".to_string()))?;

    let authenticated_user = AuthenticatedUser {
        id: user.id.unwrap(),
        email: user.email,
        username: user.username,
    };

    req.extensions_mut().insert(authenticated_user);
    tracing::debug!(
        "Usuario autenticado: {:?}",
        req.extensions().get::<AuthenticatedUser>()
    );

    Ok(next.run(req).await)
}
