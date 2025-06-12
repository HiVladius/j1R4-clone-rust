use crate::{config::Config, db::DatabaseState, router::router::get_app, state::AppState};
use axum::{Router, body::Body, http::Request};
use dotenvy::dotenv;
use serde_json::json;

use std::sync::Arc;
use tower::ServiceExt;

async fn setup_app() -> Router {
    dotenv().ok();

    let test_db_name = "jira_clone_test_db";
    let config = Arc::new(Config::from_env().expect("Fallo al cargar la configuración"));

    let db_state = Arc::new(
        DatabaseState::init(&config.database_url, test_db_name)
            .await
            .expect("Fallo al inicializar la base de datos"),
    );

    let app_state = Arc::new(AppState::new(db_state, config));
    get_app(app_state)
}

#[tokio::test]
async fn test_example() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "Expected status code 200 OK"
    );
}

#[tokio::test]
async fn test_project_creation() {
    let app = setup_app().await;

    // Generar datos únicos para este test usando timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let unique_username = format!("testuser{}", timestamp);
    let unique_email = format!("test{}@example.com", timestamp);
    let unique_project_key = format!("TEST{}", timestamp % 10000); // Limitamos a 4 dígitos para mantenernos dentro del límite de 10 caracteres

    // Primero crear un usuario
    let register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/register")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": unique_username,
                        "email": unique_email,
                        "password": "password123"
                    })
                    .to_string(),
                ))
                .expect("Failed to build register request"),
        )
        .await
        .unwrap();

    // Leer el cuerpo de la respuesta para ver el error
    let register_status = register_response.status();
    if register_status != axum::http::StatusCode::CREATED {
        let body_bytes = axum::body::to_bytes(register_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_msg = String::from_utf8_lossy(&body_bytes);
        panic!(
            "Register failed with status: {}. Error: {}",
            register_status, error_msg
        );
    }

    // Hacer login para obtener el token
    let login_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/login")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": unique_email,
                        "password": "password123"
                    })
                    .to_string(),
                ))
                .expect("Failed to build login request"),
        )
        .await
        .unwrap();

    assert_eq!(login_response.status(), axum::http::StatusCode::OK);

    // Extraer el token de la respuesta
    let body_bytes = axum::body::to_bytes(login_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let login_data: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let token = login_data["token"].as_str().unwrap();

    // Crear proyecto con autenticación
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/projects")
                .method("POST")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(
                    json!({
                        "name": "Test Project",
                        "key": unique_project_key,
                        "description": "This is a test project"
                    })
                    .to_string(),
                ))
                .expect("Failed to build request"),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::CREATED,
        "Expected status code 201 CREATED"
    );
}
