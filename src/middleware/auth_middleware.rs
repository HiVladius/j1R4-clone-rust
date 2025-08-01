use axum::{
    body::Body,
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use mongodb::bson::{doc, oid::ObjectId};
use std::sync::Arc;

use crate::{errors::AppError, models::user_model::User, state::AppState, utils::jwt_utils};

#[derive(Debug, Clone, serde::Serialize)]
pub struct AuthenticatedUser(pub User);

pub async fn auth_guard(
    State(app_state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, AppError> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|auth_value| auth_value.strip_prefix("Bearer "));
    let token = token.ok_or_else(|| {
        AppError::Unauthorized("Token de autorización no encontrado".to_string())
    })?;

    let claims = jwt_utils::verify_jwt(token, &app_state.config)?;

    let user_id = ObjectId::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("ID de usuario inválido en el token.".to_string()))?;

    let users_collection: mongodb::Collection<User> = app_state.db.get_db().collection("users");

    let user = users_collection
        .find_one(doc! { "_id": user_id })
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or_else(|| AppError::Unauthorized("El usuario del token ya no existe.".to_string()))?;

    req.extensions_mut().insert(AuthenticatedUser(user));

    Ok(next.run(req).await)
}
