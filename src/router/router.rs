use crate::{
    handlers::{
        auth_handler::{get_me_handler, login_handler, register_handler},
        project_handler::{
            add_member_handler, create_project_handler, delete_project_handler,
            get_project_handler, update_project_handler,
        },
        task_handler::{
            create_task_handler, delete_task_handler, get_task_by_id_handler,
            get_task_for_project_handler, update_task_handler,
        },
    },
    middleware::auth_middleware::auth_guard,
    state::AppState,
};
use axum::{
    Router, middleware,
    routing::{delete, get, patch, post},
};
use std::sync::Arc;

pub fn get_app(app_state: Arc<AppState>) -> Router {
    let auth_middleware = middleware::from_fn_with_state(app_state.clone(), auth_guard);

    let protected_routes = Router::new()
        .route("/me", get(get_me_handler))
        .route("/projects", post(create_project_handler))
        .route("/projects", get(get_project_handler))
        .route("/projects/{project_id}", patch(update_project_handler))
        .route("/projects/{project_id}", delete(delete_project_handler))
        .route("/projects/{project_id}/tasks",get(get_task_for_project_handler))
        .route("/projects/{project_id}/tasks", post(create_task_handler))
        .route("/tasks/{task_id}", get(get_task_by_id_handler))
        .route("/tasks/{task_id}", patch(update_task_handler))
        .route("/tasks/{task_id}", delete(delete_task_handler))
        .route("/projects/{project_id}/members", post(add_member_handler))
        .layer(auth_middleware);

    let auth_routes = Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler));

    Router::new()
        .route("/", get(root_handler))
        .nest("/api/auth", auth_routes)
        .nest("/api", protected_routes)
        .with_state(app_state)
}

async fn root_handler() -> &'static str {
    "Welcome to the Jira Clone Backend!"
}
