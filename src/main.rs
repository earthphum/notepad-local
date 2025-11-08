mod auth;
mod db;
mod models;
mod notes;
mod state;
mod utils;

use axum::{Router, routing::get, routing::post};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let db = db::connect_db().await;
    let state = state::AppState { db: Arc::new(db) };

    let app = Router::new()
        .route("/login", post(auth::login))
        .route("/notes", get(notes::get_notes).post(notes::create_note))
        .with_state(state);

    println!("🚀 Running on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
