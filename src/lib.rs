pub mod config;
pub mod db;
pub mod errors;
pub mod state;

// Hasheo de contraseñas
pub mod utils {
    pub mod jwt_utils;
    pub mod password_utils;
    pub mod validation;
}

pub mod services {
    pub mod auth_service;
    pub mod comment_service;
    pub mod date_range_service;
    pub mod permission_service;
    pub mod project_service;
    pub mod task_service;
}

//Modelos para la base de datos
pub mod models {
    pub mod comment_model;
    pub mod project_models;
    pub mod task_model;
    pub mod user_model;
}

pub mod handlers {
    pub mod auth_handler;
    pub mod comment_handler;
    pub mod date_range_handler;
    pub mod project_handler;
    pub mod task_handler;
    pub mod websocket_handler;
}

pub mod middleware {
    pub mod auth_middleware;
}

#[cfg(test)]
pub mod test {
    pub mod comment_edit_test;
    pub mod comment_integration_test;
    pub mod project_edit_test;
    pub mod project_integration_test;
    pub mod project_membership_test;
    pub mod task_creation_test;
    pub mod task_edit_test;
    pub mod task_read_test;
}

pub mod router {
    #[allow(clippy::module_inception)]
    pub mod router;
}

pub mod helpers {
    pub mod create_project_for_user;
    pub mod create_task_for_project;
    pub mod helper_setup_app;
}
