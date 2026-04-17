use axum::{
    routing::{get, post},
    Router, Json, extract::State,
    response::Html,
};
use std::sync::Arc;
use crate::database::Database;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Clone)]
pub struct HotspotState {
    pub db: Arc<Database>,
}

pub async fn run_hotspot_server(db: Arc<Database>) {
    let state = HotspotState { db };

    let app = Router::new()
        .route("/", get(landing_page))
        .route("/login", post(handle_login))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    log::info!("Hotspot Landing Page listening on 0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
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

async fn handle_login(
    State(state): State<HotspotState>,
    axum::extract::Form(req): axum::extract::Form<LoginRequest>,
) -> Html<&'static str> {
    let username = req.username;
    let password = req.password.unwrap_or_default();

    // 1. Verifikasi via RADIUS DB (Simulasi atau panggil internal)
    // Check database
    let user = sqlx::query(
        "SELECT id FROM radius_users WHERE username = ? AND password = ?"
    ).bind(&username)
     .bind(&password)
     .fetch_optional(&state.db.pool).await.unwrap_or(None);

    if let Some(_) = user {
        // 2. Berikan akses di nftables (Authorized Set)
        // Note: MAC Address biasanya didapat dari ARP atau header khusus jika lewat proxy
        // Untuk demo, kita asumsikan IP klien (tapi MAC lebih aman)
        // Kita butuh MAC Address klien. Di Axum, kita bisa coba ambil dari socket addr tapi butuh lookup ARP.
        log::info!("User {} successfully authenticated!", username);

        return Html("<h1>Success! You are now connected.</h1><p>Please wait while we redirect you...</p>");
    }

    Html("<h1>Login Failed</h1><p>Invalid voucher code or password.</p><a href='/'>Try Again</a>")
}
