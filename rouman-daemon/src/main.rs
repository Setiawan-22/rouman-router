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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("Starting Rouman Daemon (Layer 3 Control Plane)...");

    // Load eBPF Program
    #[cfg(debug_assertions)]
    let mut bpf = Ebpf::load(aya::include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/rouman-ebpf"
    ))?;
    
    #[cfg(not(debug_assertions))]
    let mut bpf = Ebpf::load(aya::include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/rouman-ebpf"
    ))?;

    if let Err(e) = EbpfLogger::init(&mut bpf) {
        log::warn!("failed to initialize eBPF logger: {}", e);
    }

    let program: &mut Xdp = bpf.program_mut("rouman_firewall").unwrap().try_into()?;
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

    // Jalankan WAN Manager
    let wan_manager = wan_manager::WanManager::new();
    let config_arc_wan = config_engine.get_active_config_arc();
    tokio::spawn(async move {
        wan_manager.run(config_arc_wan).await;
    });

    // Jalankan Basis Data RADIUS
    let db = Arc::new(database::Database::new("sqlite://radius.db").await.unwrap());

    // Jalankan RADIUS Server (UDP 1812)
    let db_radius = db.clone();
    tokio::spawn(async move {
        radius_server::start_radius_server(db_radius).await;
    });

    // Jalankan Hotspot Landing Page (Port 8080)
    let db_hotspot = db.clone();
    tokio::spawn(async move {
        hotspot_server::run_hotspot_server(db_hotspot).await;
    });

    // Jalankan Advanced Reverse Proxy (Port 443)
    let db_proxy = db.clone();
    tokio::spawn(async move {
        proxy::run_proxy_server(db_proxy).await;
    });

    // Jalankan PPPoE Manager
    let pppoe_manager_clone = pppoe_manager.clone();
    let config_arc_pppoe = config_engine.get_active_config_arc();
    tokio::spawn(async move {
        pppoe_manager_clone.run(config_arc_pppoe).await;
    });

    // Jalankan RDP Discovery
    let rdp_state_clone = rdp_state.clone();
    let config_arc_rdp = config_engine.get_active_config_arc();
    tokio::spawn(async move {
        if let Err(e) = neighbor_discovery::run_rdp_service(rdp_state_clone, config_arc_rdp).await {
            log::error!("RDP Service error: {}", e);
        }
    });

    // Jalankan DHCP Server
    let lease_pool_clone = lease_pool.clone();
    let config_arc = config_engine.get_active_config_arc();
    tokio::spawn(async move {
        if let Err(e) = dhcp_server::run_dhcp_server(config_arc, lease_pool_clone).await {
            log::error!("DHCP Server error: {}", e);
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
            if let Ok(mut sni_map) = aya::maps::HashMap::<_, [u8; 32], u32>::try_from(bpf.map_mut("SNI_BLACKLIST").unwrap()) {
                // Clear map (atau update selisihnya, tapi clear-reload paling aman untuk demo)
                // Note: Aya HashMap tidak punya clear(), jadi kita biarkan menumpuk atau kelola manual.
                // Untuk kesederhanaan, kita masukkan yang baru.
                for domain in blocked {
                    let mut key = [0u8; 32];
                    let bytes = domain.as_bytes();
                    let len = bytes.len().min(32);
                    key[..len].copy_from_slice(&bytes[..len]);
                    let _ = sni_map.insert(key, 1, 0);
                }
            }
        }
    });

    let dns_state_server = dns_state.clone();
    let config_arc_server = config_engine.get_active_config_arc();
    tokio::spawn(async move {
        if let Err(e) = dns_server::run_dns_server(dns_state_server, config_arc_server).await {
            log::error!("DNS Server error: {}", e);
        }
    });

    println!("Press Ctrl+C to terminate and detach eBPF...");
    signal::ctrl_c().await?;
    println!("Detaching eBPF and exiting...");

    Ok(())
}
