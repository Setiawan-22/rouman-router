use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::time::SystemTime;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DhcpLease {
    pub mac: [u8; 6],
    pub ip: Ipv4Addr,
    pub hostname: Option<String>,
    pub expires: SystemTime,
}

pub struct LeasePool {
    pub leases: Mutex<HashMap<[u8; 6], DhcpLease>>,
}

impl Default for LeasePool {
    fn default() -> Self {
        Self {
            leases: Mutex::new(HashMap::new()),
        }
    }
}

pub type SharedLeasePool = Arc<LeasePool>;
