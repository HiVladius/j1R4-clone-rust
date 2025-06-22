use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
// use bson::uuid;
use serde_json::json;
use tower::ServiceExt;
// use uuid::Uuid;

use crate::{
    helpers::{ helper_setup_app::{ create_project_for_user, create_task_for_project, get_auth_token, get_auth_token_and_id, setup_app}}, models::comment_model::CommentData
};


#[tokio::test]
async fn test_comment_edit_and_delete_permissions(){
    let app = setup_app().await;
    let (auth_token, _) = get_auth_token_and_id(&app, "author", "author@test.com").await;
    let author_user_token = get_auth_token(&app, "other", "other@test.com").await;
    

    let project_id = create_project_for_user(&app, &auth_token, "CEDIT").await;
    let task_id = create_task_for_project(&app, &auth_token, &project_id, None).await;

    let comment_payload = json!({"content": "Comentario original"});
    let create_resp = app.clone().oneshot(
        Request::builder()
        .method("POST").uri(format!("/api/tasks/{}/comments", task_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", auth_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(comment_payload.to_string())).unwrap()
    ).await.unwrap();

    let comment: CommentData = serde_json::from_slice(&to_bytes(create_resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    let comment_id = comment.id;

    let update_payload = json!({"content": "Contenido actualizado"});

    let unauthorized_update = app.clone().oneshot(
        Request::builder()
            .method("PATCH")
            .uri(format!("/api/tasks/{}/comments/{}", task_id, comment_id))
            .header(header::AUTHORIZATION, format!("Bearer {}", author_user_token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(update_payload.to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(unauthorized_update.status(), StatusCode::UNAUTHORIZED);

    let authorized_delete = app.clone().oneshot(
        Request::builder()
            .method("DELETE")
            .uri(format!("/api/tasks/{}/comments/{}", task_id, comment_id))
            .header(header::AUTHORIZATION, format!("Bearer {}", auth_token))
            .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(authorized_delete.status(), StatusCode::NO_CONTENT);

    let list_resp  = app.clone().oneshot(
        Request::builder()
            .method("GET")
            .uri(format!("/api/tasks/{}/comments", task_id))
            .header(header::AUTHORIZATION, format!("Bearer {}", auth_token))
            .body(Body::empty()).unwrap()
    ).await.unwrap();

    let final_comments: Vec<CommentData> = serde_json::from_slice(&to_bytes(list_resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert!(final_comments.is_empty(), "El comentario no debería existir después de eliminarlo");
}

