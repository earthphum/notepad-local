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
    eprintln!(
        "🔐 Verifying password with bcrypt hash: {}",
        &hash[..hash.len().min(30)]
    );

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
            eprintln!("💡 Hash format is invalid");
            false
        }
    }
}
