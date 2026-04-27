# ❄️ Rouman Router: Technical Knowledge & User Guide

Welcome to the official documentation for **Rouman Router**. This guide provides a comprehensive overview of the system's features and tutorials on how to manage your network effectively using the **Premium Frost** interface.

---

## 📊 1. System Dashboard
The Dashboard is your command center. It provides real-time telemetry on the health of your router.

- **CPU Load**: Displays processing power usage. If load is consistently above 80%, consider reducing the number of active Automation scripts.
- **Memory Utilization**: Shows RAM usage. Rouman's Rust-based core is highly efficient, typically staying below 200MB.
- **Internet Status**: A real-time heart-beat indicator. 
    - 🟢 **CONNECTED**: System can reach global DNS (8.8.8.8).
    - 🔴 **DISCONNECTED**: Check your WAN interfaces.

---

## 🌐 2. Interface Management
Located under **Interfaces**, this is where you define how your router connects to the world.

### Tutorial: Adding a New Interface
1. Navigate to **Interfaces**.
2. Select your physical interface from the **Dropdown List** (e.g., `eth0`).
3. Choose the **Type**: `WAN` (Internet) or `LAN` (Local Network).
4. Set the **Mode**: `DHCP` (Automatic) or `Static` (Manual).
5. Click **Save Config_** and then **Apply Changes_** at the top.

---

## 🛣️ 3. Routing & Multi-WAN
Rouman supports surgical Multi-WAN routing with automatic failover.

- **Static Routes**: Manually route traffic to specific subnets through chosen gateways.
- **Multi-WAN Policy**: 
    - **Failover**: Traffic uses WAN1; if WAN1 fails, it switches to WAN2 automatically.
    - **Load Balance**: Traffic is distributed across multiple WAN links.
- **Surgical Updates**: Changes to routing happen instantly without dropping active connections.

---

## 🛡️ 4. Firewall & Security
The firewall operates at two levels: Standard IP Tables and Kernel-level eBPF (XDP).

### IP Blacklist
Enter any malicious IP address to drop all incoming traffic from that source. This is processed at wire-speed by the eBPF engine.

### SNI Filtering (DPI)
Block websites by their domain names (e.g., `facebook.com`). This is more robust than DNS blocking because it inspects the TLS handshake directly in the kernel.

---

## 🛠️ 5. Services
Configure core network services for your LAN clients.

- **DHCP Server**: Assigns IP addresses to devices on your network. 
    - *Tip*: Always ensure your LAN interface has a static IP before enabling DHCP.
- **DNS Server**: Handles name resolution. You can add "Blocked Domains" here for simple content filtering.
- **Advanced Proxy**: A high-performance reverse proxy for hosting local services securely.

---

## ☁️ 6. Cloudflare Integration
Manage your Cloudflare Tunnels directly from the Rouman UI.

1. Obtain your **Tunnel Token** from the Cloudflare Zero Trust dashboard.
2. Enter the token in the **Cloudflare** menu.
3. Click **Start Tunnel_**. 
4. The system supervisor will ensure the tunnel stays up 24/7.

---

## 🤖 7. Automation
Create custom logic using the Rouman Automation engine.

- **Triggers**: Define events (e.g., "When WAN1 goes down").
- **Actions**: Define responses (e.g., "Send notification" or "Switch to Backup link").
- **Visual Editor**: Use the drag-and-drop interface to build complex logic without coding.

---

## ⚙️ 8. System Configuration
General system maintenance and safety.

- **Atomic Config**: Every time you save, Rouman performs an "Atomic Write". This means even if you lose power mid-save, your configuration file will not be corrupted.
- **Logout_**: Securely end your session. The system uses session-based tokens for maximum security.

---

## 💡 Pro Tips for Robustness
1. **Apply Changes_**: Always remember to click the global "Apply" button to commit your candidate configuration to the active system.
2. **Supervision**: If a service like DHCP seems stuck, don't worry—the system supervisor will automatically restart it within 5-10 seconds.
3. **Dropdowns**: Always use the dropdown lists for interface selection to avoid typos that could break your connectivity.

---
*Rouman Router - Engineered for Stability. Designed for Elegance.*
