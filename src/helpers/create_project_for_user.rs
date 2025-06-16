use axum::{body::{Body, to_bytes}, http::{Request,  header}};
use serde_json::json;
use tower::ServiceExt;
use crate::{
    
    models::{
        project_models::Project,
    },
};


pub async fn create_project_for_user(app: &axum::Router, token: &str, key: &str) -> String {
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