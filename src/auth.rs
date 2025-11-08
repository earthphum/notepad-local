use crate::{
    logging,
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
    // Get admin user from environment
    let admin_user = match env::var("ADMIN_USER") {
        Ok(user) => {
            if user.is_empty() {
                logging::log_config_error("ADMIN_USER", "empty value");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Server configuration error" })),
                );
            }
            user
        }
        Err(_) => {
            logging::log_config_error("ADMIN_USER", "not set");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Server configuration error" })),
            );
        }
    };

    // Get admin password hash from environment
    let admin_hash = match env::var("ADMIN_PASS_HASH") {
        Ok(mut hash) => {
            // Remove surrounding quotes if present
            if hash.starts_with('"') && hash.ends_with('"') {
                hash = hash.trim_matches('"').to_string();
            }

            // Validate hash format in production
            let is_development =
                env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) != "production";

            if !is_development {
                // In production, only log hash format validation issues
                if !hash.starts_with("$2b$")
                    && !hash.starts_with("$2a$")
                    && !hash.starts_with("$2y$")
                {
                    logging::log_config_error("ADMIN_PASS_HASH", "invalid bcrypt format");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Server configuration error" })),
                    );
                }

                if hash.len() != 60 {
                    logging::log_config_error("ADMIN_PASS_HASH", "invalid bcrypt hash length");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Server configuration error" })),
                    );
                }
            }

            hash
        }
        Err(_) => {
            logging::log_config_error("ADMIN_PASS_HASH", "not set");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Server configuration error" })),
            );
        }
    };

    // Verify credentials
    let username_match = req.username == admin_user;
    let password_match = verify_password(&admin_hash, &req.password);

    // Log authentication attempt (without exposing sensitive data)
    if username_match && password_match {
        match generate_token(&req.username) {
            Ok(token) => {
                logging::log_auth_success(&req.username);
                (StatusCode::OK, Json(json!({ "token": token })))
            }
            Err(e) => {
                logging::log_auth_error("token generation", &e.to_string());
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Token generation failed" })),
                )
            }
        }
    } else {
        // Log failed authentication attempt without exposing details
        let failure_reason = if !username_match {
            "invalid username"
        } else {
            "invalid password"
        };
        logging::log_auth_failure(&req.username, failure_reason);

        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid credentials" })),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_deserialization() {
        let json_data = r#"{"username":"testuser","password":"testpass"}"#;
        let login_req: LoginRequest = serde_json::from_str(json_data).unwrap();
        assert_eq!(login_req.username, "testuser");
        assert_eq!(login_req.password, "testpass");
    }

    #[test]
    fn test_environment_variable_validation() {
        // Test that we can validate missing environment variables
        // This is a unit test for validation logic
        let admin_user = std::env::var("ADMIN_USER");
        assert!(admin_user.is_err() || admin_user.unwrap().is_empty());
    }
}
