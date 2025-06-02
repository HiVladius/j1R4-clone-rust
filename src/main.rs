use axum::{routing::get, Router};
use dotenvy::dotenv;
use jira_clone_backend::config::Config;
use jira_clone_backend::db::DatabaseState;
use jira_clone_backend::errors::AppError;


fn main() {
    dotenv().ok();
    
}
