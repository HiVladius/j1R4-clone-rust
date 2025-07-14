use axum::{
    body::{Body, to_bytes},
    extract::Request,
};
use bson::uuid;
use hyper::{StatusCode, header};
use serde_json::Value;
use tower::ServiceExt;

use crate::{
    helpers::{
        create_project_for_user::create_project_for_user,
        helper_setup_app::{
            add_member_to_project, get_auth_token, setup_app,
        },
    },
    models::project_models::ProjectWithRole,
};

#[tokio::test]
async fn test_get_projects_includes_owned_and_member_projects() {
    let app = setup_app().await;

    // Crear tres usuarios
    let user1_email = format!("user1-{}@test.com", uuid::Uuid::new());
    let user2_email = format!("user2-{}@test.com", uuid::Uuid::new());
    let user3_email = format!("user3-{}@test.com", uuid::Uuid::new());

    let user1_token = get_auth_token(&app, "user1", &user1_email).await;
    let user2_token = get_auth_token(&app, "user2", &user2_email).await;
    let user3_token = get_auth_token(&app, "user3", &user3_email).await;

    // User1 crea un proyecto (será owner)
    let _project1_id = create_project_for_user(&app, &user1_token, "PROJ1").await;

    // User2 crea un proyecto (será owner)
    let project2_id = create_project_for_user(&app, &user2_token, "PROJ2").await;

    // User3 crea un proyecto (será owner)
    let _project3_id = create_project_for_user(&app, &user3_token, "PROJ3").await;

    // Agregar User1 como miembro al proyecto de User2
    add_member_to_project(&app, &user2_token, &project2_id, &user1_email).await;

    // Verificar que User1 ve tanto su proyecto propio como el proyecto donde es miembro
    let projects_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/projects")
                .header(header::AUTHORIZATION, format!("Bearer {}", user1_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(projects_resp.status(), StatusCode::OK);

    let projects: Vec<ProjectWithRole> =
        serde_json::from_slice(&to_bytes(projects_resp.into_body(), 1024 * 1024).await.unwrap())
            .unwrap();

    // User1 debe ver exactamente 2 proyectos
    assert_eq!(projects.len(), 2);

    // Verificar que los roles son correctos
    let mut has_owned_project = false;
    let mut has_member_project = false;

    for project in projects {
        if project.project_key == "PROJ1" {
            // Este es el proyecto que User1 creó, debe ser owner
            assert!(matches!(project.user_role, crate::models::project_models::UserRole::Owner));
            has_owned_project = true;
        } else if project.project_key == "PROJ2" {
            // Este es el proyecto donde User1 es miembro
            assert!(matches!(project.user_role, crate::models::project_models::UserRole::Member));
            has_member_project = true;
        }
    }

    assert!(has_owned_project, "User1 debería ver su proyecto propio");
    assert!(has_member_project, "User1 debería ver el proyecto donde es miembro");

    // Verificar que User3 solo ve su propio proyecto
    let user3_projects_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/projects")
                .header(header::AUTHORIZATION, format!("Bearer {}", user3_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(user3_projects_resp.status(), StatusCode::OK);

    let user3_projects: Vec<Value> =
        serde_json::from_slice(&to_bytes(user3_projects_resp.into_body(), 1024 * 1024).await.unwrap())
            .unwrap();

    // User3 solo debe ver 1 proyecto (el suyo)
    assert_eq!(user3_projects.len(), 1);
    assert_eq!(user3_projects[0]["key"], "PROJ3");
    assert_eq!(user3_projects[0]["user_role"], "owner");
}

#[tokio::test]
async fn test_project_role_metadata_consistency() {
    let app = setup_app().await;

    // Crear dos usuarios
    let owner_email = format!("owner-{}@test.com", uuid::Uuid::new());
    let member_email = format!("member-{}@test.com", uuid::Uuid::new());

    let owner_token = get_auth_token(&app, "owner", &owner_email).await;
    let member_token = get_auth_token(&app, "member", &member_email).await;

    // Owner crea un proyecto
    let project_id = create_project_for_user(&app, &owner_token, "ROLES").await;

    // Agregar member al proyecto
    add_member_to_project(&app, &owner_token, &project_id, &member_email).await;

    // Verificar que el owner ve el proyecto con rol "owner"
    let owner_projects_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/projects")
                .header(header::AUTHORIZATION, format!("Bearer {}", owner_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let owner_projects: Vec<Value> =
        serde_json::from_slice(&to_bytes(owner_projects_resp.into_body(), 1024 * 1024).await.unwrap())
            .unwrap();

    assert_eq!(owner_projects.len(), 1);
    assert_eq!(owner_projects[0]["user_role"], "owner");

    // Verificar que el member ve el proyecto con rol "member"
    let member_projects_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/projects")
                .header(header::AUTHORIZATION, format!("Bearer {}", member_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let member_projects: Vec<Value> =
        serde_json::from_slice(&to_bytes(member_projects_resp.into_body(), 1024 * 1024).await.unwrap())
            .unwrap();

    assert_eq!(member_projects.len(), 1);
    assert_eq!(member_projects[0]["user_role"], "member");
    assert_eq!(member_projects[0]["key"], "ROLES");
}
