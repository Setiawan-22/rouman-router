use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use sha2::{Digest, Sha256};

const ACTIVE_CONFIG_PATH: &str = "/opt/rouman/config/active.json";
const CANDIDATE_CONFIG_PATH: &str = "/opt/rouman/config/candidate.json";

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SystemConfig {
    pub hostname: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PortForwardRule {
    pub proto: String, // "tcp" or "udp"
    pub ext_port: u16,
    pub int_ip: String,
    pub int_port: u16,
    pub comment: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct QosConfig {
    pub download_mbps: u32,
    pub upload_mbps: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StaticLease {
    pub mac: String,
    pub ip: String,
    pub hostname: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DhcpConfig {
    pub enabled: bool,
    pub interface: String,
    pub range_start: String,
    pub range_end: String,
    pub gateway: String,
    pub subnet_mask: String,
    pub dns_servers: Vec<String>,
    pub lease_time_secs: u32,
    pub static_leases: Vec<StaticLease>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WireguardPeer {
    pub name: String,
    pub public_key: String,
    pub endpoint: Option<String>,
    pub allowed_ips: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WireguardInterface {
    pub name: String,
    pub private_key: String,
    pub listen_port: u16,
    pub address: String, // CIDR format, e.g., 10.0.0.1/24
    pub peers: Vec<WireguardPeer>,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WireguardConfig {
    pub interfaces: Vec<WireguardInterface>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PppoeConfig {
    pub enabled: bool,
    pub username: String,
    pub password: String,
    pub interface: String,
    pub mtu: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct VlanConfig {
    pub name: String,
    pub parent: String,
    pub vlan_id: u16,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BridgeConfig {
    pub name: String,
    pub members: Vec<String>,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct IpAssignment {
    pub interface: String,
    pub address: String, // CIDR, e.g., 192.168.10.1/24
    pub gateway: Option<String>,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct InterfaceConfig {
    pub vlans: Vec<VlanConfig>,
    pub bridges: Vec<BridgeConfig>,
    pub assignments: Vec<IpAssignment>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WanConfig {
    pub name: String,
    pub interface: String,
    pub gateway: String,
    pub weight: u32,       // Untuk PCC (cth: 1)
    pub distance: u32,     // Untuk Failover (cth: 1, 2)
    pub enabled: bool,
    pub pppoe_enabled: bool, // Link ke config PPPoE jika diperlukan
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub enum LoadBalancingMode {
    #[default]
    None,
    Failover,
    Pcc, // Per Connection Classifier
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SimpleQueue {
    pub name: String,
    pub target: String,      // IP or CIDR Subnet
    pub upload_mbps: u32,
    pub download_mbps: u32,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct NetworkConfig {
    pub wans: Vec<WanConfig>,
    pub lb_mode: LoadBalancingMode,
    pub enable_nat: bool,
    pub port_forwards: Vec<PortForwardRule>,
    pub qos: QosConfig,
    pub dhcp: DhcpConfig,
    pub pppoe: PppoeConfig,
    pub interfaces: InterfaceConfig,
    pub simple_queues: Vec<SimpleQueue>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CloudflareConfig {
    pub enabled: bool,
    pub tunnels: Vec<CloudflareTunnel>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudflareTunnel {
    pub id: String,
    pub name: String,
    pub token: String,
    pub enabled: bool,
    pub ingress: Vec<CloudflareIngress>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudflareIngress {
    pub hostname: String,
    pub service: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProxyConfig {
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FirewallConfig {
    #[serde(default)]
    pub enabled: bool,
    pub blacklist_ips: Vec<String>,
    pub enable_ebpf_ips: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DnsConfig {
    #[serde(default)]
    pub enabled: bool,
    pub sinkhole_enabled: bool,
    pub dot_enabled: bool,
    pub doh_enabled: bool,
    pub udp_enabled: bool,
    pub doh_url: String,
    pub adblock_enabled: bool,
    pub transparent_intercept: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SecurityConfig {
    pub encryption_enabled: bool,
    pub hardware_bound: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RadiusConfig {
    pub enabled: bool,
    pub secret: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RdpConfig {
    pub enabled: bool,
    pub interfaces: Vec<String>,
    #[serde(default = "default_rdp_psk")]
    pub psk: String,
}

fn default_rdp_psk() -> String {
    "rouman-discovery-v1-secret".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct NetwatchRule {
    pub name: String,
    pub target: String,
    pub interval_secs: u32,
    pub script_up: String,
    pub script_down: String,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SchedulerRule {
    pub name: String,
    pub interval_secs: u32,
    pub script: String,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CustomScript {
    pub name: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AutomationConfig {
    pub netwatch: Vec<NetwatchRule>,
    pub scheduler: Vec<SchedulerRule>,
    pub scripts: Vec<CustomScript>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ComputeConfig {
    pub microvm_enabled: bool,
    pub container_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RoumanConfig {
    pub system: SystemConfig,
    pub network: NetworkConfig,
    pub wireguard: WireguardConfig,
    pub cloudflare: CloudflareConfig,
    pub firewall: FirewallConfig,
    pub dns: DnsConfig,
    pub security: SecurityConfig,
    pub radius: RadiusConfig,
    pub proxy: ProxyConfig,
    pub rdp: RdpConfig,
    #[serde(default)]
    pub automation: AutomationConfig,
    #[serde(default)]
    pub compute: ComputeConfig,
}

pub struct ConfigEngine {
    pub active: Arc<RwLock<RoumanConfig>>,
    pub candidate: Arc<RwLock<RoumanConfig>>,
}

pub mod mutator;

impl ConfigEngine {
    pub async fn new() -> Arc<Self> {
        let active = Self::load_from_disk(ACTIVE_CONFIG_PATH).unwrap_or_else(|| {
            let default = RoumanConfig {
                system: SystemConfig {
                    hostname: "rouman-core".to_string(),
                    description: "Rouman OS Next-Gen Router".to_string(),
                },
                network: NetworkConfig {
                    wans: vec![
                        WanConfig {
                            name: "WAN1".to_string(),
                            interface: "eth0".to_string(),
                            gateway: "192.168.0.1".to_string(), // Asumsi default
                            weight: 1,
                            distance: 1,
                            enabled: true,
                            pppoe_enabled: false,
                        }
                    ],
                    lb_mode: LoadBalancingMode::None,
                    enable_nat: false,
                    port_forwards: Vec::new(),
                    qos: QosConfig::default(),
                    dhcp: DhcpConfig {
                        enabled: false,
                        interface: "eth1".to_string(),
                        range_start: "192.168.1.10".to_string(),
                        range_end: "192.168.1.250".to_string(),
                        gateway: "192.168.1.1".to_string(),
                        subnet_mask: "255.255.255.0".to_string(),
                        dns_servers: vec!["8.8.8.8".to_string()],
                        lease_time_secs: 86400,
                        static_leases: Vec::new(),
                    },
                    pppoe: PppoeConfig {
                        enabled: false,
                        username: "".to_string(),
                        password: "".to_string(),
                        interface: "eth0".to_string(),
                        mtu: 1492,
                    },
                    interfaces: InterfaceConfig {
                        vlans: Vec::new(),
                        bridges: Vec::new(),
                        assignments: vec![
                            IpAssignment {
                                interface: "eth1".to_string(),
                                address: "192.168.1.1/24".to_string(),
                                gateway: None,
                                enabled: true,
                            }
                        ],
                    },
                    simple_queues: Vec::new(),
                },
                wireguard: WireguardConfig::default(),
                cloudflare: CloudflareConfig::default(),
                firewall: FirewallConfig {
                    enabled: false,
                    blacklist_ips: vec![],
                    enable_ebpf_ips: false,
                },
                dns: DnsConfig {
                    enabled: false,
                    sinkhole_enabled: false,
                    dot_enabled: false,
                    doh_enabled: false,
                    udp_enabled: false,
                    doh_url: "https://cloudflare-dns.com/dns-query".to_string(),
                    adblock_enabled: true,
                    transparent_intercept: true,
                },
                security: SecurityConfig {
                    encryption_enabled: true,
                    hardware_bound: true,
                },
                radius: RadiusConfig {
                    enabled: false,
                    secret: "rouman-secret-123".to_string(),
                },
                proxy: ProxyConfig {
                    enabled: false,
                },
                rdp: RdpConfig {
                    enabled: false,
                    interfaces: vec!["eth1".to_string()], // Default LAN
                    psk: default_rdp_psk(),
                },
                automation: AutomationConfig::default(),
                compute: ComputeConfig {
                    microvm_enabled: false,
                    container_enabled: false,
                },
            };
            let _ = Self::save_to_disk(ACTIVE_CONFIG_PATH, &default);
            default
        });

        let candidate = Self::load_from_disk(CANDIDATE_CONFIG_PATH).unwrap_or_else(|| {
            active.clone()
        });

        let _ = Self::save_to_disk(CANDIDATE_CONFIG_PATH, &candidate);

        Arc::new(ConfigEngine {
            active: Arc::new(RwLock::new(active)),
            candidate: Arc::new(RwLock::new(candidate)),
        })
    }

    pub fn get_active_config_arc(&self) -> Arc<RwLock<RoumanConfig>> {
        self.active.clone()
    }

    fn get_encryption_key() -> Vec<u8> {
        let machine_id = fs::read_to_string("/etc/machine-id").unwrap_or_else(|_| "rouman-default-id".to_string());
        let mut hasher = Sha256::new();
        hasher.update(machine_id.trim().as_bytes());
        hasher.finalize().to_vec()
    }

    fn encrypt_data(data: &[u8]) -> Result<Vec<u8>, String> {
        let key = Self::get_encryption_key();
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
        let nonce_bytes = rand::random::<[u8; 12]>();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, data).map_err(|e| e.to_string())?;
        
        // Format: [MAGIC (14 bytes)] [NONCE (12 bytes)] [CIPHERTEXT]
        let mut result = b"ROUMAN_ENC_V1:".to_vec();
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    fn decrypt_data(data: &[u8]) -> Result<Vec<u8>, String> {
        let magic = b"ROUMAN_ENC_V1:";
        if !data.starts_with(magic) {
            return Ok(data.to_vec());
        }

        let key = Self::get_encryption_key();
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
        
        if data.len() < magic.len() + 12 {
            return Err("Invalid encrypted file format".to_string());
        }

        let nonce_bytes = &data[magic.len()..magic.len()+12];
        let ciphertext = &data[magic.len()+12..];
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher.decrypt(nonce, ciphertext).map_err(|e| e.to_string())
    }

    fn load_from_disk(path: &str) -> Option<RoumanConfig> {
        if Path::new(path).exists() {
            if let Ok(content) = fs::read(path) {
                if let Ok(decrypted) = Self::decrypt_data(&content) {
                    if let Ok(config) = serde_json::from_slice(&decrypted) {
                        return Some(config);
                    }
                }
            }
        }
        None
    }

    fn save_to_disk(path: &str, config: &RoumanConfig) -> Result<(), String> {
        let json_content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
        let final_data = if config.security.encryption_enabled {
            Self::encrypt_data(json_content.as_bytes())?
        } else {
            json_content.into_bytes()
        };
        
        // Atomic Write-Rename Pattern
        let temp_path = format!("{}.tmp", path);
        fs::write(&temp_path, final_data).map_err(|e| e.to_string())?;
        fs::rename(&temp_path, path).map_err(|e| e.to_string())?;
        
        Ok(())
    }

    pub async fn commit(&self) -> Result<(), String> {
        let candidate_guard = self.candidate.read().await;
        let new_config = candidate_guard.clone();
        
        // Memicu Mutasi pada Host OS Lapis Bawah
        mutator::apply_config(&new_config).await?;

        Self::save_to_disk(ACTIVE_CONFIG_PATH, &new_config)?;
        
        let mut active_guard = self.active.write().await;
        *active_guard = new_config;

        Ok(())
    }

    pub async fn rollback(&self) -> Result<(), String> {
        let active_guard = self.active.read().await;
        let original_config = active_guard.clone();

        Self::save_to_disk(CANDIDATE_CONFIG_PATH, &original_config)?;

        let mut candidate_guard = self.candidate.write().await;
        *candidate_guard = original_config;
        
        Ok(())
    }

    pub async fn update_candidate(&self, new_config: RoumanConfig) -> Result<(), String> {
        Self::save_to_disk(CANDIDATE_CONFIG_PATH, &new_config)?;
        let mut candidate_guard = self.candidate.write().await;
        *candidate_guard = new_config;
        Ok(())
    }
}

// ========================
// API HANDLERS
// ========================

pub async fn get_active(State(state): State<crate::AppState>) -> Json<serde_json::Value> {
    let engine = &state.config_engine;
    let config = engine.active.read().await;
    Json(serde_json::json!({ "active": &*config }))
}

pub async fn get_candidate(State(state): State<crate::AppState>) -> Json<serde_json::Value> {
    let engine = &state.config_engine;
    let config = engine.candidate.read().await;
    Json(serde_json::json!({ "candidate": &*config }))
}

pub async fn put_candidate(
    State(state): State<crate::AppState>,
    Json(payload): Json<RoumanConfig>,
) -> Json<serde_json::Value> {
    let engine = &state.config_engine;
    match engine.update_candidate(payload).await {
        Ok(_) => Json(serde_json::json!({ "status": "success", "message": "Candidate updated" })),
        Err(e) => Json(serde_json::json!({ "status": "error", "message": e })),
    }
}

pub async fn commit_config(State(state): State<crate::AppState>) -> Json<serde_json::Value> {
    let engine = &state.config_engine;
    match engine.commit().await {
        Ok(_) => Json(serde_json::json!({ "status": "success", "message": "Commit successful" })),
        Err(e) => Json(serde_json::json!({ "status": "error", "message": e })),
    }
}

pub async fn rollback_config(State(state): State<crate::AppState>) -> Json<serde_json::Value> {
    let engine = &state.config_engine;
    match engine.rollback().await {
        Ok(_) => Json(serde_json::json!({ "status": "success", "message": "Rollback successful" })),
        Err(e) => Json(serde_json::json!({ "status": "error", "message": e })),
    }
}
