use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{extract::State, Json};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PppoeStatus {
    pub connected: bool,
    pub ip_address: Option<String>,
    pub interface: String,
}

pub struct PppoeState {
    pub status: Arc<RwLock<PppoeStatus>>,
}

pub async fn get_pppoe_status(
    State(pppoe_state): State<Arc<PppoeState>>,
) -> Json<serde_json::Value> {
    let status = pppoe_state.status.read().await;
    Json(serde_json::json!({ "status": &*status }))
}
