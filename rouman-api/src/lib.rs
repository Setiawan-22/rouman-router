use std::sync::Arc;
extern crate axum_server;
use axum::{routing::{get, post}, Router, Json, extract::{State, FromRef}};
use tower_http::services::{ServeDir, ServeFile};
use tokio::time::{timeout, Duration};
use tokio::sync::Mutex;
use serde::Deserialize;
use std::time::SystemTime;
use sqlx::Row;

pub mod auth;
pub mod services;
pub mod network;
pub mod config;
pub mod firewall;
pub mod dns;
pub mod automation;
pub mod database;
pub mod compute;
pub mod error;
pub mod os_detect;
#[derive(Clone)]
pub struct AppState {
    pub config_engine: Arc<config::ConfigEngine>,
    pub db: Arc<database::Database>,
    pub dns_state: Arc<dns::DnsState>,
    pub lease_pool: network::dhcp::SharedLeasePool,
    pub firewall_state: Arc<firewall::FirewallState>,
    pub rdp_state: Arc<network::neighbors::RdpState>,
    pub pppoe_state: Arc<network::pppoe::PppoeState>,
}

impl FromRef<AppState> for Arc<config::ConfigEngine> {
    fn from_ref(state: &AppState) -> Self {
        state.config_engine.clone()
    }
}

impl FromRef<AppState> for Arc<dns::DnsState> {
    fn from_ref(state: &AppState) -> Self {
        state.dns_state.clone()
    }
}

impl FromRef<AppState> for network::dhcp::SharedLeasePool {
    fn from_ref(state: &AppState) -> Self {
        state.lease_pool.clone()
    }
}

impl FromRef<AppState> for Arc<firewall::FirewallState> {
    fn from_ref(state: &AppState) -> Self {
        state.firewall_state.clone()
    }
}

impl FromRef<AppState> for Arc<network::neighbors::RdpState> {
    fn from_ref(state: &AppState) -> Self {
        state.rdp_state.clone()
    }
}

impl FromRef<AppState> for Arc<network::pppoe::PppoeState> {
    fn from_ref(state: &AppState) -> Self {
        state.pppoe_state.clone()
    }
}

pub async fn start_api_server(
    bpf: Arc<tokio::sync::Mutex<aya::Ebpf>>,
    rdp_state: Arc<network::neighbors::RdpState>,
    pppoe_state: Arc<network::pppoe::PppoeState>,
) -> (Arc<dns::DnsState>, network::dhcp::SharedLeasePool, Arc<config::ConfigEngine>) {
    let serve_dir = ServeDir::new("ui/build")
        .not_found_service(ServeFile::new("ui/build/index.html"));

    let config_engine = config::ConfigEngine::new().await;
    let db = Arc::new(database::Database::new("sqlite:/var/lib/rouman/radius.db").await.expect("Failed to open database"));
    let dns_state = Arc::new(dns::DnsState::default());
    let lease_pool = network::dhcp::SharedLeasePool::default();
    let firewall_state = Arc::new(firewall::FirewallState { ebpf: bpf });
    
    let state = AppState {
        config_engine: config_engine.clone(),
        db: db.clone(),
        dns_state: dns_state.clone(),
        lease_pool: lease_pool.clone(),
        firewall_state: firewall_state.clone(),
        rdp_state,
        pppoe_state,
    };

    let dns_state_for_daemon = dns_state.clone();
    let lease_pool_for_daemon = lease_pool.clone();
    let config_for_daemon = config_engine.clone();

    let app = Router::<AppState>::new()
        .route("/api/health", get(|| async { "OK" }))
        .nest("/api/auth", auth::auth_routes())
        .nest("/api/v1", Router::<AppState>::new()
            .route("/system/info", get(|| async { "Rouman System: OK" }))
            .route("/system/internet-status", get(internet_status))
            .route("/system/telemetry", get(telemetry))
            .route("/system/update/check", post(system_update_check))
            .route("/system/update/upgrade", post(system_update_upgrade))
            .route("/network/interfaces", get(interfaces_handler))
            .route("/network/leases", get(leases_handler))
            .route("/services/cloudflare/start", post(cf_start))
            .route("/services/cloudflare/stop", post(cf_stop))
            .route("/services/cloudflare/status", get(cf_status))
            .route("/config/active", get(config::get_active))
            .route("/config/candidate", get(config::get_candidate))
            .route("/config/candidate", axum::routing::put(config::put_candidate))
            .route("/config/commit", post(commit_config))
            .route("/config/rollback", post(config::rollback_config))
            .nest("/firewall", Router::<AppState>::new()
                .route("/stats", get(firewall::get_stats))
                .route("/blacklist", post(firewall::add_blacklist))
            )
            .nest("/dns", dns::dns_routes())
            .nest("/radius", network::radius::radius_routes())
            .nest("/proxy", network::proxy::proxy_routes())
            .nest("/compute", compute::compute_routes())
            .nest("/network", Router::<AppState>::new()
                .route("/wireguard/config", get(network::wireguard::get_wireguard_config))
                .route("/wireguard/status", get(network::wireguard::get_wg_status))
                .route("/wireguard/generate-keys", post(network::wireguard::generate_keys))
                .route("/wireguard/client-config/{iface}/{peer}", get(network::wireguard::get_client_config))
                .route("/cloudflare/config", get(network::cloudflare::get_config))
                .route("/cloudflare/check", get(network::cloudflare::check_binary))
                .route("/neighbors", get(network::neighbors::get_neighbors))
                .route("/pppoe/status", get(network::pppoe::get_pppoe_status))
                .route("/automation/test", post(automation::run_test_script))
            )
            .nest("/notifications", services::notifications::notification_routes())
            .route_layer(axum::middleware::from_fn(auth::auth_middleware))
        )
        .with_state(state)
        .fallback_service(serve_dir);

    // HTTP Server for Web UI
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    let app_http = app.clone();
    tokio::spawn(async move {
        axum::serve(listener, app_http).await.unwrap();
    });

    // HTTPS Server for Secure Agent Communication
    let cert_dir = "/etc/rouman/certs";
    std::fs::create_dir_all(cert_dir).unwrap_or_default();
    let cert_path = format!("{}/cert.pem", cert_dir);
    let key_path = format!("{}/key.pem", cert_dir);

    if !std::path::Path::new(&cert_path).exists() {
        println!("Generating self-signed TLS certificates for Compute Agents...");
        let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string(), "rouman-core".to_string()];
        let cert = rcgen::generate_simple_self_signed(subject_alt_names).expect("Failed to generate TLS cert");
        std::fs::write(&cert_path, cert.cert.pem()).unwrap();
        std::fs::write(&key_path, cert.signing_key.serialize_pem()).unwrap();
    }

    let app_https = app;
    tokio::spawn(async move {
        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path).await.unwrap();
        axum_server::bind_rustls("0.0.0.0:8443".parse::<std::net::SocketAddr>().unwrap(), tls_config)
            .serve(app_https.into_make_service())
            .await
            .unwrap();
    });

    (dns_state_for_daemon, lease_pool_for_daemon, config_for_daemon)
}

async fn leases_handler(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let leases = state.lease_pool.leases.lock().await;
    let result: Vec<serde_json::Value> = leases.values().map(|l| {
        serde_json::json!({
            "mac": l.mac.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(":"),
            "ip": l.ip.to_string(),
            "hostname": l.hostname,
            "expires": l.expires.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs(),
        })
    }).collect();
    
    Json(serde_json::json!({ "leases": result }))
}

async fn internet_status(State(_): State<AppState>) -> Json<serde_json::Value> {
    // Mengecek internet dengan mencoba koneksi TCP ke 8.8.8.8 (Google Public DNS)
    let addr = "8.8.8.8:53";
    let is_connected = match timeout(Duration::from_secs(2), tokio::net::TcpStream::connect(addr)).await {
        Ok(Ok(_)) => true,   // Sukses terkoneksi
        _ => false,          // Gagal / Timeout
    };
    
    Json(serde_json::json!({
        "status": if is_connected { "CONNECTED" } else { "DISCONNECTED" }
    }))
}

#[derive(Deserialize)]
struct CfStartPayload {
    token: String,
}

async fn cf_start(State(state): State<AppState>, Json(payload): Json<CfStartPayload>) -> Json<serde_json::Value> {
    match sqlx::query("UPDATE cloudflare_tunnels SET enabled = 1 WHERE token = ?")
        .bind(&payload.token)
        .execute(&state.db.pool)
        .await 
    {
        Ok(_) => Json(serde_json::json!({ "status": "instruction_sent_to_daemon" })),
        Err(e) => Json(serde_json::json!({ "error": format!("{:?}", e) })),
    }
}

async fn cf_stop(State(state): State<AppState>, Json(payload): Json<CfStartPayload>) -> Json<serde_json::Value> {
    match sqlx::query("UPDATE cloudflare_tunnels SET enabled = 0 WHERE token = ?")
        .bind(&payload.token)
        .execute(&state.db.pool)
        .await 
    {
        Ok(_) => Json(serde_json::json!({ "status": "instruction_sent_to_daemon" })),
        Err(e) => Json(serde_json::json!({ "error": format!("{:?}", e) })),
    }
}

async fn cf_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let row = sqlx::query("SELECT status FROM cloudflare_tunnels LIMIT 1")
        .fetch_optional(&state.db.pool)
        .await
        .unwrap_or(None);
    
    let status = match row {
        Some(r) => r.get::<String, _>("status"),
        None => "not_configured".to_string(),
    };
    
    Json(serde_json::json!({ "status": status }))
}

static SYSTEM_INFO: Mutex<Option<sysinfo::System>> = Mutex::const_new(None);

async fn telemetry(State(_): State<AppState>) -> Json<serde_json::Value> {
    let mut sys_guard = SYSTEM_INFO.lock().await;
    if sys_guard.is_none() {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        *sys_guard = Some(sys);
    }
    
    let sys = match sys_guard.as_mut() {
        Some(s) => s,
        None => return Json(serde_json::json!({ "error": "System info not available" })),
    };
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    
    let cpu_usage = sys.global_cpu_usage();
    let total_mem = sys.total_memory() / 1024 / 1024; // MB
    let used_mem = sys.used_memory() / 1024 / 1024; // MB
    let uptime = sysinfo::System::uptime(); // secs
    
    Json(serde_json::json!({
        "cpu": cpu_usage,
        "memory_total": total_mem,
        "memory_used": used_mem,
        "uptime": uptime,
    }))
}

pub async fn commit_config(State(state): State<crate::AppState>) -> Json<serde_json::Value> {
    let engine = &state.config_engine;
    match engine.commit().await {
        Ok(_) => {
            // Setelah commit sukses, sinkronkan tunnel ke database bersama
            let config = engine.active.read().await;
            if let Err(e) = network::cloudflare::sync_tunnels(&config.cloudflare, &state.db).await {
                return Json(serde_json::json!({ "status": "warning", "message": format!("Commit OK, but tunnel sync failed: {}", e) }));
            }
            Json(serde_json::json!({ "status": "success", "message": "Commit successful" }))
        },
        Err(e) => Json(serde_json::json!({ "status": "error", "message": e })),
    }
}

async fn interfaces_handler(State(_): State<AppState>) -> Json<serde_json::Value> {
    match network::interfaces::get_all_interfaces().await {
        Ok(ifaces) => Json(serde_json::json!({ "interfaces": ifaces })),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}

async fn system_update_check(State(_): State<AppState>) -> Json<serde_json::Value> {
    let os = crate::os_detect::detect_os();
    let command = match os {
        crate::os_detect::OsFamily::Fedora => "dnf check-update",
        crate::os_detect::OsFamily::Alpine => "apk update",
        _ => "apt-get update && apt-get --just-print upgrade", // Default to Debian/Ubuntu
    };

    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .await;

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            Json(serde_json::json!({
                "status": "success",
                "os_detected": os.to_string(),
                "stdout": stdout,
                "stderr": stderr
            }))
        }
        Err(e) => Json(serde_json::json!({ "status": "error", "message": format!("{}", e) })),
    }
}

async fn system_update_upgrade(State(_): State<AppState>) -> Json<serde_json::Value> {
    // SECURITY FIX: Disabling direct package upgrade via API for production safety.
    // It is highly recommended to perform system upgrades via SSH to monitor progress and handle prompts.
    let os = crate::os_detect::detect_os();
    let cmd_suggestion = match os {
        crate::os_detect::OsFamily::Fedora => "dnf upgrade",
        crate::os_detect::OsFamily::Alpine => "apk upgrade",
        _ => "apt-get upgrade",
    };

    Json(serde_json::json!({
        "status": "error",
        "message": format!("Direct system upgrade via Web UI is disabled for security reasons. Please use SSH to perform '{}' on your {} system.", cmd_suggestion, os.to_string())
    }))
}
