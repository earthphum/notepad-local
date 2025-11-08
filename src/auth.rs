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
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Server configuration error" })),
            );
        }
    };

    let admin_hash = match env::var("ADMIN_PASS_HASH") {
        Ok(hash) => hash,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Server configuration error" })),
            );
        }
    };

    if req.username == admin_user && verify_password(&admin_hash, &req.password) {
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
