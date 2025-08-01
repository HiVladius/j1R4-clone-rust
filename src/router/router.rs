use crate::{
    handlers::{
        auth_handler::{get_me_handler, login_handler, register_handler, update_me_handler},
        comment_handler::{
            create_comment_handler, delete_comment_handler, get_comments_handler,
            update_comment_handler,
        },
        date_range_handler::{
            delete_task_date_range_handler, get_project_date_ranges_handler,
            get_task_date_range_handler, set_task_date_range_handler,
            update_task_date_range_handler,
        },
        image_handler::{
            delete_image_handler, download_image_handler, get_image_info_handler,
            list_project_images_handler, list_task_images_handler, list_user_images_handler,
            update_image_handler, upload_image_handler,
        },
        project_handler::{
            add_member_handler, create_project_handler, delete_project_handler,
            get_project_handler, list_members_handler, remove_member_handler,
            update_project_handler,
        },
        task_handler::{
            create_task_handler, delete_task_handler, get_task_by_id_handler,
            get_task_for_project_handler, get_task_with_date_range_handler, update_task_handler,
        },
        websocket_handler::websocket_handler,
    },
    middleware::auth_middleware::auth_guard,
    state::AppState,
};
use axum::{
    Router, middleware,
    routing::{delete, get, patch, post, put},
};
use std::sync::Arc;

use axum::http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

pub fn get_app(app_state: Arc<AppState>) -> Router {
    let auth_middleware = middleware::from_fn_with_state(app_state.clone(), auth_guard);

    let cors = CorsLayer::new()
        .allow_origin(
            app_state
                .config
                .cors_origins
                .iter()
                .map(|origin| origin.parse::<HeaderValue>().unwrap())
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true);

    let protected_routes = Router::new()
        .route("/me", get(get_me_handler))
        .route("/me", put(update_me_handler))
        .route("/projects", post(create_project_handler))
        .route("/projects", get(get_project_handler))
        .route("/projects/{project_id}", patch(update_project_handler))
        .route("/projects/{project_id}", delete(delete_project_handler))
        .route(
            "/projects/{project_id}/tasks",
            get(get_task_for_project_handler),
        )
        .route("/projects/{project_id}/tasks", post(create_task_handler))
        .route("/tasks/{task_id}", get(get_task_by_id_handler))
        .route(
            "/tasks/{task_id}/full",
            get(get_task_with_date_range_handler),
        )
        .route("/tasks/{task_id}", patch(update_task_handler))
        .route("/tasks/{task_id}", delete(delete_task_handler))
        .route("/projects/{project_id}/members", post(add_member_handler))
        .route("/projects/{project_id}/members", get(list_members_handler))
        .route(
            "/projects/{project_id}/members/{member_id}",
            delete(remove_member_handler),
        )
        .route("/tasks/{task_id}/comments", post(create_comment_handler))
        .route("/tasks/{task_id}/comments", get(get_comments_handler))
        .route(
            "/tasks/{task_id}/comments/{comment_id}",
            patch(update_comment_handler),
        )
        .route(
            "/tasks/{task_id}/comments/{comment_id}",
            delete(delete_comment_handler),
        )
        .route(
            "/tasks/{task_id}/date-range",
            post(set_task_date_range_handler),
        )
        .route(
            "/tasks/{task_id}/date-range",
            get(get_task_date_range_handler),
        )
        .route(
            "/tasks/{task_id}/date-range",
            patch(update_task_date_range_handler),
        )
        .route(
            "/tasks/{task_id}/date-range",
            delete(delete_task_date_range_handler),
        )
        .route(
            "/projects/{project_id}/date-ranges",
            get(get_project_date_ranges_handler),
        )
        .route("/images", post(upload_image_handler))
        .route("/images", get(list_user_images_handler))
        .route("/images/{image_id}", get(get_image_info_handler))
        .route("/images/{image_id}/download", get(download_image_handler))
        .route("/images/{image_id}", patch(update_image_handler))
        .route("/images/{image_id}", delete(delete_image_handler))
        .route("/projects/{project_id}/images", get(list_project_images_handler))
        .route("/tasks/{task_id}/images", get(list_task_images_handler))
        .layer(auth_middleware);

    let auth_routes = Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler));

    Router::new()
        .route("/ws", get(websocket_handler))
        .route("/", get(root_handler))
        .nest("/api/auth", auth_routes)
        .nest("/api", protected_routes)
        .layer(cors)
        .with_state(app_state)
}

async fn root_handler() -> &'static str {
    "Welcome to the manager projects!"
}
