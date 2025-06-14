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
    models::{
        project_models::Project,
        task_model::{Task, TaskStatus},
    },
};

#[tokio::test]
async fn test_task_update_and_delete_flow() {
    // //* 1.- Setup
    // //* Dos usuarios. El usuario A crea un proyecto y una tarea.

    let app = setup_app().await;    
    let user_a_email = format!("taskowner{}@example.com", Uuid::new().to_string().replace("-", ""));
    let user_b_email = format!("taskassignee{}@example.com", Uuid::new().to_string().replace("-", ""));
    let token_a = get_auth_token(&app, "user_a_task", &user_a_email).await;
    let token_b = get_auth_token(&app, "user_b_task", &user_b_email).await;

    // Usuario A crea proyecto y tarea
    let project_id_str = create_project_for_user(&app, &token_a, "PROJEDIT").await;
    let task_payload = json!({"title": "Tarea para editar"});
    let create_task_resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/projects/{}/tasks", project_id_str))
            .header(header::AUTHORIZATION, format!("Bearer {}", token_a))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(task_payload.to_string()))
            .unwrap(),
    ).await.unwrap();

    let task: Task = serde_json::from_slice(
        &to_bytes(create_task_resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    let task_id_str = task.id.unwrap().to_hex();

    // //* 2 Prueba UPDATE

    let update_payload = json!({"title": "Titulo actualizado", "status": "InProgress",});    // //* 2 Prueba UPDATE NO AUTORIZADA
    // El usuario B intenta actualizar una tarea del usuario A -> Debe fallar

    let unauthorized_update = app.clone().oneshot(
        Request::builder()
        .method("PATCH").uri(format!("/api/tasks/{}", task_id_str))
        .header(header::AUTHORIZATION, format!("Bearer {}", token_b))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(update_payload.to_string())).unwrap(),
    ).await.unwrap();

    assert_eq!(unauthorized_update.status(), StatusCode::UNAUTHORIZED);

    // //* 2.5 Prueba UPDATE AUTORIZADA
    // El usuario A actualiza su propia tarea -> Debe funcionar

    let authorized_update = app.clone().oneshot(
        Request::builder()
        .method("PATCH").uri(format!("/api/tasks/{}", task_id_str))
        .header(header::AUTHORIZATION, format!("Bearer {}", token_a))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(update_payload.to_string())).unwrap(),
    ).await.unwrap();

    assert_eq!(authorized_update.status(), StatusCode::OK);
    let update_tasks: Task = serde_json::from_slice(
        &to_bytes(authorized_update.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(update_tasks.title, "Titulo actualizado");
    assert_eq!(update_tasks.status, TaskStatus::InProgress);

    // //* 3 Prueba DELETE

    let unauthorized_delete = app.clone().oneshot(
          Request::builder()
            .method("DELETE").uri(format!("/api/tasks/{}", task_id_str))
            .header(header::AUTHORIZATION, format!("Bearer {}", token_b))
            .body(Body::empty()).unwrap(),
    ).await.unwrap();
    assert_eq!(unauthorized_delete.status(), StatusCode::UNAUTHORIZED);

    // Usuario A elimina  -> Debe de funcionar
    let authorized_delete = app.clone().oneshot(
        Request::builder()
           .method("DELETE").uri(format!("/api/tasks/{}", task_id_str))
            .header(header::AUTHORIZATION, format!("Bearer {}", token_a))
            .body(Body::empty()).unwrap(),
    ).await.unwrap();
    assert_eq!(authorized_delete.status(), StatusCode::NO_CONTENT);

    // //*4 Verificacion final
    // Intentar obtener la tarea eliminada debe dar un 404 NOT FOUND
    let get_delete_task = app.clone().oneshot(
        Request::builder()
            .method("GET")
            .uri(format!("/api/tasks/{}", task_id_str))
            .header(header::AUTHORIZATION, format!("Bearer {}", token_a))
            .body(Body::empty()).unwrap(),   
    ).await.unwrap();
    assert_eq!(get_delete_task.status(), StatusCode::NOT_FOUND);


    async fn create_project_for_user(app: &axum::Router, token: &str, key: &str) -> String {
        let payload = json!({"name": format!("Project for {}", key), "key": key});
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/projects")
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let project: Project =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        project.id.unwrap().to_hex()
    }
}
