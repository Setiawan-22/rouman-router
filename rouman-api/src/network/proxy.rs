use axum::{routing::{get, post, delete, put}, Router, Json, extract::{State, Path}};
use serde::{Deserialize, Serialize};
use crate::AppState;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct ProxyHost {
    pub id: Option<i64>,
    pub domain: String,
    pub upstream_url: String,
    pub ssl_enabled: bool,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct ProxySettings {
    pub acme_email: Option<String>,
    pub cf_api_token: Option<String>,
}

pub fn proxy_routes() -> Router<AppState> {
    Router::new()
        .route("/hosts", get(list_hosts))
        .route("/hosts", post(create_host))
        .route("/hosts/:id", delete(delete_host))
        .route("/settings", get(get_settings))
        .route("/settings", put(update_settings))
}

async fn list_hosts(State(state): State<AppState>) -> Json<Vec<ProxyHost>> {
    let rows = sqlx::query_as::<_, ProxyHost>(
        "SELECT id, domain, upstream_url, ssl_enabled FROM proxy_hosts"
    ).fetch_all(&state.db.pool).await.unwrap_or_default();
    
    Json(rows)
}

async fn create_host(State(state): State<AppState>, Json(payload): Json<ProxyHost>) -> Json<serde_json::Value> {
    let _ = sqlx::query(
        "INSERT INTO proxy_hosts (domain, upstream_url, ssl_enabled) VALUES (?, ?, ?)"
    ).bind(payload.domain)
     .bind(payload.upstream_url)
     .bind(payload.ssl_enabled)
     .execute(&state.db.pool).await;

    Json(serde_json::json!({ "status": "created" }))
}

async fn delete_host(State(state): State<AppState>, Path(id): Path<i64>) -> Json<serde_json::Value> {
    let _ = sqlx::query("DELETE FROM proxy_hosts WHERE id = ?")
        .bind(id)
        .execute(&state.db.pool).await;
    
    Json(serde_json::json!({ "status": "deleted" }))
}

async fn get_settings(State(state): State<AppState>) -> Json<ProxySettings> {
    let row = sqlx::query_as::<_, ProxySettings>(
        "SELECT acme_email, cf_api_token FROM proxy_settings WHERE id = 1"
    ).fetch_one(&state.db.pool).await.unwrap_or(ProxySettings { acme_email: None, cf_api_token: None });

    Json(row)
}

async fn update_settings(State(state): State<AppState>, Json(payload): Json<ProxySettings>) -> Json<serde_json::Value> {
    let _ = sqlx::query(
        "INSERT INTO proxy_settings (id, acme_email, cf_api_token) VALUES (1, ?, ?) 
         ON CONFLICT(id) DO UPDATE SET acme_email = EXCLUDED.acme_email, cf_api_token = EXCLUDED.cf_api_token"
    ).bind(payload.acme_email)
     .bind(payload.cf_api_token)
     .execute(&state.db.pool).await;

    Json(serde_json::json!({ "status": "updated" }))
}
