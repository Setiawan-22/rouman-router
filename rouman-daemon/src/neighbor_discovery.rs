use pnet::datalink::{self, Channel::Ethernet, NetworkInterface};
use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket, EtherType};
use pnet::packet::Packet;
use pnet::util::MacAddr;
use serde::{Deserialize, Serialize};
use hmac::{Hmac, Mac, KeyInit};
use sha2::Sha256;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::time::{Duration};
use rouman_api::config::RoumanConfig;
use rouman_api::network::neighbors::{Neighbor, RdpState};

type HmacSha256 = Hmac<Sha256>;

pub async fn run_rdp_service(
    state: Arc<RdpState>,
    config_arc: Arc<tokio::sync::RwLock<RoumanConfig>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting RDP (Rouman Discovery Protocol) Discovery Task...");

    // Ambil Secret dari machine-id untuk HMAC
    let machine_id = std::fs::read_to_string("/etc/machine-id").unwrap_or_else(|_| "default-rdp-secret".to_string());
    let hmac_key_vec = machine_id.trim().as_bytes().to_vec();

    let state_receiver = state.clone();
    let hmac_key_receiver = hmac_key_vec.clone();
    
    // 1. RECEIVER LOOP
    tokio::spawn(async move {
        let interfaces = datalink::interfaces();
        for iface in interfaces {
            // Kita dengerin di semua interface yang UP dan punya MAC (kecuali loopback)
            if iface.is_loopback() || iface.mac.is_none() || !iface.is_up() {
                continue;
            }

            let state = state_receiver.clone();
            let hmac_key = hmac_key_receiver.clone();
            
            tokio::spawn(async move {
                let (_, mut rx) = match datalink::channel(&iface, Default::default()) {
                    Ok(Ethernet(tx, rx)) => (tx, rx),
                    _ => return,
                };

                println!("[RDP] Receiver started on {}", iface.name);
                
                loop {
                    match rx.next() {
                        Ok(packet) => {
                            let eth = EthernetPacket::new(packet).unwrap();
                            // EtherType Custom (Local Use)
                            if eth.get_ethertype() == EtherType::new(0x88B5) {
                                let payload = eth.payload();
                                if payload.len() < 32 + 14 { continue; } // HMAC(32) + MAGIC(14)
                                
                                let magic = &payload[..14];
                                if magic != b"ROUMAN_RDP_V1" { continue; }

                                let signature = &payload[14..46];
                                let json_data = &payload[46..];

                                // Verifikasi HMAC
                                if let Ok(mut mac) = HmacSha256::new_from_slice(&hmac_key) {
                                    mac.update(json_data);
                                    if mac.verify_slice(signature).is_ok() {
                                        if let Ok(neighbor_data) = serde_json::from_slice::<Neighbor>(json_data) {
                                            let mut neighbors = state.neighbors.lock().await;
                                            let mut updated_neighbor = neighbor_data.clone();
                                            updated_neighbor.last_seen = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                                            neighbors.insert(neighbor_data.mac.clone(), updated_neighbor);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("RDP Recv Error on {}: {}", iface.name, e);
                            break;
                        }
                    }
                }
            });
        }
    });

    // 2. SENDER LOOP (Broadcast)
    let hmac_key_sender = hmac_key_vec.clone();
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        let (enabled, target_ifaces, hostname) = {
            let conf = config_arc.read().await;
            (conf.rdp.enabled, conf.rdp.interfaces.clone(), conf.system.hostname.clone())
        };

        if !enabled { continue; }

        {
            let mut neighbors = state.neighbors.lock().await;
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            neighbors.retain(|_, v| now - v.last_seen < 30);
        }

        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let interfaces = datalink::interfaces();
        for iface in interfaces {
            if !target_ifaces.contains(&iface.name) { continue; }
            
            let (mut tx, _) = match datalink::channel(&iface, Default::default()) {
                Ok(Ethernet(tx, rx)) => (tx, rx),
                _ => continue,
            };

            let neighbor_info = Neighbor {
                hostname: hostname.clone(),
                mac: iface.mac.unwrap().to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                last_seen: now,
            };

            if let Ok::<Vec<u8>, _>(json_vec) = serde_json::to_vec(&neighbor_info) {
                if let Ok(mut mac) = HmacSha256::new_from_slice(&hmac_key_sender) {
                    mac.update(&json_vec);
                    let signature = mac.finalize().into_bytes();

                    let mut packet_buf = [0u8; 1500]; // Standard MTU
                    let mut eth_packet = MutableEthernetPacket::new(&mut packet_buf).unwrap();
                    
                    eth_packet.set_destination(MacAddr::broadcast());
                    eth_packet.set_source(iface.mac.unwrap());
                    eth_packet.set_ethertype(EtherType::new(0x88B5));

                    let mut final_payload = b"ROUMAN_RDP_V1".to_vec();
                    final_payload.extend_from_slice(&signature);
                    final_payload.extend_from_slice(&json_vec);
                    
                    eth_packet.set_payload(&final_payload);
                    
                    let total_len = 14 + final_payload.len();
                    let _ = tx.send_to(&packet_buf[..total_len], None);
                }
            }
        }
    }
}
