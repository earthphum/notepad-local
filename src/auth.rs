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
    let admin_user = env::var("ADMIN_USER").unwrap();
    let admin_hash = env::var("ADMIN_PASS_HASH").unwrap();

    if req.username == admin_user && verify_password(&admin_hash, &req.password) {
        let token = generate_token(&req.username);
        return (StatusCode::OK, Json(json!({ "token": token })));
    }
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "Invalid credentials" })),
    )
}
