use std::process::Command;
use log::{info, warn, error};

pub mod firecracker;
pub mod containerd;

pub struct ComputeManager {
    bridge_name: String,
}

impl ComputeManager {
    pub fn new() -> Self {
        let manager = Self {
            bridge_name: "rouman-compute0".to_string(),
        };
        manager.setup_bridge();
        manager
    }

    fn setup_bridge(&self) {
        info!("Setting up Edge Compute isolated network: {}", self.bridge_name);

        // Create the isolated bridge if it doesn't exist
        let _ = Command::new("ip").args(&["link", "add", "name", &self.bridge_name, "type", "bridge"]).output();
        let _ = Command::new("ip").args(&["link", "set", &self.bridge_name, "up"]).output();

        // Assign IP to the router interface of the bridge
        let _ = Command::new("ip").args(&["addr", "add", "10.10.10.1/24", "dev", &self.bridge_name]).output();

        // Setup basic NAT/Masquerade for the isolated network
        let _ = Command::new("iptables").args(&["-t", "nat", "-A", "POSTROUTING", "-s", "10.10.10.0/24", "!", "-d", "10.10.10.0/24", "-j", "MASQUERADE"]).output();
        
        info!("Compute network isolated NAT ready (10.10.10.0/24)");
    }
}
