use sqlx::{MySql, MySqlPool, Pool};

pub async fn connect_db() -> Pool<MySql> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MySqlPool::connect(&url)
        .await
        .expect("Failed to connect to MySQL")
}
