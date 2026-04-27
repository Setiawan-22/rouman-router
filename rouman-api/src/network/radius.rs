use axum::{routing::{get, post, delete}, Router, Json, extract::{State, Path}};
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::database::Database;
use rand::{distributions::Alphanumeric, Rng};
use std::sync::Arc;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct RadiusUser {
    pub id: Option<i64>,
    pub username: String,
    pub password: String,
    pub profile_id: Option<i64>,
    pub is_voucher: bool,
    pub status: String,
}

#[derive(Deserialize)]
pub struct CreateVoucherRequest {
    pub count: u32,
    pub profile_id: i64,
}

pub fn radius_routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/vouchers/generate", post(generate_vouchers))
        .route("/users/{username}", delete(delete_user))
        .route("/profiles", get(list_profiles))
}

async fn list_users(State(state): State<AppState>) -> Json<Vec<RadiusUser>> {
    let rows = sqlx::query_as::<_, RadiusUser>(
        "SELECT id, username, password, profile_id, is_voucher, status FROM radius_users"
    ).fetch_all(&state.db.pool).await.unwrap_or_default();
    
    Json(rows)
}

async fn create_user(State(state): State<AppState>, Json(payload): Json<RadiusUser>) -> Json<serde_json::Value> {
    let _ = sqlx::query(
        "INSERT INTO radius_users (username, password, profile_id, is_voucher) VALUES (?, ?, ?, ?)"
    ).bind(payload.username)
     .bind(payload.password)
     .bind(payload.profile_id)
     .bind(payload.is_voucher)
     .execute(&state.db.pool).await;

    Json(serde_json::json!({ "status": "created" }))
}

async fn generate_vouchers(State(state): State<AppState>, Json(payload): Json<CreateVoucherRequest>) -> Json<serde_json::Value> {
    for _ in 0..payload.count {
        let code: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();
        
        let _ = sqlx::query(
            "INSERT INTO radius_users (username, password, profile_id, is_voucher) VALUES (?, ?, ?, ?)"
        ).bind(&code)
         .bind(&code)
         .bind(payload.profile_id)
         .bind(true)
         .execute(&state.db.pool).await;
    }

    Json(serde_json::json!({ "status": "generated", "count": payload.count }))
}

async fn delete_user(State(state): State<AppState>, Path(username): Path<String>) -> Json<serde_json::Value> {
    let _ = sqlx::query("DELETE FROM radius_users WHERE username = ?")
        .bind(username)
        .execute(&state.db.pool).await;
    
    Json(serde_json::json!({ "status": "deleted" }))
}

async fn list_profiles(State(state): State<AppState>) -> Json<serde_json::Value> {
    let rows = sqlx::query("SELECT id, name, upload_mbps, download_mbps FROM radius_profiles")
        .fetch_all(&state.db.pool).await.unwrap_or_default();
    
    let result: Vec<_> = rows.into_iter().map(|r| {
        use sqlx::Row;
        serde_json::json!({
            "id": r.get::<i64, _>("id"),
            "name": r.get::<String, _>("name"),
            "upload": r.get::<i64, _>("upload_mbps"),
            "download": r.get::<i64, _>("download_mbps"),
        })
    }).collect();

    Json(serde_json::json!(result))
}
