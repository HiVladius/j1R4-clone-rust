use crate::helpers::helper_setup_app::{get_auth_token, setup_app};

use crate::models::project_models::Project;
use axum::body::to_bytes;
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use bson::uuid;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_project_update_and_delete_permissions() {
    // //* --- 1. SETUP ---
    // Creamos dos usuarios diferentes para probar los permisos cruzados.
    let app = setup_app().await;

    let user_a_email = format!("owner-{}@test.com", Uuid::new());
    let user_b_email = format!("other-{}@test.com", Uuid::new());

    let token_a = get_auth_token(&app, "user_a", &user_a_email).await;
    let token_b = get_auth_token(&app, "user_b", &user_b_email).await;

    // //* --- 2. CREACIÓN ---
    // El usuario A crea un proyecto.
    let create_payload = json!({ "name": "Proyecto de A", "key": "PROJA" });
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/projects")
                .header(header::AUTHORIZATION, format!("Bearer {}", token_a))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_payload.to_string()))
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
    let project_id = project.id.unwrap().to_hex();

    //  //* --- 3. PRUEBAS DE UPDATE ---
    // //! El usuario B (no propietario) intenta actualizar -> DEBE FALLAR (401 Unauthorized)
    let update_payload = json!({ "name": "Proyecto Hackeado" });
    let unauthorized_update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/projects/{}", project_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", token_b))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(update_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        unauthorized_update_response.status(),
        StatusCode::UNAUTHORIZED
    );

    // //!El usuario A (propietario) actualiza -> DEBE FUNCIONAR
    let authorized_update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/projects/{}", project_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", token_a))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(update_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(authorized_update_response.status(), StatusCode::OK);
    let updated_project: Project = serde_json::from_slice(
        &to_bytes(authorized_update_response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(updated_project.name, "Proyecto Hackeado");

    // //* --- 4. PRUEBAS DE DELETE ---
    // El usuario B intenta eliminar -> DEBE FALLAR (401 Unauthorized)
    let unauthorized_delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/projects/{}", project_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", token_b))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        unauthorized_delete_response.status(),
        StatusCode::UNAUTHORIZED
    );

    // //!  El usuario A elimina -> DEBE FUNCIONAR
    let authorized_delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/projects/{}", project_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", token_a))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(authorized_delete_response.status(), StatusCode::NO_CONTENT);

    // //* --- 5. VERIFICACIÓN FINAL ---
    // //!  El proyecto ya no debería existir en la lista del usuario A
    let final_get_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/projects")
                .header(header::AUTHORIZATION, format!("Bearer {}", token_a))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let final_projects: Vec<Project> = serde_json::from_slice(
        &to_bytes(final_get_response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(final_projects.is_empty());
}
