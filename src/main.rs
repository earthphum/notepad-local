mod auth;
mod db;
mod logging;
mod models;
mod notes;
mod state;
mod utils;

use axum::{Router, middleware, routing::get, routing::post};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize logging first
    logging::init_logging();

    // Load environment variables
    dotenv::dotenv().ok();

    // Connect to database
    let db = match db::connect_db().await {
        Ok(pool) => {
            logging::log_db_operation("connection", "mysql");
            Arc::new(pool)
        }
        Err(e) => {
            logging::log_db_error("connection", &e.to_string());
            eprintln!("Failed to connect to database: {}", e);
            return;
        }
    };

    let state = state::AppState { db };

    // Get server configuration
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    // Build the application with middleware
    let app = Router::new()
        .route("/login", post(auth::login))
        .route("/notes", get(notes::get_notes).post(notes::create_note))
        .layer(middleware::from_fn(
            logging::middleware::request_logging_middleware,
        ))
        .layer(middleware::from_fn(
            logging::middleware::error_logging_middleware,
        ))
        .with_state(state);

    // Log application startup
    logging::log_app_startup(&host, port);
    info!("🚀 Notepad API server starting on {}:{}", host, port);

    let listener = tokio::net::TcpListener::bind((host, port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
