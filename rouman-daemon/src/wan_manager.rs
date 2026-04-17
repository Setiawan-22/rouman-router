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

    pub async fn run(&self, config: Arc<RwLock<RoumanConfig>>) {
        println!("WAN Manager started (Multi-WAN health monitoring)...");
        
        loop {
            // Kita ambil config saat ini
            let wans = {
                let cfg = config.read().await;
                cfg.network.wans.clone()
            };

            for wan in wans {
                if !wan.enabled { continue; }

                // Ping gateway untuk cek kesehatan
                let success = self.check_health(&wan.gateway, &wan.interface).await;
                
                if !success {
                    log::warn!("WAN Link {} ({}) is UNREACHABLE!", wan.name, wan.interface);
                    // TODO: Di masa depan, kita bisa memicu mutator::apply_config secara manual 
                    // dengan menonaktifkan WAN ini sementara di candidate config.
                }
            }

            sleep(Duration::from_secs(10)).await;
        }
    }

    async fn check_health(&self, gateway: &str, _interface: &str) -> bool {
        // Ping sederhana (1 paket, timeout 1 detik)
        let output = Command::new("ping")
            .args(["-c", "1", "-W", "1", gateway])
            .output();

        match output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }
}
