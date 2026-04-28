use clap::Parser;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP of the Rouman Core
    #[arg(long)]
    core_ip: String,

    /// Authentication token
    #[arg(long)]
    token: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!("Starting Rouman Compute Agent...");
    println!("Core IP: {}", args.core_ip);
    
    // Accept self-signed certs from the Core
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap_or_default();
        
    let url = format!("https://{}:8443/api/v1/compute/agent/register", args.core_ip);
    
    let hostname = hostname::get()
        .unwrap_or_else(|_| std::ffi::OsString::from("unknown-agent"))
        .to_string_lossy()
        .to_string();

    let wg_public_key = std::fs::read_to_string("/etc/wireguard/publickey")
        .unwrap_or_else(|_| "dummy_pub_key_if_not_found".to_string())
        .trim()
        .to_string();

    // Basic IP detection (in a real scenario, this is complex due to multiple interfaces)
    // Here we'll just let the Core record the IP from the request, but we'll send a dummy one
    let ip = "0.0.0.0".to_string(); // In a real scenario, we'd use a crate like local-ip-address

    println!("Agent Hostname: {}", hostname);
    println!("Starting Heartbeat Loop...");

    loop {
        let payload = json!({
            "token": args.token,
            "hostname": hostname,
            "ip": ip,
            "wg_public_key": wg_public_key,
        });

        match client.post(&url)
            .json(&payload)
            .send()
            .await 
        {
            Ok(res) => {
                if res.status().is_success() {
                    println!("Heartbeat sent successfully.");
                } else {
                    println!("Heartbeat failed: HTTP {}", res.status());
                }
            }
            Err(e) => {
                println!("Failed to connect to Rouman Core: {}", e);
            }
        }

        // Sleep for 60 seconds before next heartbeat
        sleep(Duration::from_secs(60)).await;
    }
}
