use axum::http::HeaderMap;
use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{DecodingKey, Validation, decode};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn generate_token(username: &str) -> Result<String, Box<dyn std::error::Error>> {
    let secret = env::var("JWT_SECRET").map_err(|_| "JWT_SECRET environment variable not set")?;

    // Validate JWT secret in production
    let is_development =
        env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) != "production";
    if !is_development && secret.len() < 32 {
        return Err("JWT_SECRET must be at least 32 characters in production".into());
    }

    let claims = Claims {
        sub: username.to_string(),
        exp: (chrono::Utc::now().timestamp() + 3600) as usize, // 1 hour expiration
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

#[allow(dead_code)]
pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).unwrap()
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    // Remove all sensitive logging - only log errors in production

    // Check if hash looks like valid bcrypt format
    if !hash.starts_with("$2b$") && !hash.starts_with("$2a$") && !hash.starts_with("$2y$") {
        // Log this in development only for debugging
        let is_development =
            env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) != "production";
        if is_development {
            eprintln!("❌ Hash doesn't start with bcrypt identifier ($2b$, $2a$, or $2y$)");
        }
        return false;
    }

    match verify(password, hash) {
        Ok(result) => result,
        Err(e) => {
            // In production, we don't want to expose hash details in logs
            // Just log that verification failed without sensitive data
            let is_development =
                env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) != "production";
            if is_development {
                eprintln!("❌ Password verification error: {}", e);
                eprintln!("💡 Hash length: {} characters", hash.len());
            }
            false
        }
    }
}

pub fn extract_user_from_token(headers: &HeaderMap) -> Result<String, Box<dyn std::error::Error>> {
    let auth_header = headers
        .get("authorization")
        .ok_or("Missing authorization header")?
        .to_str()
        .map_err(|_| "Invalid authorization header format")?;

    if !auth_header.starts_with("Bearer ") {
        return Err("Invalid authorization header format".into());
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix
    let secret = env::var("JWT_SECRET").map_err(|_| "JWT_SECRET environment variable not set")?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims.sub)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "test_password";
        let hash = hash_password(password);

        // Verify the hash starts with bcrypt identifier
        assert!(hash.starts_with("$2b$"));

        // Verify the hash can verify the original password
        assert!(verify_password(&hash, password));

        // Verify the hash rejects wrong passwords
        assert!(!verify_password(&hash, "wrong_password"));
    }

    #[test]
    fn test_verify_password_invalid_format() {
        let invalid_hash = "invalid_hash_format";
        assert!(!verify_password(invalid_hash, "password"));
    }

    #[test]
    fn test_generate_token_success() {
        unsafe {
            env::set_var(
                "JWT_SECRET",
                "test_secret_key_for_testing_that_is_long_enough",
            );
        }

        let result = generate_token("testuser");
        assert!(result.is_ok());

        let token = result.unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_generate_token_no_secret() {
        unsafe {
            env::remove_var("JWT_SECRET");
        }

        let result = generate_token("testuser");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_token_short_secret_in_production() {
        unsafe {
            env::set_var("RUST_ENV", "production");
        }
        unsafe {
            env::set_var("JWT_SECRET", "short");
        }

        let result = generate_token("testuser");
        assert!(result.is_err());

        // Clean up
        unsafe {
            env::remove_var("RUST_ENV");
        }
    }
}
