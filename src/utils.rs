use bcrypt::{DEFAULT_COST, hash, verify};
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
    let claims = Claims {
        sub: username.to_string(),
        exp: (chrono::Utc::now().timestamp() + 3600) as usize,
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
    eprintln!("🔐 Verifying password with bcrypt hash: {}", hash);

    // Check if hash looks like valid bcrypt format
    if !hash.starts_with("$2b$") && !hash.starts_with("$2a$") && !hash.starts_with("$2y$") {
        eprintln!("❌ Hash doesn't start with bcrypt identifier ($2b$, $2a$, or $2y$)");
        return false;
    }

    match verify(password, hash) {
        Ok(result) => {
            if result {
                eprintln!("✅ Password verification successful");
            } else {
                eprintln!("❌ Password verification failed - passwords don't match");
            }
            result
        }
        Err(e) => {
            eprintln!("❌ Password verification error: {}", e);
            eprintln!("💡 Full hash that failed: '{}'", hash);
            eprintln!("💡 Hash length: {} characters", hash.len());
            false
        }
    }
}
