use crate::{
    handlers::{
        auth_handler::{get_me_handler, login_handler, register_handler},
        project_handler::{create_project_handler, get_project_handler},
    },
    middleware::auth_middleware::auth_guard,
    state::AppState,
};
use axum::{
    Router, middleware,
    routing::{get, post},
};
use std::sync::Arc;

pub fn get_app(app_state: Arc<AppState>) -> Router {
    let auth_middleware = middleware::from_fn_with_state(app_state.clone(), auth_guard);

    let protected_routes = Router::new()
        .route("/me", get(get_me_handler))
        .route("/projects", post(create_project_handler))
        .route("/projects", get(get_project_handler))
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