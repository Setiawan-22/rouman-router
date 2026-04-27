use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use rouman_api::config::RoumanConfig;
use std::process::Command;

pub struct WanManager {
    // Bisa ditambahkan state untuk status kesehatan tiap WAN jika perlu
}

impl WanManager {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&self, config: Arc<RwLock<RoumanConfig>>, db: Arc<crate::database::Database>) {
        println!("WAN Manager started (Multi-WAN health monitoring)...");
        
        let mut last_primary_wan: Option<String> = None;

        loop {
            // Kita ambil config saat ini
            let (wans, lb_mode) = {
                let cfg = config.read().await;
                (cfg.network.wans.clone(), cfg.network.lb_mode.clone())
            };

            if wans.is_empty() {
                sleep(Duration::from_secs(10)).await;
                continue;
            }

            // Cari WAN terbaik yang sedang UP
            let mut best_wan = None;
            let mut min_distance = u32::MAX;

            for wan in &wans {
                if !wan.enabled { continue; }

                // Ping gateway untuk cek kesehatan
                let is_up = self.check_health(&wan.gateway, &wan.interface).await;
                
                if is_up {
                    if wan.distance < min_distance {
                        min_distance = wan.distance;
                        best_wan = Some(wan.clone());
                    }
                } else {
                    log::warn!("WAN Link {} ({}) is DOWN!", wan.name, wan.interface);
                    let _ = db.add_notification(
                        "WAN Link Down", 
                        &format!("WAN Link {} ({}) has lost connection.", wan.name, wan.interface),
                        "error"
                    ).await;
                }
            }

            // Jika ada perubahan WAN utama (Failover), update routing table
            if let Some(target_wan) = best_wan {
                if last_primary_wan.as_ref() != Some(&target_wan.name) {
                    println!("[FAILOVER] Switching primary WAN to {} via {}", target_wan.name, target_wan.interface);
                    
                    // Update default route di Host OS
                    // ip route replace default via [GW] dev [IFACE]
                    let status = Command::new("ip")
                        .args(["route", "replace", "default", "via", &target_wan.gateway, "dev", &target_wan.interface])
                        .status();
                    
                    if let Ok(s) = status {
                        if s.success() {
                            last_primary_wan = Some(target_wan.name.clone());
                            let _ = db.add_notification(
                                "WAN Failover", 
                                &format!("Switched primary WAN to {} via {}.", target_wan.name, target_wan.interface),
                                "info"
                            ).await;
                        }
                    }
                }
            } else {
                log::error!("[WAN] NO WAN LINKS ARE UP! Internet is dead.");
            }

            sleep(Duration::from_secs(10)).await;
        }
    }

    async fn check_health(&self, gateway: &str, _interface: &str) -> bool {
        let gateway = gateway.to_string();
        tokio::task::spawn_blocking(move || {
            let output = Command::new("ping")
                .args(["-c", "1", "-W", "1", &gateway])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();

            match output {
                Ok(s) => s.success(),
                Err(_) => false,
            }
        }).await.unwrap_or(false)
    }
}
