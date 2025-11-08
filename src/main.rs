mod auth;
mod content;
mod db;
mod logging;
mod models;
mod state;
mod utils;

use axum::{
    Router,
    middleware::from_fn,
    response::Json,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

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

    // Build admin routes (authentication handled in each handler)
    let admin_router = Router::new()
        .route("/contents", get(content::get_all_contents))
        .route("/contents", post(content::create_content))
        .route("/contents/:id", get(content::get_content_by_id_admin))
        .route("/contents/:id", put(content::update_content))
        .route("/contents/:id", delete(content::delete_content))
        .route("/stats", get(content::get_stats));

    // Build the application with routes
    let app = Router::new()
        // Public routes (no authentication required)
        .route("/", get(root_handler))
        .route("/health", get(health_check))
        .route("/contents", get(content::get_public_contents))
        .route("/contents/:id", get(content::get_content_by_id))
        // Authentication route
        .route("/login", post(auth::login))
        // Nest admin routes under /admin
        .nest("/admin", admin_router)
        .with_state(state)
        // Apply request logging middleware to all routes
        .layer(from_fn(logging::middleware::request_logging_middleware))
        .layer(from_fn(logging::middleware::error_logging_middleware));

    // Log application startup
    logging::log_app_startup(&host, port);
    println!("🚀 Notepad API server starting on {}:{}", host, port);

    let listener = tokio::net::TcpListener::bind((host, port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Root handler
async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "message": "Welcome to Notepad Content Management API",
        "version": "2.0.0",
        "features": {
            "public_notes": "GET /contents - Get all public notes",
            "public_note_by_id": "GET /contents/:id - Get specific public note",
            "authentication": "POST /login - Admin login",
            "user_notes": "GET /admin/contents - Get all user notes (auth required)",
            "create_note": "POST /admin/contents - Create new note (auth required)",
            "update_note": "PUT /admin/contents/:id - Update note (auth required)",
            "delete_note": "DELETE /admin/contents/:id - Delete note (auth required)",
            "stats": "GET /admin/stats - Get user statistics (auth required)"
        }
    }))
}

// Health check endpoint
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "notepad-api",
        "version": "2.0.0"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
    };

    #[tokio::test]
    async fn test_root_handler() {
        let app = Router::new().route("/", get(root_handler));

        let request = Request::builder().uri("/").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = Router::new().route("/health", get(health_check));

        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
