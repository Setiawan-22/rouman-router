use tokio::process::{Child, Command};
use std::process::Stdio;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use crate::database::Database;
use sqlx::Row;

pub struct CloudflareManager {
    db: Arc<Database>,
    processes: HashMap<String, Child>,
}

impl CloudflareManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            processes: HashMap::new(),
        }
    }

    pub async fn run(&mut self) {
        println!("Cloudflare Tunnel Supervisor started.");
        
        loop {
            if let Err(e) = self.reconcile().await {
                eprintln!("Cloudflare reconcile error: {}", e);
            }
            sleep(Duration::from_secs(10)).await;
        }
    }

    async fn reconcile(&mut self) -> Result<(), sqlx::Error> {
        let rows = sqlx::query("SELECT id, name, token, enabled FROM cloudflare_tunnels")
            .fetch_all(&self.db.pool)
            .await?;

        let mut current_ids = Vec::new();

        for row in rows {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let token: String = row.get("token");
            let enabled: bool = row.get("enabled");
            current_ids.push(id.clone());

            // 1. Cek jika harus jalan tapi belum ada prosesnya
            if enabled {
                let is_running = if let Some(child) = self.processes.get_mut(&id) {
                    // Cek jika proses masih hidup
                    match child.try_wait() {
                        Ok(None) => true, // Masih jalan
                        _ => {
                            self.processes.remove(&id);
                            false // Sudah mati
                        }
                    }
                } else {
                    false
                };

                if !is_running {
                    println!("Starting Cloudflare Tunnel: {}", name);
                    match self.spawn_tunnel(&id, &token).await {
                        Ok(child) => {
                            self.processes.insert(id.clone(), child);
                            let _ = self.update_status(&id, "online", None).await;
                        }
                        Err(e) => {
                            let _ = self.update_status(&id, "error", Some(&e)).await;
                        }
                    }
                }
            } else {
                // 2. Cek jika harus mati tapi masih ada prosesnya
                if let Some(mut child) = self.processes.remove(&id) {
                    println!("Stopping Cloudflare Tunnel: {}", name);
                    let _ = child.kill().await;
                    let _ = self.update_status(&id, "offline", None).await;
                }
            }
        }

        // 3. Bersihkan proses yang ID-nya sudah dihapus dari DB
        let mut to_remove = Vec::new();
        for id in self.processes.keys() {
            if !current_ids.contains(id) {
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            if let Some(mut child) = self.processes.remove(&id) {
                let _ = child.kill().await;
            }
        }

        Ok(())
    }

    async fn spawn_tunnel(&self, _id: &str, token: &str) -> Result<Child, String> {
        Command::new("cloudflared")
            .args(["tunnel", "--no-autoupdate", "run", "--token", token])
            .stdout(Stdio::null())
            .stderr(Stdio::piped()) // Kita bisa log stderr jika perlu di masa depan
            .spawn()
            .map_err(|e| format!("Failed to spawn cloudflared: {}", e))
    }

    async fn update_status(&self, id: &str, status: &str, error: Option<&str>) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE cloudflare_tunnels SET status = ?, last_error = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
        )
        .bind(status)
        .bind(error)
        .bind(id)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }
}
