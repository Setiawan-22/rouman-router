use crate::config::CloudflareConfig;
use crate::AppState;
use axum::{extract::State, Json};
use serde_json::{json, Value};
use std::process::{Command, Stdio};

pub async fn sync_tunnels(config: &CloudflareConfig) -> Result<(), String> {
    // 1. Matikan semua proses cloudflared lama agar bersih (pendekatan sederhana)
    // Di masa depan kita bisa mematikan per tunnel ID menggunakan PID tracking
    let _ = Command::new("killall").arg("cloudflared").output();

    for tunnel in &config.tunnels {
        if tunnel.enabled {
            if tunnel.token.is_empty() {
                continue;
            }

            let child = Command::new("cloudflared")
                .args(["tunnel", "run", "--token", &tunnel.token])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("Gagal menjalankan tunnel {}: {}", tunnel.name, e))?;
            
            println!("Tunnel {} started with PID: {}", tunnel.name, child.id());
        }
    }

    Ok(())
}

// ========================
// API HANDLERS
// ========================

pub async fn get_config(State(state): State<AppState>) -> Json<Value> {
    let config = state.config_engine.active.read().await;
    Json(json!({ "cloudflare": &config.cloudflare }))
}

pub async fn check_binary(State(_): State<AppState>) -> Json<Value> {
    let output = Command::new("cloudflared").arg("--version").output();
    match output {
        Ok(out) => {
            let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Json(json!({ "installed": true, "version": version }))
        },
        Err(_) => {
            Json(json!({ "installed": false, "version": "" }))
        }
    }
}
