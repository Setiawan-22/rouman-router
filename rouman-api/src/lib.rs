use std::sync::Arc;
use axum::{routing::{get, post}, Router, Json, extract::{State, FromRef}};
use tower_http::services::{ServeDir, ServeFile};
use tokio::time::{timeout, Duration};
use tokio::sync::Mutex;
use serde::Deserialize;
use std::time::SystemTime;

pub mod auth;
pub mod services;
pub mod network;
pub mod config;
pub mod firewall;
pub mod dns;
pub mod automation;
pub mod database;

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
    let db = Arc::new(database::Database::new("sqlite://radius.db").await.unwrap());
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
            .route("/network/interfaces", get(interfaces_handler))
            .route("/network/leases", get(leases_handler))
            .route("/services/cloudflare/start", post(cf_start))
            .route("/services/cloudflare/stop", post(cf_stop))
            .route("/services/cloudflare/status", get(cf_status))
            .route("/config/active", get(config::get_active))
            .route("/config/candidate", get(config::get_candidate))
            .route("/config/candidate", axum::routing::put(config::put_candidate))
            .route("/config/commit", post(config::commit_config))
            .route("/config/rollback", post(config::rollback_config))
            .nest("/firewall", Router::<AppState>::new()
                .route("/stats", get(firewall::get_stats))
                .route("/blacklist", post(firewall::add_blacklist))
            )
            .nest("/dns", dns::dns_routes())
            .nest("/radius", network::radius::radius_routes())
            .nest("/proxy", network::proxy::proxy_routes())
            .nest("/network", Router::<AppState>::new()
                .route("/wireguard/config", get(network::wireguard::get_wireguard_config))
                .route("/wireguard/status", get(network::wireguard::get_wg_status))
                .route("/wireguard/generate-keys", post(network::wireguard::generate_keys))
                .route("/wireguard/client-config/:iface/:peer", get(network::wireguard::get_client_config))
                .route("/cloudflare/config", get(network::cloudflare::get_config))
                .route("/cloudflare/check", get(network::cloudflare::check_binary))
                .route("/neighbors", get(network::neighbors::get_neighbors))
                .route("/pppoe/status", get(network::pppoe::get_pppoe_status))
                .route("/automation/test", post(automation::run_test_script))
            )
            .route_layer(axum::middleware::from_fn(auth::auth_middleware))
        )
        .with_state(state)
        .fallback_service(serve_dir);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
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

async fn cf_start(State(_): State<AppState>, Json(payload): Json<CfStartPayload>) -> Json<serde_json::Value> {
    match services::cloudflared::start(&payload.token).await {
        Ok(_) => Json(serde_json::json!({ "status": "started" })),
        Err(e) => Json(serde_json::json!({ "error": format!("{:?}", e) })),
    }
}

async fn cf_stop(State(_): State<AppState>) -> Json<serde_json::Value> {
    match services::cloudflared::stop().await {
        Ok(_) => Json(serde_json::json!({ "status": "stopped" })),
        Err(e) => Json(serde_json::json!({ "error": format!("{:?}", e) })),
    }
}

async fn cf_status(State(_): State<AppState>) -> Json<serde_json::Value> {
    let running = services::cloudflared::status().await;
    Json(serde_json::json!({ "running": running }))
}

static SYSTEM_INFO: Mutex<Option<sysinfo::System>> = Mutex::const_new(None);

async fn telemetry(State(_): State<AppState>) -> Json<serde_json::Value> {
    let mut sys_guard = SYSTEM_INFO.lock().await;
    if sys_guard.is_none() {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        *sys_guard = Some(sys);
    }
    
    let sys = sys_guard.as_mut().unwrap();
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

async fn interfaces_handler(State(_): State<AppState>) -> Json<serde_json::Value> {
    match network::interfaces::get_all_interfaces().await {
        Ok(ifaces) => Json(serde_json::json!({ "interfaces": ifaces })),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}
