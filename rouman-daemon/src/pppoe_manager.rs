use std::fs;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use tokio::sync::RwLock;
use rouman_api::config::RoumanConfig;
use rouman_api::network::pppoe::{PppoeStatus, PppoeState};

pub struct PppoeManager {
    pub status: Arc<RwLock<PppoeStatus>>,
}

impl PppoeManager {
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(PppoeStatus {
                connected: false,
                ip_address: None,
                interface: "ppp0".to_string(),
            })),
        }
    }

    pub async fn run(&self, config_arc: Arc<RwLock<RoumanConfig>>) {
        loop {
            let (enabled, username, password, interface, mtu) = {
                let conf = config_arc.read().await;
                let c = &conf.network.pppoe;
                (c.enabled, c.username.clone(), c.password.clone(), c.interface.clone(), c.mtu)
            };

            if !enabled {
                self.stop_pppd().await;
                sleep(Duration::from_secs(10)).await;
                continue;
            }

            // Generate config
            if let Err(e) = self.generate_peers_file(&username, &password, &interface, mtu) {
                log::error!("Failed to generate PPPoE peers file: {}", e);
                sleep(Duration::from_secs(10)).await;
                continue;
            }

            // Start pppd if not running
            if !self.is_pppd_running().await {
                println!("Starting PPPoE connection for {} on {}...", username, interface);
                match Command::new("pppd").arg("call").arg("rouman").spawn() {
                    Ok(_) => log::info!("PPPoE process (pppd) spawned successfully."),
                    Err(e) => log::error!("Failed to spawn pppd: {}", e),
                }
            }

            // Monitor status
            self.update_status().await;
            sleep(Duration::from_secs(5)).await;
        }
    }

    fn generate_peers_file(&self, user: &str, pass: &str, iface: &str, mtu: u16) -> std::io::Result<()> {
        let content = format!(
            "plugin rp-pppoe.so {}\n\
            user \"{}\"\n\
            password \"{}\"\n\
            noipdefault\n\
            defaultroute\n\
            replacedefaultroute\n\
            hide-password\n\
            noauth\n\
            persist\n\
            maxfail 0\n\
            holdoff 5\n\
            lcp-echo-interval 20\n\
            lcp-echo-failure 3\n\
            mtu {}\n\
            mru {}\n",
            iface, user, pass, mtu, mtu
        );

        fs::create_dir_all("/etc/ppp/peers")?;
        fs::write("/etc/ppp/peers/rouman", content)?;
        
        // Update pap/chap secrets
        let secret_entry = format!("\"{}\" * \"{}\"\n", user, pass);
        let _ = fs::write("/etc/ppp/pap-secrets", &secret_entry);
        let _ = fs::write("/etc/ppp/chap-secrets", &secret_entry);
        
        Ok(())
    }

    async fn is_pppd_running(&self) -> bool {
        // Cek apakah ppp0 ada
        std::path::Path::new("/sys/class/net/ppp0").exists()
    }

    async fn stop_pppd(&self) {
        if self.is_pppd_running().await {
            println!("Stopping PPPoE connection...");
            match Command::new("poff").arg("rouman").output().await {
                Ok(out) if !out.status.success() => log::error!("poff failed: {}", String::from_utf8_lossy(&out.stderr)),
                Err(e) => log::error!("Failed to execute poff: {}", e),
                _ => log::info!("PPPoE connection stopped."),
            }
            
            let mut status = self.status.write().await;
            status.connected = false;
            status.ip_address = None;
        }
    }

    async fn update_status(&self) {
        let exists = std::path::Path::new("/sys/class/net/ppp0").exists();
        let mut ip = None;

        if exists {
            // Get IP via ip addr
            let output = Command::new("ip")
                .arg("-4")
                .arg("addr")
                .arg("show")
                .arg("ppp0")
                .output()
                .await;

            if let Ok(out) = output {
                let s = String::from_utf8_lossy(&out.stdout);
                if let Some(line) = s.lines().find(|l| l.contains("inet ")) {
                    let parts: Vec<&str> = line.trim().split_whitespace().collect();
                    if parts.len() > 1 {
                        ip = Some(parts[1].split('/').next().unwrap_or("").to_string());
                    }
                }
            }
        }

        let mut status = self.status.write().await;
        status.connected = exists && ip.is_some();
        status.ip_address = ip;
    }
}
