use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use axum::{extract::State, Json};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Neighbor {
    pub hostname: String,
    pub mac: String,
    pub ip: String,
    pub version: String,
    pub last_seen: u64,
    pub neighbor_type: String, // "Physical", "MicroVM", "Container"
}

pub struct RdpState {
    pub neighbors: Mutex<HashMap<String, Neighbor>>,
}

pub async fn get_neighbors(
    State(rdp_state): State<Arc<RdpState>>,
) -> Json<serde_json::Value> {
    let mut neighbors_guard = rdp_state.neighbors.lock().await;
    let mut list: Vec<Neighbor> = neighbors_guard.values().cloned().collect();
    
    // Add Compute Instances from internal state
    // We import compute storage here to avoid circular dependencies if possible
    // Actually, we can just access COMPUTE_STORAGE directly if we use lazy_static
    
    let vms = crate::compute::get_vms_internal().await;
    for vm in vms {
        list.push(Neighbor {
            hostname: vm.id.clone(),
            mac: "VIRTUAL".to_string(),
            ip: "Dynamic".to_string(), // VMs often get IP via DHCP
            version: "Firecracker".to_string(),
            last_seen: 0,
            neighbor_type: "MicroVM".to_string(),
        });
    }

    let containers = crate::compute::get_containers_internal().await;
    for ct in containers {
        list.push(Neighbor {
            hostname: ct.id.clone(),
            mac: "VIRTUAL".to_string(),
            ip: ct.ip.clone(),
            version: ct.image.clone(),
            last_seen: 0,
            neighbor_type: "Container".to_string(),
        });
    }

    Json(serde_json::json!({ "neighbors": list }))
}
