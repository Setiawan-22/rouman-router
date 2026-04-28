use aya::{
    programs::{Xdp, XdpFlags},
    Ebpf,
};
use aya_log::EbpfLogger;
use std::sync::Arc;
use tokio::signal;

mod dns_server;
mod dhcp_server;
mod neighbor_discovery;
mod pppoe_manager;
mod automation_manager;
mod wan_manager;
mod database;
mod radius_server;
mod hotspot_server;
mod proxy;
mod cloudflare_manager;
mod compute;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // Initialize rustls crypto provider (required for rustls 0.23+)
    let _ = rustls::crypto::ring::default_provider().install_default();

    println!("Starting Rouman Daemon (Layer 3 Control Plane)...");

    // Load eBPF Program
    // Selalu gunakan binary release untuk eBPF karena versi debug memiliki isu verifier (core::fmt)
    let mut bpf = Ebpf::load(aya::include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/rouman-ebpf"
    ))?;

    if let Err(e) = EbpfLogger::init(&mut bpf) {
        log::warn!("failed to initialize eBPF logger: {}", e);
    }

    let program: &mut Xdp = match bpf.program_mut("rouman_firewall") {
        Some(p) => p.try_into()?,
        None => {
            log::error!("eBPF program 'rouman_firewall' not found in ELF");
            return Err("eBPF program not found".into());
        }
    };
    program.load()?;
    
    // Deteksi antarmuka jaringan otomatis
    let interface = std::fs::read_dir("/sys/class/net")?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .find(|name| name != "lo" && {
            std::fs::read_to_string(format!("/sys/class/net/{}/operstate", name))
                .map(|s| s.trim() == "up")
                .unwrap_or(false)
        })
        .ok_or("Could not find any active network interface (non-loopback)")?;

    println!("Target interface detected: {}", interface);

    program.attach(&interface, XdpFlags::default())
        .map_err(|e| format!("failed to attach XDP program to {}: {}", interface, e))?;

    println!("eBPF Firewall attached to {}", interface);


    // Jalankan API Gateway dan Web UI Serve
    let bpf_arc = Arc::new(tokio::sync::Mutex::new(bpf));
    let bpf_arc_clone = bpf_arc.clone();
    
    let rdp_state = Arc::new(rouman_api::network::neighbors::RdpState {
        neighbors: tokio::sync::Mutex::new(std::collections::HashMap::new()),
    });
    let rdp_state_for_api = rdp_state.clone();

    let pppoe_manager = Arc::new(pppoe_manager::PppoeManager::new());
    let pppoe_state = Arc::new(rouman_api::network::pppoe::PppoeState {
        status: pppoe_manager.status.clone(),
    });
    let pppoe_state_for_api = pppoe_state.clone();

    let automation_manager = Arc::new(automation_manager::AutomationManager::new());
    let automation_manager_for_api = automation_manager.clone();

    let (dns_state, lease_pool, config_engine) = rouman_api::start_api_server(
        bpf_arc_clone, 
        rdp_state_for_api,
        pppoe_state_for_api,
    ).await;
    
    // Jalankan Automation Manager
    let auto_manager_clone = automation_manager.clone();
    let config_arc_auto = config_engine.get_active_config_arc();
    tokio::spawn(async move {
        auto_manager_clone.run(config_arc_auto).await;
    });

    // Jalankan Basis Data RADIUS
    let db = match database::Database::new("sqlite:/var/lib/rouman/radius.db").await {
        Ok(d) => Arc::new(d),
        Err(e) => {
            log::error!("Failed to initialize database: {}", e);
            return Err(e.into());
        }
    };

    // Load persisted DHCP leases into memory
    let db_for_load = db.clone();
    let lease_pool_for_load = lease_pool.clone();
    tokio::spawn(async move {
        match db_for_load.get_active_dhcp_leases().await {
            Ok(leases) => {
                let mut pool = lease_pool_for_load.leases.lock().await;
                for (mac_str, ip_str, hostname, expires_at) in leases {
                    // Parse MAC
                    let mut mac = [0u8; 6];
                    let parts: Vec<&str> = mac_str.split(':').collect();
                    if parts.len() == 6 {
                        for (i, p) in parts.iter().enumerate() {
                            if let Ok(b) = u8::from_str_radix(p, 16) {
                                mac[i] = b;
                            }
                        }
                    }
                    
                    // Parse IP
                    if let Ok(ip) = ip_str.parse::<std::net::Ipv4Addr>() {
                        let expires = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(expires_at as u64);
                        pool.insert(mac, rouman_api::network::dhcp::DhcpLease {
                            mac,
                            ip,
                            hostname,
                            expires,
                        });
                    }
                }
                log::info!("Loaded {} DHCP leases from database.", pool.len());
            },
            Err(e) => log::error!("Failed to load DHCP leases from DB: {}", e),
        }
    });

    // Jalankan WAN Manager (Supervised)
    let wan_manager = wan_manager::WanManager::new();
    let config_arc_wan = config_engine.get_active_config_arc();
    let db_for_wan = db.clone();
    tokio::spawn(async move {
        loop {
            log::info!("Starting WAN Manager supervisor...");
            wan_manager.run(config_arc_wan.clone(), db_for_wan.clone()).await;
            log::error!("WAN Manager exited unexpectedly. Restarting in 5s...");
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    let _ = db.add_notification(
        "Firewall Active", 
        &format!("eBPF Firewall successfully attached to {}.", interface),
        "success"
    ).await;

    let config_arc_active = config_engine.get_active_config_arc();
    
    // Extract all needed config flags, then DROP the read lock immediately
    // to avoid deadlocking the config when the API tries to write.
    let (
        radius_enabled, proxy_enabled, cloudflare_enabled,
        pppoe_enabled, rdp_enabled
    ) = {
        let config_val = config_arc_active.read().await;
        (
            config_val.radius.enabled,
            config_val.proxy.enabled,
            config_val.cloudflare.enabled,
            config_val.network.pppoe.enabled,
            config_val.rdp.enabled,
        )
    }; // read lock dropped here

    // Jalankan RADIUS Server (UDP 1812)
    if radius_enabled {
        let db_radius = db.clone();
        tokio::spawn(async move {
            radius_server::start_radius_server(db_radius).await;
        });
    }

    // Jalankan Hotspot Landing Page (Port 8080)
    let db_hotspot = db.clone();
    let bpf_hotspot = bpf_arc.clone();
    tokio::spawn(async move {
        hotspot_server::run_hotspot_server(db_hotspot, bpf_hotspot).await;
    });

    // Jalankan Advanced Reverse Proxy (Port 443)
    if proxy_enabled {
        let db_proxy = db.clone();
        tokio::spawn(async move {
            proxy::run_proxy_server(db_proxy).await;
        });
    }

    // Jalankan Cloudflare Tunnel Manager (Supervisor)
    if cloudflare_enabled {
        let db_cf = db.clone();
        tokio::spawn(async move {
            loop {
                log::info!("Starting Cloudflare Manager supervisor...");
                let mut cf_manager = cloudflare_manager::CloudflareManager::new(db_cf.clone());
                cf_manager.run().await;
                log::error!("Cloudflare Manager crashed. Restarting in 10s...");
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        });
    }

    // Jalankan Edge Compute Manager (Firecracker & Containerd Isolation Network)
    let bridge_subnet = config_engine.active.read().await.compute.bridge_subnet.clone();
    let _compute_manager = compute::ComputeManager::new(bridge_subnet);

    // Jalankan PPPoE Manager
    if pppoe_enabled {
        let pppoe_manager_clone = pppoe_manager.clone();
        let config_arc_pppoe = config_engine.get_active_config_arc();
        tokio::spawn(async move {
            pppoe_manager_clone.run(config_arc_pppoe).await;
        });
    }

    // Jalankan RDP Discovery
    if rdp_enabled {
        let rdp_state_clone = rdp_state.clone();
        let config_arc_rdp = config_engine.get_active_config_arc();
        tokio::spawn(async move {
            if let Err(e) = neighbor_discovery::run_rdp_service(rdp_state_clone, config_arc_rdp).await {
                log::error!("RDP Service error: {}", e);
            }
        });
    }

    // Jalankan DHCP Server (Supervised)
    let lease_pool_clone = lease_pool.clone();
    let config_arc_dhcp = config_engine.get_active_config_arc();
    let db_for_dhcp = db.clone();
    tokio::spawn(async move {
        loop {
            log::info!("Starting DHCP Server supervisor...");
            if let Err(e) = dhcp_server::run_dhcp_server(
                config_arc_dhcp.clone(), 
                lease_pool_clone.clone(),
                db_for_dhcp.clone(),
            ).await {
                log::error!("DHCP Server error: {}. Restarting in 5s...", e);
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    // Jalankan DNS Server
    let dns_state_clone = dns_state.clone();
    let bpf_for_sni = bpf_arc.clone();
    
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            let blocked = {
                let guard = dns_state_clone.blocked_domains.lock().await;
                guard.iter().cloned().collect::<Vec<String>>()
            };

            let mut bpf = bpf_for_sni.lock().await;
            if let Some(map) = bpf.map_mut("SNI_BLACKLIST") {
                if let Ok(mut sni_map) = aya::maps::HashMap::<_, [u8; 32], u32>::try_from(map) {
                    for domain in blocked {
                        let mut key = [0u8; 32];
                        let bytes = domain.as_bytes();
                        let len = bytes.len().min(32);
                        key[..len].copy_from_slice(&bytes[..len]);
                        let _ = sni_map.insert(key, 1, 0);
                    }
                }
            } else {
                // Log only once per minute to avoid spam if map is missing
                static mut LAST_WARN: u64 = 0;
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                unsafe {
                    if now - LAST_WARN > 60 {
                        log::warn!("SNI_BLACKLIST map not found - DNS filtering might be limited");
                        LAST_WARN = now;
                    }
                }
            }
        }
    });

    let dns_state_server = dns_state.clone();
    let config_arc_server = config_engine.get_active_config_arc();
    tokio::spawn(async move {
        loop {
            log::info!("Starting DNS Server supervisor...");
            if let Err(e) = dns_server::run_dns_server(dns_state_server.clone(), config_arc_server.clone()).await {
                log::error!("DNS Server error: {}. Restarting in 5s...", e);
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    println!("Press Ctrl+C to terminate and detach eBPF...");
    signal::ctrl_c().await?;
    println!("Detaching eBPF and exiting...");
    std::process::exit(0);
}
