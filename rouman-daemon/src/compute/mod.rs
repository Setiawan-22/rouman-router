use std::process::Command;
use log::{info, warn, error};

pub mod firecracker;
pub mod containerd;

pub struct ComputeManager {
    bridge_name: String,
    bridge_subnet: String,
}

impl ComputeManager {
    pub fn new(bridge_subnet: String) -> Self {
        let manager = Self {
            bridge_name: "rouman-compute0".to_string(),
            bridge_subnet,
        };
        manager.setup_bridge();
        manager
    }

    fn run_cmd(&self, cmd: &str, args: &[&str]) {
        match Command::new(cmd).args(args).output() {
            Ok(output) => {
                if !output.status.success() {
                    let err = String::from_utf8_lossy(&output.stderr);
                    error!("Compute network command failed ({} {:?}): {}", cmd, args, err.trim());
                }
            }
            Err(e) => error!("Failed to execute {}: {}", cmd, e),
        }
    }

    fn setup_bridge(&self) {
        info!("Setting up Edge Compute isolated network: {}", self.bridge_name);

        self.run_cmd("ip", &["link", "add", "name", &self.bridge_name, "type", "bridge"]);
        self.run_cmd("ip", &["link", "set", &self.bridge_name, "up"]);
        self.run_cmd("ip", &["addr", "add", &self.bridge_subnet, "dev", &self.bridge_name]);

        let network = self.bridge_subnet.replace(".1/24", ".0/24");
        self.run_cmd("iptables", &["-t", "nat", "-A", "POSTROUTING", "-s", &network, "!", "-d", &network, "-j", "MASQUERADE"]);
        
        info!("Compute network isolated NAT ready ({})", network);
    }
}
