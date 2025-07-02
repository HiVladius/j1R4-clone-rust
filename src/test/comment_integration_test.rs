use axum::body::{Body, to_bytes};
use axum::extract::Request;
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
    models::comment_model::CommentData,
};

#[tokio::test]
async fn test_comment_flow_and_permissions() {
    let app = setup_app().await;
    let owner_token = get_auth_token(&app, "owner_comm", "owner.comm@test.com").await;
    let (member_token, _) =
        get_auth_token_and_id(&app, "member_comm", "member.comm@test.com").await;
    let stranger_token = get_auth_token(&app, "stranger_comm", "stranger.comm@test.com").await;

    let project_id = create_project_for_user(&app, &owner_token, "COMM").await;
    add_member_to_project(&app, &owner_token, &project_id, "member.comm@test.com").await;
    let task_id = create_task_for_project(&app, &owner_token, &project_id, None).await;

    // --- PRUEBA CREAR COMENTARIO ---
    // Un extraño intenta comentar -> DEBE FALLAR
    let comment_payload = json!({ "content": "Soy un extraño" });
    let stranger_comment_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/tasks/{}/comments", task_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", stranger_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(comment_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(stranger_comment_resp.status(), StatusCode::UNAUTHORIZED);

    // El dueño comenta -> DEBE FUNCIONAR
    let owner_comment_payload = json!({ "content": "Soy el dueño" });
    let owner_comment_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/tasks/{}/comments", task_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", owner_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(owner_comment_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(owner_comment_resp.status(), StatusCode::CREATED);

    // Un miembro comenta -> DEBE FUNCIONAR
    let member_comment_payload = json!({ "content": "Soy un miembro" });
    let member_comment_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/tasks/{}/comments", task_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", member_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(member_comment_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(member_comment_resp.status(), StatusCode::CREATED);

    // --- PRUEBA LISTAR COMENTARIOS ---
    // El dueño lista los comentarios -> DEBE FUNCIONAR y obtener 2 comentarios
    let list_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/tasks/{}/comments", task_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", owner_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let comments: Vec<CommentData> =
        serde_json::from_slice(&to_bytes(list_resp.into_body(), usize::MAX).await.unwrap())
            .unwrap();
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].content, "Soy el dueño");
    assert_eq!(comments[0].author.username, "owner_comm");
    assert_eq!(comments[1].content, "Soy un miembro");
    assert_eq!(comments[1].author.username, "member_comm");
}
