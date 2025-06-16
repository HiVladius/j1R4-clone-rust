use axum::extract::Request;
use bson::uuid;
use hyper::{header, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use crate::{
    helpers::{
        helper_setup_app::{get_auth_token, setup_app},
        create_project_for_user::create_project_for_user},
    
};
use axum::body::Body;



#[tokio::test]
async fn test_project_membership_flow(){
    let app = setup_app().await;

    // tres usuarios: dueño, futuro miembro y no miembro
    let owner_email = format!("owner-{}@test.com", uuid::Uuid::new());
    let member_email = format!("member-{}@test.com", uuid::Uuid::new());
    let non_member_email = format!("stranger-{}@test.com", uuid::Uuid::new());

    let owner_token = get_auth_token(&app, "owner", &owner_email).await;
    let member_token = get_auth_token(&app, "member", &member_email).await;
    let non_member_token = get_auth_token(&app, "stranger", &non_member_email).await;

    // El dueño crea un proyecto
    let project_id = create_project_for_user(&app, &owner_token, "MEMBERS").await;


    // Prueba de permisos para añadir
    // Un extraño no puede añadir miembros
    let add_payload = json!({"email": member_email});
    let stranger_add_resp = app.clone().oneshot(
        Request::builder()
            .method("POST").uri(format!("/api/projects/{}/members", project_id))
            .header(header::AUTHORIZATION, format!("Bearer {}", non_member_token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::from(add_payload.to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(stranger_add_resp.status(), StatusCode::UNAUTHORIZED);

    // El dueño añade al futuro miembro. Debe de funcionar
    let owner_add_resp = app.clone().oneshot(
        Request::builder()
            .method("POST").uri(format!("/api/projects/{}/members", project_id))
            .header(header::AUTHORIZATION, format!("Bearer {}", owner_token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::from(add_payload.to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(owner_add_resp.status(), StatusCode::OK);

    // PRUEBA DE ACCESO DE MIEMBROS
    // el nuevo miembro ahora intenta crear una tarea. DEBE DE FUNCIONAR
    let task_payload = json!({ "title": "Tarea del miembro" });
    let member_create_task_resp = app.clone().oneshot(
        Request::builder()
            .method("POST").uri(format!("/api/projects/{}/tasks", project_id))
            .header(header::AUTHORIZATION, format!("Bearer {}", member_token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(task_payload.to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(member_create_task_resp.status(), StatusCode::CREATED);
}