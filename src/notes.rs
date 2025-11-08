use crate::{models::Note, state::AppState};
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
    sqlx::query("INSERT INTO notes (user, content) VALUES (?, ?)")
        .bind("earth")
        .bind(&note.content)
        .execute(&*state.db)
        .await
        .unwrap();
    (
        StatusCode::CREATED,
        Json(json!({ "message": "Note created" })),
    )
}

pub async fn get_notes(State(state): State<AppState>) -> Json<Vec<Note>> {
    let notes = sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE user = ?")
        .bind("earth")
        .fetch_all(&*state.db)
        .await
        .unwrap();
    Json(notes)
}
