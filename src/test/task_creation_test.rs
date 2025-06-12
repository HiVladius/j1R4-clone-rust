use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use bson::uuid;

use crate::{
    helpers::helper_setup_app::{get_auth_token, setup_app},
    models::{project_models::Project, task_model::{Task, TaskStatus::ToDo}},
};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use axum::body::to_bytes;

#[tokio::test]
async fn test_task_creation_permissions() {
    let app = setup_app().await;

    let user_a_email = format!("task-owner-{}@test.com", Uuid::new());
    let user_b_email = format!("other_user-{}@test.com", Uuid::new());

    let token_a = get_auth_token(&app, "user_a", &user_a_email).await;
    let token_b = get_auth_token(&app, "user_b", &user_b_email).await;

    // //! El usuario A crea un proyecto
    let create_project_payload = json!({"name": "Proyecto de A", "key": "TSK"});
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

    let response_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");

    let project: Project =
        serde_json::from_slice(&response_body).expect("Failed to deserialize project");
    let project_id = project.id.unwrap().to_hex();

    // //! 2. PRUEBA DE CREACION NO AUTORIZADA
    // //? El usuario B intenta crear una tarea en el proyecto de A -> Debe Fallar 403 UNAUTHORIZED
    let task_payload = json!({"title": "Tarea maliciosa"});
    let unauthorized_reponse = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/projects/{}/tasks", project_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", token_b))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(task_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauthorized_reponse.status(), StatusCode::UNAUTHORIZED);


    // //! 3. PRUEBA DE CREACION AUTORIZADA
    // //? El usuario A crea una tarea en su proyecto -> Debe Satisfactorio 201 CREATED
    let authorized_reponse = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/projects/{}/tasks", project_id))
            .header(header::AUTHORIZATION, format!("Bearer {}", token_a))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(task_payload.to_string()))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(authorized_reponse.status(), StatusCode::CREATED);

    let created_task: Task = serde_json::from_slice(
        &to_bytes(authorized_reponse.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body"),
    )
    .expect("Failed to deserialize task");

    assert_eq!(created_task.title, "Tarea maliciosa");
    assert_eq!(created_task.project_id.to_hex(), project_id);
    assert_eq!(created_task.status, ToDo);


}
