use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use axum::{extract::State, Json};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Neighbor {
    pub hostname: String,
    pub mac: String,
    pub version: String,
    pub last_seen: u64,
}

pub struct RdpState {
    pub neighbors: Mutex<HashMap<String, Neighbor>>,
}

pub async fn get_neighbors(
    State(rdp_state): State<Arc<RdpState>>,
) -> Json<serde_json::Value> {
    let neighbors = rdp_state.neighbors.lock().await;
    let list: Vec<Neighbor> = neighbors.values().cloned().collect();
    Json(serde_json::json!({ "neighbors": list }))
}
