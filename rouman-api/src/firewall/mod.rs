use axum::{extract::State, Json};
use aya::maps::{HashMap, Array};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::net::Ipv4Addr;

pub struct FirewallState {
    pub ebpf: Arc<Mutex<aya::Ebpf>>,
}

pub async fn get_stats(
    State(state): State<crate::AppState>,
) -> Json<serde_json::Value> {
    let fw = &state.firewall_state;
    let bpf = fw.ebpf.lock().await;
    
    // Safely attempt to get the STATS map
    let stats_map = bpf.map("STATS");
    if stats_map.is_none() {
        return Json(serde_json::json!({
            "status": "inactive",
            "engine": "None",
            "total_packets": 0,
            "dropped_packets": 0,
            "warning": "eBPF map STATS not found (Dev Mode?)"
        }));
    }

    let stats: Array<_, u64> = match Array::try_from(stats_map.unwrap()) {
        Ok(it) => it,
        Err(_) => return Json(serde_json::json!({ "status": "error", "error": "Invalid map type" })),
    };
    
    // Indeks 0 = Total, 1 = Dropped
    let total_packets = stats.get(&0, 0).unwrap_or(0);
    let dropped_packets = stats.get(&1, 0).unwrap_or(0);

    Json(serde_json::json!({
        "status": "active",
        "engine": "eBPF/XDP",
        "total_packets": total_packets,
        "dropped_packets": dropped_packets
    }))
}

pub async fn add_blacklist(
    State(state): State<crate::AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let fw = &state.firewall_state;
    let ip_str = payload["ip"].as_str().unwrap_or("");
    let ip: Ipv4Addr = match ip_str.parse() {
        Ok(it) => it,
        Err(_) => return Json(serde_json::json!({ "error": "Invalid IP" })),
    };

    let mut bpf = fw.ebpf.lock().await;
    let map = bpf.map_mut("BLACKLIST");
    if map.is_none() {
        return Json(serde_json::json!({ "error": "Firewall engine inactive (BLACKLIST map missing)" }));
    }

    let mut blacklist: HashMap<_, u32, u32> = match HashMap::try_from(map.unwrap()) {
        Ok(it) => it,
        Err(_) => return Json(serde_json::json!({ "error": "Invalid map type" })),
    };
    
    let ip_u32 = u32::from(ip);
    if let Err(e) = blacklist.insert(ip_u32, 1, 0) {
        return Json(serde_json::json!({ "error": format!("Failed to insert: {}", e) }));
    }

    Json(serde_json::json!({ "status": "success", "message": format!("IP {} blacklisted", ip_str) }))
}
