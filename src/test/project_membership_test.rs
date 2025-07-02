use axum::{
    body::{Body, to_bytes},
    extract::Request,
};
use bson::uuid;
use hyper::{StatusCode, header};
use serde_json::json;
use tower::ServiceExt;

use crate::{
    helpers::{
        create_project_for_user::create_project_for_user,
        create_task_for_project::create_task_for_project,
        helper_setup_app::{
            add_member_to_project, get_auth_token, get_auth_token_and_id, setup_app,
        },
    },
    models::user_model::UserData,
};

#[tokio::test]
async fn test_project_membership_flow() {
    let app = setup_app().await;

    // tres usuarios: dueño, futuro miembro y no miembro
    let owner_email = format!("owner-{}@test.com", uuid::Uuid::new());
    let member_email = format!("member-{}@test.com", uuid::Uuid::new());
    let non_member_email = format!("stranger-{}@test.com", uuid::Uuid::new());

    let owner_token = get_auth_token(&app, "owner", &owner_email).await;
    let (member_token, member_id) = get_auth_token_and_id(&app, "member_man", &member_email).await;
    let non_member_token = get_auth_token(&app, "stranger", &non_member_email).await;
    // El dueño crea un proyecto
    let project_id = create_project_for_user(&app, &owner_token, "MEMBERS").await;
    let task_id = create_task_for_project(&app, &owner_token, &project_id, None).await;

    // Add member to the project before attempting to update the task
    add_member_to_project(&app, &owner_token, &project_id, &member_email).await;

    let update_payload = json!({"title": "Tarea actualizada por asignado"});
    let update_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/tasks/{}", task_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", member_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(update_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_resp.status(), StatusCode::OK);

    let list_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/projects/{}/members", project_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", owner_token))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let members: Vec<UserData> =
        serde_json::from_slice(&to_bytes(list_resp.into_body(), 1024 * 1024).await.unwrap())
            .unwrap();
    assert_eq!(members.len(), 2);

    let delete_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/projects/{}/members/{}",
                    project_id, member_id
                ))
                .header(header::AUTHORIZATION, format!("Bearer {}", owner_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);

    // --- VERIFICACIÓN FINAL ---
    // El ex-miembro ya no puede acceder a las tareas del proyecto.
    let final_access_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/tasks/{}", task_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", member_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(final_access_resp.status(), StatusCode::UNAUTHORIZED);

    // Prueba de permisos para añadir
    // Un extraño no puede añadir miembros
    let add_payload = json!({"email": member_email});
    let stranger_add_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/projects/{}/members", project_id))
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", non_member_token),
                )
                .header(header::CONTENT_TYPE, "application/json")
                .body(axum::body::Body::from(add_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(stranger_add_resp.status(), StatusCode::UNAUTHORIZED);

    // El dueño añade al futuro miembro. Debe de funcionar
    let owner_add_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/projects/{}/members", project_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", owner_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(axum::body::Body::from(add_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(owner_add_resp.status(), StatusCode::OK);

    // PRUEBA DE ACCESO DE MIEMBROS
    // el nuevo miembro ahora intenta crear una tarea. DEBE DE FUNCIONAR
    let task_payload = json!({ "title": "Tarea del miembro" });
    let member_create_task_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/projects/{}/tasks", project_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", member_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(task_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(member_create_task_resp.status(), StatusCode::CREATED);
}
