use crate::logging;
use sqlx::{MySql, MySqlPool, Pool};
use std::env;

pub async fn connect_db() -> Result<Pool<MySql>, Box<dyn std::error::Error>> {
    let database_url =
        env::var("DATABASE_URL").map_err(|_| "DATABASE_URL environment variable must be set")?;

    logging::log_db_operation("connection_attempt", "mysql");

    match MySqlPool::connect(&database_url).await {
        Ok(pool) => {
            logging::log_db_operation("connection_established", "mysql");
            Ok(pool)
        }
        Err(e) => {
            logging::log_db_error("connection", &e.to_string());
            Err(format!("Failed to connect to database: {}", e).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect_db_missing_url() {
        unsafe {
            env::remove_var("DATABASE_URL");
        }
        let result = connect_db().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connect_db_invalid_url() {
        unsafe {
            env::set_var(
                "DATABASE_URL",
                "mysql://invalid:invalid@localhost:9999/invalid",
            );
        }
        let result = connect_db().await;
        assert!(result.is_err());
    }
}
