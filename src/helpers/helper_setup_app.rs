use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, header, StatusCode},
};
use bson:: oid::ObjectId;
use dotenvy::dotenv;
use mongodb::bson::doc;
use tokio::sync::broadcast;
use tower::ServiceExt;

use crate::{
    config::Config,
    db::DatabaseState,
    models::{
        project_models::Project,
        user_model::{User,  LoginResponse},
        task_model::Task,
        comment_model::Comment,
    },
    router::router::get_app,
    state::{AppState},
};

use serde_json::from_slice;
use serde_json::json;
use std::sync::Arc;

pub async fn setup_app() -> Router {
    dotenv().ok();
    // Use the existing database from config instead of creating a new one
    let config = Arc::new(Config::from_env().expect("Fallo al cargar config de prueba"));
    
    // Use a fixed database name for tests
    let db_state = Arc::new(
        DatabaseState::init(&config.database_url, "test_db")
            .await
            .expect("Fallo al conectar a la DB de prueba"),
    );
    // Clean up collections using the actual collection names used by the services
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
    db_state
        .get_db()
        .collection::<Task>("tasks")
        .delete_many(doc! {})
        .await
        .ok();
    db_state
        .get_db()
        .collection::<Comment>("comments")
        .delete_many(doc! {})
        .await
        .ok();
    
    // Create a temporary WebSocket channel for tests
    let (ws_tx, _) = broadcast::channel::<String>(100);
    
    let app_state = Arc::new(AppState::new(db_state, config, ws_tx));
    get_app(app_state)
}

pub async fn get_auth_token(app: &Router, username: &str, email: &str) -> String {
    let register_payload =
        json!({ "username": username, "email": email, "password": "password123" });
    
    // Send registration request
    let register_response = app
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
    
    // Check registration status
    let register_status = register_response.status();
    println!("Registration response status: {:?}", register_status);
    
    // Check registration response body
    let register_body_bytes = to_bytes(register_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let register_body_str = String::from_utf8_lossy(&register_body_bytes);
    println!("Registration response body: {}", register_body_str);
    
    // Ensure registration was successful
    if !register_status.is_success() {
        panic!("Registration failed with status {} and body: {}", register_status, register_body_str);
    }
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
    
    // Check and print status code
    let status = login_response.status();
    println!("Login response status: {:?}", status);
    
    let body_bytes = to_bytes(login_response.into_body(), usize::MAX)
        .await
        .unwrap();
    
    // Print response body as string for debugging
    let body_str = String::from_utf8_lossy(&body_bytes);
    println!("Login response body: {}", body_str);
    
    // Try to deserialize with more descriptive error handling
    let login_data: LoginResponse = match from_slice(&body_bytes) {
        Ok(data) => data,
        Err(e) => {
            println!("JSON deserialization error: {}", e);
            println!("Expected structure: {{ \"token\": \"...\", \"user\": {{ ... }} }}");
            panic!("No se pudo deserializar la respuesta del login: {}", e);
        }
    };
    login_data.token
}

pub async fn get_auth_token_and_id(app: &Router, username: &str, email: &str) -> (String, ObjectId) {
    let register_payload = json!({ "username": username, "email": email, "password": "password123" });
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST").uri("/api/auth/register").header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(register_payload.to_string())).unwrap()
    ).await.unwrap();
    let login_payload = json!({ "email": email, "password": "password123" });
    let login_response = app.clone().oneshot(
        Request::builder()
            .method("POST").uri("/api/auth/login").header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(login_payload.to_string())).unwrap()
    ).await.unwrap();
    let body_bytes = to_bytes(login_response.into_body(), usize::MAX).await.unwrap();
    let login_data: LoginResponse = from_slice(&body_bytes).unwrap();
    let user_id = ObjectId::parse_str(&login_data.user.id)
        .expect("No se pudo parsear el ID de usuario desde la respuesta del login en el helper");
    (login_data.token, user_id)
}

pub async fn create_project_for_user(app: &Router, token: &str, key: &str) -> String {
    let payload = json!({ "name": format!("Proyecto {}", key), "key": key });
    let response = app.clone().oneshot(
        Request::builder()
            .method("POST").uri("/api/projects").header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json").body(Body::from(payload.to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED, "Helper 'create_project_for_user' falló");
    let project: Project = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    project.id.unwrap().to_hex()
}


pub async fn add_member_to_project(app: &Router, owner_token: &str, project_id: &str, member_email: &str) {
    let payload = json!({ "email": member_email });
    let response = app.clone().oneshot(
        Request::builder()
            .method("POST").uri(format!("/api/projects/{}/members", project_id))
            .header(header::AUTHORIZATION, format!("Bearer {}", owner_token))
            .header(header::CONTENT_TYPE, "application/json").body(Body::from(payload.to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK, "Helper 'add_member_to_project' falló");
}


pub async fn create_task_for_project(
    app: &Router,
    token: &str,
    project_id: &str,
    assignee_id: Option<ObjectId>,
) -> String {
    let mut payload = json!({ "title": "Tarea de prueba desde Helper" });
    if let Some(id) = assignee_id {
        payload["assignee_id"] = json!(id.to_hex());
    }
    let response = app.clone().oneshot(
        Request::builder()
            .method("POST").uri(format!("/api/projects/{}/tasks", project_id))
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json").body(Body::from(payload.to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED, "Helper 'create_task_for_project' falló");
    let task: Task = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    task.id.unwrap().to_hex()
}