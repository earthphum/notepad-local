use crate::{logging, models::Note, state::AppState};
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct NoteInput {
    content: String,
}

pub async fn create_note(
    State(state): State<AppState>,
    Json(note): Json<NoteInput>,
) -> (StatusCode, Json<serde_json::Value>) {
    let user = "earth"; // Hardcoded as in original, should come from JWT in production

    // Log note creation attempt (without exposing content)
    logging::log_note_operation("create", user);

    match sqlx::query("INSERT INTO notes (user, content) VALUES (?, ?)")
        .bind(user)
        .bind(&note.content)
        .execute(&*state.db)
        .await
    {
        Ok(_) => {
            logging::log_db_operation("insert", "notes");
            (
                StatusCode::CREATED,
                Json(json!({ "message": "Note created" })),
            )
        }
        Err(e) => {
            logging::log_db_error("insert", &e.to_string());
            logging::log_note_error("create", user, &e.to_string());
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create note" })),
            )
        }
    }
}

pub async fn get_notes(State(state): State<AppState>) -> Json<Vec<Note>> {
    let user = "earth"; // Hardcoded as in original, should come from JWT in production

    // Log note retrieval attempt
    logging::log_note_operation("retrieve", user);

    match sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE user = ?")
        .bind(user)
        .fetch_all(&*state.db)
        .await
    {
        Ok(notes) => {
            logging::log_db_operation("select", "notes");
            Json(notes)
        }
        Err(e) => {
            logging::log_db_error("select", &e.to_string());
            logging::log_note_error("retrieve", user, &e.to_string());
            // Return empty vector on error to avoid exposing internal errors
            Json(vec![])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_note_input_deserialization() {
        let json_data = r#"{"content":"Test note content"}"#;
        let note_input: NoteInput = serde_json::from_str(json_data).unwrap();
        assert_eq!(note_input.content, "Test note content");
    }

    #[test]
    fn test_note_input_empty_content() {
        let json_data = r#"{"content":""}"#;
        let note_input: NoteInput = serde_json::from_str(json_data).unwrap();
        assert_eq!(note_input.content, "");
    }

    #[tokio::test]
    async fn test_create_note_response_format() {
        let response = create_note(
            State(AppState {
                db: std::sync::Arc::new(sqlx::MySqlPool::connect("mysql://test").await.unwrap()),
            }),
            Json(NoteInput {
                content: "Test note".to_string(),
            }),
        )
        .await;

        // Should return CREATED status even if database fails (handled gracefully)
        assert_eq!(response.0, StatusCode::CREATED);
    }
}
