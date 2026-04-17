use crate::config::WireguardConfig;
use crate::AppState;
use axum::{extract::State, Json};
use serde_json::{json, Value};
use x25519_dalek::{StaticSecret, PublicKey};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::process::Command;
use wireguard_control::{Backend, Device, DeviceUpdate, InterfaceName, Key, PeerConfigBuilder};
use std::net::SocketAddr;
use ipnetwork::IpNetwork;
use std::time::SystemTime;

// ========================
// CORE LOGIC
// ========================

pub fn generate_keypair_inner() -> (String, String) {
    let secret = StaticSecret::from(rand::random::<[u8; 32]>());
    let public = PublicKey::from(&secret);
    
    (
        STANDARD.encode(secret.to_bytes()),
        STANDARD.encode(public.as_bytes())
    )
}

pub async fn apply_wireguard_settings(config: &WireguardConfig) -> Result<(), String> {
    for iface in &config.interfaces {
        let interface_name: InterfaceName = iface.name.parse().map_err(|e| format!("Invalid interface name {}: {}", iface.name, e))?;

        if !iface.enabled {
            let _ = Command::new("ip").args(["link", "delete", &iface.name]).output();
            continue;
        }

        // 1. Create interface if missing
        let check = Command::new("ip").args(["link", "show", &iface.name]).output();
        if let Ok(out) = check {
            if !out.status.success() {
                let _ = Command::new("ip").args(["link", "add", "dev", &iface.name, "type", "wireguard"]).output();
            }
        }

        // 2. Set IP Address
        let _ = Command::new("ip").args(["addr", "flush", "dev", &iface.name]).output();
        let _ = Command::new("ip").args(["addr", "add", &iface.address, "dev", &iface.name]).output();

        // 3. Configure WG via wireguard-control
        let priv_key = Key::from_base64(&iface.private_key).map_err(|e| format!("Invalid Private Key: {}", e))?;

        let mut update = DeviceUpdate::new()
            .set_private_key(priv_key)
            .set_listen_port(iface.listen_port)
            .replace_peers();

        for p in &iface.peers {
            let pub_key = Key::from_base64(&p.public_key).map_err(|e| format!("Invalid Peer Public Key: {}", e))?;
            
            let mut peer_builder = PeerConfigBuilder::new(&pub_key);
            
            for ip_str in &p.allowed_ips {
                if let Ok(net) = ip_str.parse::<IpNetwork>() {
                    peer_builder = peer_builder.add_allowed_ip(net.ip(), net.prefix());
                }
            }

            if let Some(ref endpoint_str) = p.endpoint {
                if let Ok(addr) = endpoint_str.parse::<SocketAddr>() {
                    peer_builder = peer_builder.set_endpoint(addr);
                }
            }
            
            update = update.add_peer(peer_builder);
        }

        update.apply(&interface_name, Backend::Kernel)
            .map_err(|e| format!("Failed to apply WG config to {}: {}", iface.name, e))?;

        // 4. Bring UP
        let _ = Command::new("ip").args(["link", "set", "up", "dev", &iface.name]).output();
    }
    
    Ok(())
}

// ========================
// API HANDLERS
// ========================

pub async fn generate_keys(State(_): State<AppState>) -> Json<Value> {
    let (priv_key, pub_key) = generate_keypair_inner();
    Json(json!({
        "private_key": priv_key,
        "public_key": pub_key
    }))
}

pub async fn get_wireguard_config(State(state): State<AppState>) -> Json<Value> {
    let config = state.config_engine.active.read().await;
    Json(json!({ "wireguard": &config.wireguard }))
}

pub async fn get_wg_status(State(_): State<AppState>) -> Json<Value> {
    let device_names = match get_wg_interface_names() {
        Ok(names) => names,
        Err(_) => vec!["wg0".to_string()],
    };

    let mut results = Vec::new();
    for name_str in device_names {
        if let Ok(name) = name_str.parse::<InterfaceName>() {
            if let Ok(device) = Device::get(&name, Backend::Kernel) {
                let mut peers = Vec::new();
                for peer in device.peers {
                    peers.push(json!({
                        "public_key": STANDARD.encode(peer.config.public_key.as_bytes()),
                        "endpoint": peer.config.endpoint.map(|e: std::net::SocketAddr| e.to_string()),
                        "allowed_ips": peer.config.allowed_ips.iter().map(|i: &wireguard_control::AllowedIp| format!("{}/{}", i.address, i.cidr)).collect::<Vec<_>>(),
                        "last_handshake": peer.stats.last_handshake_time.and_then(|t: SystemTime| t.duration_since(SystemTime::UNIX_EPOCH).ok()).map(|d: std::time::Duration| d.as_secs()),
                        "rx_bytes": peer.stats.rx_bytes,
                        "tx_bytes": peer.stats.tx_bytes,
                    }));
                }
                results.push(json!({
                    "interface": name_str,
                    "listen_port": device.listen_port,
                    "public_key": device.public_key.map(|k: Key| STANDARD.encode(k.as_bytes())),
                    "peers": peers
                }));
            }
        }
    }

    Json(json!({ "status": results }))
}

fn get_wg_interface_names() -> Result<Vec<String>, std::io::Error> {
    let output = Command::new("ip").args(["-j", "link", "show", "type", "wireguard"]).output()?;
    if output.status.success() {
        let val: Value = serde_json::from_slice(&output.stdout)?;
        if let Some(list) = val.as_array() {
            return Ok(list.iter().filter_map(|i| i["ifname"].as_str().map(|s| s.to_string())).collect());
        }
    }
    Ok(Vec::new())
}

// Tambahan: Handler untuk menghasilkan string konfigurasi klien
pub async fn get_client_config(
    State(state): State<AppState>,
    axum::extract::Path((iface_name, peer_pubkey)): axum::extract::Path<(String, String)>,
) -> Json<Value> {
    let config = state.config_engine.active.read().await;
    
    // Cari interface dan peer terkait
    let iface = config.wireguard.interfaces.iter().find(|i| i.name == iface_name);
    let server_pubkey = if let Some(i) = iface {
        let priv_key = Key::from_base64(&i.private_key).ok();
        priv_key.map(|k| STANDARD.encode(PublicKey::from(&StaticSecret::from(<[u8; 32]>::try_from(k.as_bytes()).unwrap())).as_bytes()))
    } else { None };

    if let (Some(i), Some(s_pub)) = (iface, server_pubkey) {
        let peer = i.peers.iter().find(|p| p.public_key == peer_pubkey);
        if let Some(p) = peer {
            // Generate dummy client private key (atau ambil jika ada di sistem)
            // Note: Idealnya klien punya kunci sendiri, tapi kita sediakan template.
            let client_config = format!(
                "[Interface]\nPrivateKey = <CLIENT_PRIVATE_KEY>\nAddress = {}\nDNS = 1.1.1.1\n\n[Peer]\nPublicKey = {}\nAllowedIPs = 0.0.0.0/0\nEndpoint = <ROUTER_PUBLIC_IP>:{}",
                p.allowed_ips.first().cloned().unwrap_or_else(|| "10.0.0.2/32".to_string()),
                s_pub,
                i.listen_port
            );
            return Json(json!({ "config": client_config }));
        }
    }

    Json(json!({ "error": "Peer or Interface not found" }))
}
