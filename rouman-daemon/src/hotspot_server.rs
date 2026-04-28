use axum::{
    routing::{get, post},
    Router, extract::{State, ConnectInfo},
    response::Html,
};
use std::sync::Arc;
use crate::database::Database;
use serde::Deserialize;
use std::process::Command;
use std::net::SocketAddr;

#[derive(Clone)]
pub struct HotspotState {
    pub db: Arc<Database>,
    pub bpf: Arc<tokio::sync::Mutex<aya::Ebpf>>,
}

pub async fn run_hotspot_server(db: Arc<Database>, bpf: Arc<tokio::sync::Mutex<aya::Ebpf>>) {
    let state = HotspotState { db, bpf };

    let app = Router::new()
        .route("/", get(landing_page))
        .route("/login", post(handle_login))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    log::info!("Hotspot Landing Page listening on 0.0.0.0:8080");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

async fn landing_page() -> Html<&'static str> {
    Html(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Rouman Hotspot Login</title>
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <style>
                body { font-family: sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background: #0f172a; color: white; }
                .card { background: #1e293b; padding: 2rem; border-radius: 1rem; box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1); width: 300px; text-align: center; }
                input { width: 100%; padding: 0.5rem; margin-bottom: 1rem; border-radius: 0.5rem; border: none; }
                button { width: 100%; padding: 0.5rem; background: #3b82f6; color: white; border: none; border-radius: 0.5rem; cursor: pointer; }
                .logo { font-size: 1.5rem; font-weight: bold; margin-bottom: 1rem; color: #60a5fa; }
            </style>
        </head>
        <body>
            <div class="card">
                <div class="logo">ROUMAN CORE</div>
                <form action="/login" method="post">
                    <input type="text" name="username" placeholder="Voucher / Username" required>
                    <input type="password" name="password" placeholder="Password (Optional)">
                    <button type="submit">Connect to Internet</button>
                </form>
            </div>
        </body>
        </html>
    "#)
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: Option<String>,
}

async fn get_mac_from_ip(ip: &str) -> Option<String> {
    if let Ok(output) = Command::new("ip").args(["neigh", "show", ip]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Format: 192.168.100.10 dev eth0 lladdr aa:bb:cc:dd:ee:ff REACHABLE
        for part in stdout.split_whitespace().collect::<Vec<_>>().windows(2) {
            if part[0] == "lladdr" {
                return Some(part[1].to_string());
            }
        }
    }
    None
}

fn parse_mac(mac_str: &str) -> Option<[u8; 6]> {
    let mut mac = [0u8; 6];
    let parts: Vec<&str> = mac_str.split(':').collect();
    if parts.len() != 6 { return None; }
    for i in 0..6 {
        mac[i] = u8::from_str_radix(parts[i], 16).ok()?;
    }
    Some(mac)
}

async fn handle_login(
    State(state): State<HotspotState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    axum::extract::Form(req): axum::extract::Form<LoginRequest>,
) -> Html<&'static str> {
    let username = req.username;
    let password = req.password.unwrap_or_default();
    let client_ip = addr.ip().to_string();

    // 1. Verifikasi via RADIUS DB
    let user = sqlx::query(
        "SELECT id FROM radius_users WHERE username = ? AND password = ?"
    ).bind(&username)
     .bind(&password)
     .fetch_optional(&state.db.pool).await.unwrap_or(None);

    if let Some(_) = user {
        log::info!("User {} successfully authenticated from IP {}", username, client_ip);

        // 2. Dapatkan MAC Address dari ARP/Neighbor table
        if let Some(mac_str) = get_mac_from_ip(&client_ip).await {
            log::info!("Authorizing MAC {} for user {}", mac_str, username);
            
            // 3. Tambahkan ke eBPF AUTHORIZED_MACS map
            if let Some(mac_bytes) = parse_mac(&mac_str) {
                let mut bpf = state.bpf.lock().await;
                if let Some(map) = bpf.map_mut("AUTHORIZED_MACS") {
                    if let Ok(mut auth_map) = aya::maps::HashMap::<_, [u8; 6], u32>::try_from(map) {
                        let _ = auth_map.insert(mac_bytes, 1, 0);
                    }
                }
            }

            // 4. Fallback ke NFTables untuk paket yang tidak kena XDP
            let nft_cmd = format!("add element ip rouman_nat authorized_macs {{ {} }}", mac_str);
            match Command::new("nft").args(nft_cmd.split_whitespace()).output() {
                Ok(out) if !out.status.success() => log::error!("Failed to authorize MAC in nftables: {}", String::from_utf8_lossy(&out.stderr)),
                Err(e) => log::error!("Failed to execute nft: {}", e),
                _ => {}
            }
            
            return Html("<h1>Success! You are now connected.</h1><p>Please wait while we redirect you...</p>");
        } else {
            log::warn!("Could not find MAC address for IP {}", client_ip);
            return Html("<h1>Warning: Authorized but MAC not found.</h1><p>Your device might not be able to browse the internet.</p>");
        }
    }

    Html("<h1>Login Failed</h1><p>Invalid voucher code or password.</p><a href='/'>Try Again</a>")
}
