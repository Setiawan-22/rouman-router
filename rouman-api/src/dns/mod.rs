use axum::extract::{State, FromRef};
use axum::{routing::{get, post, delete}, Router, Json};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashSet;
use serde::Deserialize;

#[derive(Default)]
pub struct DnsState {
    pub blocked_domains: Mutex<HashSet<String>>,
}

#[derive(Deserialize)]
pub struct DomainPayload {
    pub domain: String,
}

pub fn dns_routes() -> Router<crate::AppState> {
    Router::<crate::AppState>::new()
        .route("/blacklist", get(get_blacklist))
        .route("/blacklist", post(add_to_blacklist))
        .route("/blacklist", delete(remove_from_blacklist))
}

async fn get_blacklist(State(state): State<crate::AppState>) -> Json<Vec<String>> {
    let dns = &state.dns_state;
    let domains = dns.blocked_domains.lock().await;
    Json(domains.iter().cloned().collect())
}

async fn add_to_blacklist(
    State(state): State<crate::AppState>,
    Json(payload): Json<DomainPayload>,
) -> Json<serde_json::Value> {
    let dns = &state.dns_state;
    let mut domains = dns.blocked_domains.lock().await;
    domains.insert(payload.domain.clone());
    Json(serde_json::json!({ "status": "success", "message": format!("Domain {} blocked", payload.domain) }))
}

async fn remove_from_blacklist(
    State(state): State<crate::AppState>,
    Json(payload): Json<DomainPayload>,
) -> Json<serde_json::Value> {
    let dns = &state.dns_state;
    let mut domains = dns.blocked_domains.lock().await;
    if domains.remove(&payload.domain) {
        Json(serde_json::json!({ "status": "success", "message": format!("Domain {} unblocked", payload.domain) }))
    } else {
        Json(serde_json::json!({ "error": "Domain not found" }))
    }
}
