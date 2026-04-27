use crate::config::CloudflareConfig;
use crate::AppState;
use axum::{extract::State, Json};
use serde_json::{json, Value};
use std::process::Command;
use crate::database::Database;

pub async fn sync_tunnels(config: &CloudflareConfig, db: &Database) -> Result<(), String> {
    // Sinkronisasi setiap tunnel di config ke database
    for tunnel in &config.tunnels {
        sqlx::query(
            "INSERT INTO cloudflare_tunnels (id, name, token, enabled) 
             VALUES (?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET 
             name = excluded.name,
             token = excluded.token,
             enabled = excluded.enabled"
        )
        .bind(&tunnel.id)
        .bind(&tunnel.name)
        .bind(&tunnel.token)
        .bind(tunnel.enabled)
        .execute(&db.pool)
        .await
        .map_err(|e| format!("DB Sync Error: {}", e))?;
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
