use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub user: String,
    pub content: String,
}
