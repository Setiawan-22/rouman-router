use serde::{Deserialize, Serialize};
use hyperlocal::{Uri, UnixClientExt};
use http_body_util::{BodyExt, Full};
use bytes::Bytes;
use std::path::Path;
use log::{info, error};

#[derive(Serialize, Deserialize, Clone)]
pub struct VmConfig {
    pub id: String,
    pub vcpu_count: u32,
    pub mem_size_mib: u32,
    pub kernel_image_path: String,
    pub rootfs_path: String,
    pub mac_address: String,
}

pub struct FirecrackerClient {
    socket_path: String,
}

impl FirecrackerClient {
    pub fn new(id: &str) -> Self {
        Self {
            socket_path: format!("/tmp/firecracker-{}.socket", id),
        }
    }

    pub async fn start_vm(&self, config: &VmConfig) -> Result<(), Box<dyn std::error::Error>> {
        let client = hyper_util::client::legacy::Client::unix();

        // 1. Configure Boot Source (Kernel)
        let boot_source = serde_json::json!({
            "kernel_image_path": config.kernel_image_path,
            "boot_args": "console=ttyS0 reboot=k panic=1 pci=off"
        });
        self.put_api(&client, "/machine-config", &serde_json::json!({
            "vcpu_count": config.vcpu_count,
            "mem_size_mib": config.mem_size_mib,
        })).await?;

        self.put_api(&client, "/boot-source", &boot_source).await?;

        // 2. Configure Rootfs
        let drive = serde_json::json!({
            "drive_id": "rootfs",
            "path_on_host": config.rootfs_path,
            "is_root_device": true,
            "is_read_only": false
        });
        self.put_api(&client, "/drives/rootfs", &drive).await?;

        // 3. Configure Network (TAP)
        let tap_name = format!("tap-{}", config.id);
        let network = serde_json::json!({
            "iface_id": "eth0",
            "guest_mac": config.mac_address,
            "host_dev_name": tap_name
        });
        self.put_api(&client, "/network-interfaces/eth0", &network).await?;

        // 4. Start Instance Action
        let start_action = serde_json::json!({
            "action_type": "InstanceStart"
        });
        self.put_api(&client, "/actions", &start_action).await?;

        info!("Firecracker MicroVM {} started successfully on socket {}", config.id, self.socket_path);
        Ok(())
    }

    async fn put_api(
        &self, 
        client: &hyper_util::client::legacy::Client<hyperlocal::UnixConnector, Full<Bytes>>,
        path: &str, 
        payload: &serde_json::Value
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url: hyper::Uri = Uri::new(&self.socket_path, path).into();
        let body = Full::new(Bytes::from(payload.to_string()));
        let req = hyper::Request::builder()
            .method(hyper::Method::PUT)
            .uri(url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(body)?;

        let res = client.request(req).await?;
        if !res.status().is_success() {
            let body_bytes = res.into_body().collect().await?.to_bytes();
            let err_msg = String::from_utf8_lossy(&body_bytes);
            error!("Firecracker API Error {}: {}", path, err_msg);
            return Err(format!("API Error: {}", err_msg).into());
        }
        Ok(())
    }
}
