use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use bson::uuid;

use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    helpers::helper_setup_app::{get_auth_token, setup_app},
    models::{project_models::Project, task_model::Task},
};

#[tokio::test]
async fn test_task_read_permissions() {
    let app = setup_app().await;

    let user_a_email = format!("read_owner{}@test.com", Uuid::new());
    let user_b_email = format!("read-owner{}@test.com", Uuid::new());

    let token_a = get_auth_token(&app, "user_a_read", &user_a_email).await;
    let token_b = get_auth_token(&app, "user_b_read", &user_b_email).await;

    // Usuario A crea un proyecto
    let create_project_payload = json!({"name": "Proyecto de lectura", "key": "READ"});
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/projects")
                .header(header::AUTHORIZATION, format!("Bearer {}", token_a))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_project_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let project: Project = serde_json::from_slice(
        &to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    let project_id_str = project.id.unwrap().to_hex();

    // Usuario A crea dos tareas
    let task_payload_1 = json!({"title": "Tarea 1"});
    let task_payload_2 = json!({"title": "Tarea 2"});

    let create_task_resp_1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/projects/{}/tasks", project_id_str))
                .header(header::AUTHORIZATION, format!("Bearer {}", token_a))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(task_payload_1.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let task1: Task = serde_json::from_slice(
        &to_bytes(create_task_resp_1.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let task1_id_str = task1.id.unwrap().to_hex();

    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/projects/{}/tasks", project_id_str))
                .header(header::AUTHORIZATION, format!("Bearer {}", token_a))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(task_payload_2.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Pruebas de listar tareas
    // Usuario B intenta listar las tareas -> DEBE DE FALLAR

    // --- 2. PRUEBAS DE LISTAR TAREAS ---
    // Usuario B intenta listar las tareas -> DEBE FALLAR (401 Unauthorized)
    let unauthorized_list_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/projects/{}/tasks", project_id_str))
                .header(header::AUTHORIZATION, format!("Bearer {}", token_b))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauthorized_list_resp.status(), StatusCode::UNAUTHORIZED);

    // Usuario A lista sus tareas -> DEBE FUNCIONAR y devolver 2 tareas
    let authorized_list_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/projects/{}/tasks", project_id_str))
                .header(header::AUTHORIZATION, format!("Bearer {}", token_a))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(authorized_list_resp.status(), StatusCode::OK);
    let tasks: Vec<Task> = serde_json::from_slice(
        &to_bytes(authorized_list_resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(tasks.len(), 2);

    // --- 3. PRUEBAS DE OBTENER TAREA INDIVIDUAL ---
    // Usuario B intenta obtener una tarea por ID -> DEBE FALLAR (401 Unauthorized)
    let unauthorized_get_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/tasks/{}", task1_id_str))
                .header(header::AUTHORIZATION, format!("Bearer {}", token_b))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauthorized_get_resp.status(), StatusCode::UNAUTHORIZED);

    // Usuario A obtiene la tarea por ID -> DEBE FUNCIONAR
    let authorized_get_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/tasks/{}", task1_id_str))
                .header(header::AUTHORIZATION, format!("Bearer {}", token_a))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(authorized_get_resp.status(), StatusCode::OK);
    let fetched_task: Task = serde_json::from_slice(
        &to_bytes(authorized_get_resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(fetched_task.title, "Tarea 1");
}
