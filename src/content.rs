use crate::{
    logging,
    models::{CreateNoteRequest, Note, UpdateNoteRequest},
    state::AppState,
    utils::extract_user_from_token,
};
use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
    response::Json as ResponseJson,
};
use serde_json::json;
// use uuid::Uuid; // Unused import removed

// Public endpoints (no authentication required)

/// Get all public notes
pub async fn get_public_contents(
    State(state): State<AppState>,
) -> Result<ResponseJson<Vec<Note>>, StatusCode> {
    logging::log_api_request("GET", "/contents", 200);

    match sqlx::query_as::<_, Note>(
        "SELECT id, title, content, user, is_public, created_at, updated_at
         FROM notes
         WHERE is_public = true
         ORDER BY created_at DESC",
    )
    .fetch_all(&*state.db)
    .await
    {
        Ok(notes) => {
            logging::log_db_operation("select", "public_notes");
            Ok(ResponseJson(notes))
        }
        Err(e) => {
            logging::log_db_error("select", &e.to_string());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a specific public note by ID
pub async fn get_content_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<ResponseJson<Note>, StatusCode> {
    logging::log_api_request("GET", &format!("/contents/{}", id), 200);

    match sqlx::query_as::<_, Note>(
        "SELECT id, title, content, user, is_public, created_at, updated_at
         FROM notes
         WHERE id = ? AND is_public = true",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await
    {
        Ok(Some(note)) => {
            logging::log_db_operation("select", "public_note");
            Ok(ResponseJson(note))
        }
        Ok(None) => {
            logging::log_api_request("GET", &format!("/contents/{}", id), 404);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            logging::log_db_error("select", &e.to_string());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Admin/User endpoints (authentication required)

/// Get all notes (both public and private) for authenticated user
pub async fn get_all_contents(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ResponseJson<Vec<Note>>, StatusCode> {
    let username: String = match extract_user_from_token(&headers) {
        Ok(user) => user,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    logging::log_api_request("GET", "/admin/contents", 200);

    match sqlx::query_as::<_, Note>(
        "SELECT id, title, content, user, is_public, created_at, updated_at
         FROM notes
         WHERE user = ?
         ORDER BY created_at DESC",
    )
    .bind(&username)
    .fetch_all(&*state.db)
    .await
    {
        Ok(notes) => {
            logging::log_db_operation("select", "user_notes");
            logging::log_note_operation("retrieve", &username);
            Ok(ResponseJson(notes))
        }
        Err(e) => {
            logging::log_db_error("select", &e.to_string());
            logging::log_note_error("retrieve", &username, &e.to_string());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new note
pub async fn create_content(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateNoteRequest>,
) -> Result<(StatusCode, ResponseJson<serde_json::Value>), StatusCode> {
    let username: String = match extract_user_from_token(&headers) {
        Ok(user) => user,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    logging::log_note_operation("create", &username);

    match sqlx::query(
        "INSERT INTO notes (title, content, user, is_public, created_at, updated_at)
         VALUES (?, ?, ?, ?, NOW(), NOW())",
    )
    .bind(&request.title)
    .bind(&request.content)
    .bind(&username)
    .bind(request.is_public)
    .execute(&*state.db)
    .await
    {
        Ok(result) => {
            let note_id = result.last_insert_id();
            logging::log_db_operation("insert", "notes");

            let response = json!({
                "message": "Note created successfully",
                "id": note_id
            });

            Ok((StatusCode::CREATED, ResponseJson(response)))
        }
        Err(e) => {
            logging::log_db_error("insert", &e.to_string());
            logging::log_note_error("create", &username, &e.to_string());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a specific note by ID (user can access their own notes, any user can access public notes)
pub async fn get_content_by_id_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<ResponseJson<Note>, StatusCode> {
    let username = match extract_user_from_token(&headers) {
        Ok(user) => user,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    logging::log_api_request("GET", &format!("/admin/contents/{}", id), 200);

    match sqlx::query_as::<_, Note>(
        "SELECT id, title, content, user, is_public, created_at, updated_at
         FROM notes
         WHERE id = ? AND (user = ? OR is_public = true)",
    )
    .bind(id)
    .bind(&username)
    .fetch_optional(&*state.db)
    .await
    {
        Ok(Some(note)) => {
            logging::log_db_operation("select", "note");
            Ok(ResponseJson(note))
        }
        Ok(None) => {
            logging::log_api_request("GET", &format!("/admin/contents/{}", id), 404);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            logging::log_db_error("select", &e.to_string());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update a note
pub async fn update_content(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(request): Json<UpdateNoteRequest>,
) -> Result<(StatusCode, ResponseJson<serde_json::Value>), StatusCode> {
    let username: String = match extract_user_from_token(&headers) {
        Ok(user) => user,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Check if note exists and belongs to user
    let note_exists = sqlx::query("SELECT id FROM notes WHERE id = ? AND user = ?")
        .bind(id)
        .bind(&username)
        .fetch_optional(&*state.db)
        .await
        .map_err(|e| {
            logging::log_db_error("select", &e.to_string());
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if note_exists.is_none() {
        logging::log_api_request("PUT", &format!("/admin/contents/{}", id), 404);
        return Err(StatusCode::NOT_FOUND);
    }

    logging::log_note_operation("update", &username);

    // Check if any fields to update
    if request.title.is_none() && request.content.is_none() && request.is_public.is_none() {
        return Ok((
            StatusCode::BAD_REQUEST,
            ResponseJson(json!({"error": "No fields to update"})),
        ));
    }

    // Update title if provided
    if let Some(title) = request.title {
        sqlx::query("UPDATE notes SET title = ?, updated_at = NOW() WHERE id = ? AND user = ?")
            .bind(&title)
            .bind(id)
            .bind(&username)
            .execute(&*state.db)
            .await
            .map_err(|e| {
                logging::log_db_error("update", &e.to_string());
                logging::log_note_error("update", &username, &e.to_string());
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    // Update content if provided
    if let Some(content) = request.content {
        sqlx::query("UPDATE notes SET content = ?, updated_at = NOW() WHERE id = ? AND user = ?")
            .bind(&content)
            .bind(id)
            .bind(&username)
            .execute(&*state.db)
            .await
            .map_err(|e| {
                logging::log_db_error("update", &e.to_string());
                logging::log_note_error("update", &username, &e.to_string());
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    // Update is_public if provided
    if let Some(is_public) = request.is_public {
        sqlx::query("UPDATE notes SET is_public = ?, updated_at = NOW() WHERE id = ? AND user = ?")
            .bind(is_public)
            .bind(id)
            .bind(&username)
            .execute(&*state.db)
            .await
            .map_err(|e| {
                logging::log_db_error("update", &e.to_string());
                logging::log_note_error("update", &username, &e.to_string());
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    logging::log_db_operation("update", "notes");

    Ok((
        StatusCode::OK,
        ResponseJson(json!({"message": "Note updated successfully"})),
    ))
}

/// Delete a note
pub async fn delete_content(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<(StatusCode, ResponseJson<serde_json::Value>), StatusCode> {
    let username: String = match extract_user_from_token(&headers) {
        Ok(user) => user,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    logging::log_note_operation("delete", &username);

    match sqlx::query("DELETE FROM notes WHERE id = ? AND user = ?")
        .bind(id)
        .bind(&username)
        .execute(&*state.db)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                logging::log_db_operation("delete", "notes");
                Ok((
                    StatusCode::OK,
                    ResponseJson(json!({"message": "Note deleted successfully"})),
                ))
            } else {
                logging::log_api_request("DELETE", &format!("/admin/contents/{}", id), 404);
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            logging::log_db_error("delete", &e.to_string());
            logging::log_note_error("delete", &username, &e.to_string());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get statistics about notes
pub async fn get_stats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let username = match extract_user_from_token(&headers) {
        Ok(user) => user,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    logging::log_api_request("GET", "/admin/stats", 200);

    // Get total notes for user
    let total_notes: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM notes WHERE user = ?")
        .bind(&username)
        .fetch_one(&*state.db)
        .await
    {
        Ok(count) => count,
        Err(e) => {
            logging::log_db_error("select", &e.to_string());
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get public notes count for user
    let public_notes: i64 =
        match sqlx::query_scalar("SELECT COUNT(*) FROM notes WHERE user = ? AND is_public = true")
            .bind(&username)
            .fetch_one(&*state.db)
            .await
        {
            Ok(count) => count,
            Err(e) => {
                logging::log_db_error("select", &e.to_string());
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

    // Get private notes count for user
    let private_notes = total_notes - public_notes;

    let stats = json!({
        "total_notes": total_notes,
        "public_notes": public_notes,
        "private_notes": private_notes,
        "user": username
    });

    Ok(ResponseJson(stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_note_request_deserialization() {
        let json_data = r#"{
            "title": "Test Note",
            "content": "This is test content",
            "is_public": true
        }"#;
        let request: CreateNoteRequest = serde_json::from_str(json_data).unwrap();
        assert_eq!(request.title, "Test Note");
        assert_eq!(request.content, "This is test content");
        assert_eq!(request.is_public, true);
    }

    #[test]
    fn test_update_note_request_deserialization() {
        let json_data = r#"{
            "title": "Updated Title",
            "is_public": false
        }"#;
        let request: UpdateNoteRequest = serde_json::from_str(json_data).unwrap();
        assert_eq!(request.title, Some("Updated Title".to_string()));
        assert_eq!(request.content, None);
        assert_eq!(request.is_public, Some(false));
    }
}
