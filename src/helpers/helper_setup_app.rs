use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, header},
};
use bson::uuid;
use dotenvy::dotenv;
use mongodb::bson::doc;
use tower::ServiceExt;

use crate::{
    config::Config,
    db::DatabaseState,
    models::{
        project_models::Project,
        user_model::{User, UserLoginResponseTest},
    },
    router::router::get_app,
    state::AppState,
};

use serde_json::from_slice;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub async fn setup_app() -> Router {
    dotenv().ok();
    // Generar un nombre de DB más corto para cumplir con el límite de 38 bytes de MongoDB Atlas
    let uuid_short = Uuid::new().to_string().replace("-", "")[..8].to_string();
    let test_db_name = format!("test_{}", uuid_short);
    let config = Arc::new(Config::from_env().expect("Fallo al cargar config de prueba"));
    let db_state = Arc::new(
        DatabaseState::init(&config.database_url, &test_db_name)
            .await
            .expect("Fallo al conectar a la DB de prueba"),
    );
    db_state
        .get_db()
        .collection::<User>("users")
        .delete_many(doc! {})
        .await
        .ok();
    db_state
        .get_db()
        .collection::<Project>("projects")
        .delete_many(doc! {})
        .await
        .ok();
    let app_state = Arc::new(AppState::new(db_state, config));
    get_app(app_state)
}

pub async fn get_auth_token(app: &Router, username: &str, email: &str) -> String {
    let register_payload =
        json!({ "username": username, "email": email, "password": "password123" });
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(register_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let login_payload = json!({ "email": email, "password": "password123" });
    let login_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(login_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let body_bytes = to_bytes(login_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let login_data: UserLoginResponseTest = from_slice(&body_bytes)
        .expect("No se pudo deserializar la respuesta del login en la prueba");
    login_data.token
}
