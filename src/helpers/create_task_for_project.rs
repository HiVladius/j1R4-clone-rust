use crate::models::task_model::Task;
use axum::{
    body::{Body, to_bytes},
    http::{Request, header},
};
use serde_json::json;
use tower::ServiceExt;

pub async fn create_task_for_project(
    app: &axum::Router,
    token: &str,
    project_id: &str,
    assignee_id: Option<String>,
) -> String {
    let mut payload = json!({
        "title": "Test Task",
        "description": "Task created for testing purposes",
        "status": "ToDo",
        "priority": "Medium"
    });

    // Si se proporciona un assignee_id, lo añadimos al payload
    if let Some(assignee) = assignee_id {
        payload["assignee_id"] = json!(assignee);
    }

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/projects/{}/tasks", project_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let task: Task =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();

    task.id.unwrap().to_hex()
}
