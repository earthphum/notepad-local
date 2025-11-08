use crate::{
    state::AppState,
    utils::{generate_token, verify_password},
};
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use std::env;

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn login(
    State(_state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let admin_user = match env::var("ADMIN_USER") {
        Ok(user) => user,
        Err(_) => {
            eprintln!("❌ ADMIN_USER environment variable not set");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Server configuration error" })),
            );
        }
    };

    let admin_hash = match env::var("ADMIN_PASS_HASH") {
        Ok(mut hash) => {
            // Remove surrounding quotes if present
            if hash.starts_with('"') && hash.ends_with('"') {
                hash = hash.trim_matches('"').to_string();
            }
            eprintln!(
                "🔍 Loaded ADMIN_PASS_HASH: '{}' (length: {})",
                hash,
                hash.len()
            );
            hash
        }
        Err(_) => {
            eprintln!("❌ ADMIN_PASS_HASH environment variable not set");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Server configuration error" })),
            );
        }
    };

    eprintln!(
        "🔍 Login attempt - Username: '{}', Provided password length: {}",
        req.username,
        req.password.len()
    );
    eprintln!("🔍 Configured admin user: '{}'", admin_user);
    eprintln!("🔍 Stored hash: '{}'", admin_hash);

    let username_match = req.username == admin_user;
    let password_match = verify_password(&admin_hash, &req.password);

    eprintln!(
        "🔍 Username match: {}, Password match: {}",
        username_match, password_match
    );

    if username_match && password_match {
        match generate_token(&req.username) {
            Ok(token) => {
                return (StatusCode::OK, Json(json!({ "token": token })));
            }
            Err(e) => {
                eprintln!("Token generation failed: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Token generation failed" })),
                );
            }
        }
    }
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "Invalid credentials" })),
    )
}
