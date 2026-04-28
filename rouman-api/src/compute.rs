use axum::{
    routing::{get, post}, 
    Router, Json, 
    extract::{State, Query},
    response::Response,
    http::header
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct AgentInstallQuery {
    pub token: String,
    pub core_ip: String,
}

#[derive(Deserialize)]
pub struct AgentRegisterPayload {
    pub token: String,
    pub hostname: String,
    pub ip: String,
    pub wg_public_key: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VmInstance {
    pub id: String,
    pub vcpu: u32,
    pub ram: u32,
    pub image: String,
    pub status: String,
    pub uptime: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ContainerInstance {
    pub id: String,
    pub image: String,
    pub status: String,
    pub ip: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StackInstance {
    pub name: String,
    pub config: String,
    pub service_count: u32,
    pub status: String,
    pub uptime: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StorageMount {
    pub protocol: String,
    pub remote: String,
    pub local: String,
    pub options: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ComputeNode {
    pub id: String,
    pub hostname: String,
    pub ip: String,
    pub status: String, // "Online", "Offline"
    pub is_local: bool,
}

pub struct ComputeState {
    pub nodes: Mutex<Vec<ComputeNode>>,
    pub vms: Mutex<Vec<VmInstance>>,
    pub containers: Mutex<Vec<ContainerInstance>>,
    pub stacks: Mutex<Vec<StackInstance>>,
    pub storage: Mutex<Vec<StorageMount>>,
}

impl Default for ComputeState {
    fn default() -> Self {
        Self {
            nodes: Mutex::new(vec![
                ComputeNode {
                    id: "local".to_string(),
                    hostname: "Rouman-Primary".to_string(),
                    ip: "127.0.0.1".to_string(),
                    status: "Online".to_string(),
                    is_local: true,
                }
            ]),
            vms: Mutex::new(vec![]),
            containers: Mutex::new(vec![]),
            stacks: Mutex::new(vec![]),
            storage: Mutex::new(vec![]),
        }
    }
}

// Global in-memory state for compute (since we don't have a DB yet for this)
lazy_static::lazy_static! {
    static ref COMPUTE_STORAGE: Arc<ComputeState> = Arc::new(ComputeState::default());
}

async fn get_vms() -> Json<Value> {
    let vms = COMPUTE_STORAGE.vms.lock().await;
    Json(serde_json::json!({ "vms": *vms }))
}

async fn post_vm(Json(payload): Json<VmInstance>) -> Json<Value> {
    let mut vms = COMPUTE_STORAGE.vms.lock().await;
    let mut new_vm = payload;
    new_vm.status = "Running".to_string();
    new_vm.uptime = "0s".to_string();
    vms.push(new_vm);
    Json(serde_json::json!({ "status": "success" }))
}

async fn get_containers() -> Json<Value> {
    let containers = COMPUTE_STORAGE.containers.lock().await;
    Json(serde_json::json!({ "containers": *containers }))
}

async fn post_container(Json(payload): Json<ContainerInstance>) -> Json<Value> {
    let mut containers = COMPUTE_STORAGE.containers.lock().await;
    let mut new_ct = payload;
    new_ct.status = "Running".to_string();
    new_ct.ip = format!("10.88.0.{}", containers.len() + 2);
    containers.push(new_ct);
    Json(serde_json::json!({ "status": "success" }))
}

async fn get_stacks() -> Json<Value> {
    let stacks = COMPUTE_STORAGE.stacks.lock().await;
    Json(serde_json::json!({ "stacks": *stacks }))
}

async fn post_stack(Json(payload): Json<StackInstance>) -> Json<Value> {
    let mut stacks = COMPUTE_STORAGE.stacks.lock().await;
    let mut new_stack = payload;
    new_stack.status = "Active".to_string();
    new_stack.uptime = "0s".to_string();
    new_stack.service_count = 2; // Simulated count from YAML
    stacks.push(new_stack);
    Json(serde_json::json!({ "status": "success" }))
}

async fn get_storage() -> Json<Value> {
    let storage = COMPUTE_STORAGE.storage.lock().await;
    Json(serde_json::json!({ "mounts": *storage }))
}

async fn post_storage(Json(payload): Json<StorageMount>) -> Json<Value> {
    let mut storage = COMPUTE_STORAGE.storage.lock().await;
    storage.push(payload);
    Json(serde_json::json!({ "status": "success" }))
}

pub async fn get_vms_internal() -> Vec<VmInstance> {
    COMPUTE_STORAGE.vms.lock().await.clone()
}

pub async fn get_containers_internal() -> Vec<ContainerInstance> {
    COMPUTE_STORAGE.containers.lock().await.clone()
}

async fn get_nodes() -> Json<Value> {
    let nodes = COMPUTE_STORAGE.nodes.lock().await;
    Json(json!({ "nodes": *nodes }))
}

async fn post_node(Json(payload): Json<ComputeNode>) -> Json<Value> {
    let mut nodes = COMPUTE_STORAGE.nodes.lock().await;
    let mut new_node = payload;
    new_node.status = "Online".to_string(); // In real app, we would probe it
    new_node.is_local = false;
    nodes.push(new_node);
    Json(json!({ "status": "success" }))
}

// -----------------------------------------------------------------------------
// Auto-Provisioning Agent Endpoints
// -----------------------------------------------------------------------------

async fn agent_install_sh(Query(query): Query<AgentInstallQuery>) -> Response<String> {
    let script = format!(r#"#!/bin/bash
# Rouman Compute Agent - Auto Provisioning Script
# Generated for Core IP: {core_ip}

set -e

echo "=> Detecting Operating System..."
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=\$ID
else
    echo "Unsupported OS"
    exit 1
fi

echo "=> Installing WireGuard Tools..."
if [ "\$OS" = "fedora" ] || [ "\$OS" = "rhel" ] || [ "\$OS" = "centos" ]; then
    dnf install -y wireguard-tools jq
elif [ "\$OS" = "ubuntu" ] || [ "\$OS" = "debian" ]; then
    apt-get update && apt-get install -y wireguard-tools jq
elif [ "\$OS" = "alpine" ]; then
    apk add wireguard-tools jq
fi

echo "=> Generating WireGuard Keys..."
mkdir -p /etc/wireguard
wg genkey | tee /etc/wireguard/privatekey | wg pubkey > /etc/wireguard/publickey
WG_PRIV=\$(cat /etc/wireguard/privatekey)
WG_PUB=\$(cat /etc/wireguard/publickey)

echo "=> Downloading Rouman Agent..."
# Placeholder for actual binary download
# curl -sSL http://{core_ip}:8000/download/rouman-agent-\$OS -o /usr/local/bin/rouman-agent
# chmod +x /usr/local/bin/rouman-agent
echo "Simulating download for \$OS..."

echo "=> Registering with Rouman Core..."
PAYLOAD=\$(cat <<EOF
{{
  "token": "{token}",
  "hostname": "\$(hostname)",
  "ip": "\$(hostname -I | awk '{{print \$1}}')",
  "wg_public_key": "\$WG_PUB"
}}
EOF
)

RESPONSE=\$(curl -s -k -X POST https://{core_ip}:8443/api/v1/compute/agent/register \
     -H "Content-Type: application/json" \
     -d "\$PAYLOAD")

STATUS=\$(echo \$RESPONSE | jq -r .status)
if [ "\$STATUS" != "success" ]; then
    echo "Registration failed: \$(echo \$RESPONSE | jq -r .message)"
    exit 1
fi

CLUSTER_IP=\$(echo \$RESPONSE | jq -r .cluster_ip)
CORE_PUB=\$(echo \$RESPONSE | jq -r .core_public_key)

echo "=> Configuring WireGuard Overlay Network (wg-cluster)..."
cat <<EOF > /etc/wireguard/wg-cluster.conf
[Interface]
PrivateKey = \$WG_PRIV
Address = \$CLUSTER_IP/24

[Peer]
PublicKey = \$CORE_PUB
Endpoint = {core_ip}:51820
AllowedIPs = 10.42.0.0/16
PersistentKeepalive = 25
EOF

echo "=> Starting WireGuard Tunnel..."
systemctl enable wg-quick@wg-cluster || true
systemctl start wg-quick@wg-cluster || true

echo "=> Creating Systemd Service..."
cat <<EOF > /etc/systemd/system/rouman-agent.service
[Unit]
Description=Rouman Compute Agent
After=network.target wg-quick@wg-cluster.service

[Service]
ExecStart=/usr/local/bin/rouman-agent --core-ip {core_ip} --token {token}
Restart=always
User=root

[Install]
WantedBy=multi-user.target
EOF

echo "=> Enabling and starting Rouman Agent..."
systemctl daemon-reload
systemctl enable rouman-agent.service
# systemctl start rouman-agent.service # Uncomment when binary is real

echo ""
echo "=> Installation Complete! This node is now part of the Rouman Overlay Network (10.42.x.x)."
"#, core_ip = query.core_ip, token = query.token);

    Response::builder()
        .header(header::CONTENT_TYPE, "text/x-shellscript")
        .body(script)
        .unwrap_or_default()
}

async fn agent_register(
    State(state): State<crate::AppState>,
    Json(payload): Json<AgentRegisterPayload>
) -> Json<Value> {
    // Read the cluster token and subnet from config
    let config_guard = state.config_engine.active.read().await;
    let cluster_token = config_guard.system.cluster_token.clone();
    let cluster_subnet = config_guard.compute.cluster_subnet.clone();
    drop(config_guard); // Free the lock quickly

    // Strictly validate the token against the active configuration
    if payload.token.is_empty() || payload.token != cluster_token {
        return Json(json!({ "status": "error", "message": "Unauthorized: Invalid cluster token" }));
    }

    let mut nodes = COMPUTE_STORAGE.nodes.lock().await;
    
    // WireGuard Overlay Network Orchestration
    let mut config = state.config_engine.active.read().await.clone();
    let cluster_prefix = cluster_subnet.split('/').next().unwrap_or("10.42.0.0").trim_end_matches(".0");
    let core_address = format!("{}.1/24", cluster_prefix);
    
    let wg_interface_exists = config.wireguard.interfaces.iter().any(|i| i.name == "wg-cluster");
    if !wg_interface_exists {
        let new_iface = crate::config::WireguardInterface {
            name: "wg-cluster".to_string(),
            private_key: "yE8ZJ+H1lGj8V+D0XhV3Xw3y8BvNlV2+hX5R0tZ6a0I=".to_string(), // Dummy real-looking key
            listen_port: 51820,
            address: core_address,
            peers: vec![],
            enabled: true,
        };
        config.wireguard.interfaces.push(new_iface);
    }
    
    let wg_iface = match config.wireguard.interfaces.iter_mut().find(|i| i.name == "wg-cluster") {
        Some(iface) => iface,
        None => return Json(json!({ "status": "error", "message": "Failed to configure WG interface" })),
    };
    
    let core_pub_key = "+M5QoJ8eG/sH9+Qy/M3qQZ6+XG6B/hX5R0tZ6a0I=".to_string(); // Dummy pub key

    let next_ip_num = wg_iface.peers.len() + 2; // .1 is core
    let cluster_ip = format!("{}.{}", cluster_prefix, next_ip_num);

    let peer_exists = wg_iface.peers.iter().any(|p| p.public_key == payload.wg_public_key);
    if !peer_exists {
        wg_iface.peers.push(crate::config::WireguardPeer {
            name: payload.hostname.clone(),
            public_key: payload.wg_public_key.clone(),
            endpoint: Some(format!("{}:51820", payload.ip)), 
            allowed_ips: vec![format!("{}/32", cluster_ip)],
        });
    }

    // Save config via the public API to trigger mutator and kernel updates
    let _ = state.config_engine.update_candidate(config).await;
    let _ = state.config_engine.commit().await;

    // Compute Node Tracking
    if let Some(node) = nodes.iter_mut().find(|n| n.ip == payload.ip) {
        node.status = "Online".to_string();
        node.hostname = payload.hostname.clone();
        return Json(json!({ 
            "status": "success", 
            "message": "Node updated",
            "cluster_ip": cluster_ip,
            "core_public_key": core_pub_key
        }));
    }

    // Add new node
    use rand::Rng;
    let rng = rand::thread_rng();
    let random_str: String = rng
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    nodes.push(ComputeNode {
        id: format!("node-{}", random_str),
        hostname: payload.hostname,
        ip: payload.ip,
        status: "Online".to_string(),
        is_local: false,
    });

    Json(json!({ 
        "status": "success", 
        "message": "Node registered",
        "cluster_ip": cluster_ip,
        "core_public_key": core_pub_key
    }))
}

pub fn compute_routes() -> Router<crate::AppState> {
    Router::new()
        .route("/nodes", get(get_nodes).post(post_node))
        .route("/agent/install.sh", get(agent_install_sh))
        .route("/agent/register", post(agent_register))
        .route("/vms", get(get_vms).post(post_vm))
        .route("/containers", get(get_containers).post(post_container))
        .route("/stacks", get(get_stacks).post(post_stack))
        .route("/storage", get(get_storage).post(post_storage))
}
