pub mod config;
pub mod db;
pub mod errors;
pub mod state;

// Hasheo de contraseñas
pub mod utils {
    pub mod jwt_utils;
    pub mod password_utils;
}

pub mod services {
    pub mod auth_service;
}

//Modelos para la base de datos
pub mod models {
    pub mod user_model;
}

pub mod handlers {
    pub mod auth_handler;
}

pub mod middleware {
    pub mod auth_middleware;
}
