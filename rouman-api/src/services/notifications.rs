use axum::{routing::{get, post}, Router, Json, extract::{State, Path}};
use crate::{AppState, error::AppResult};
use sqlx::Row;
use serde_json::json;

pub fn notification_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_notifications))
        .route("/read/{id}", post(mark_as_read))
        .route("/clear", post(clear_all))
}

async fn list_notifications(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let rows = sqlx::query("SELECT id, title, message, type, is_read, created_at FROM notifications ORDER BY created_at DESC LIMIT 50")
        .fetch_all(&state.db.pool)
        .await?;

    let result: Vec<serde_json::Value> = rows.into_iter().map(|r| {
        json!({
            "id": r.get::<i64, _>("id"),
            "title": r.get::<String, _>("title"),
            "message": r.get::<String, _>("message"),
            "type": r.get::<String, _>("type"),
            "is_read": r.get::<bool, _>("is_read"),
            "created_at": r.get::<String, _>("created_at"),
        })
    }).collect();

    Ok(Json(json!({ "notifications": result })))
}

async fn mark_as_read(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Json<serde_json::Value>> {
    sqlx::query("UPDATE notifications SET is_read = 1 WHERE id = ?")
        .bind(id)
        .execute(&state.db.pool)
        .await?;
        
    Ok(Json(json!({ "status": "success" })))
}

async fn clear_all(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    sqlx::query("DELETE FROM notifications")
        .execute(&state.db.pool)
        .await?;
        
    Ok(Json(json!({ "status": "success" })))
}
