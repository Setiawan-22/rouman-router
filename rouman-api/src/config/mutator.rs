use super::RoumanConfig;
use std::process::Command;

pub async fn apply_config(new_config: &RoumanConfig) -> Result<(), String> {
    // 0. Manajemen Interface (VLAN & Bridge)
    let iface_cfg = &new_config.network.interfaces;

    // 0.1 Buat Bridge
    for br in &iface_cfg.bridges {
        if br.enabled {
            // Cek jika sudah ada
            let _ = Command::new("ip").args(["link", "add", "name", &br.name, "type", "bridge"]).output();
            let _ = Command::new("ip").args(["link", "set", &br.name, "up"]).output();

            for port in &br.members {
                let _ = Command::new("ip").args(["link", "set", port, "master", &br.name]).output();
                let _ = Command::new("ip").args(["link", "set", port, "up"]).output();
            }
        }
    }

    // 0.2 Buat VLAN
    for vlan in &iface_cfg.vlans {
        if vlan.enabled {
            let _ = Command::new("ip").args([
                "link", "add", "link", &vlan.parent, "name", &vlan.name, "type", "vlan", "id", &vlan.vlan_id.to_string()
            ]).output();
            let _ = Command::new("ip").args(["link", "set", &vlan.name, "up"]).output();
        }
    }

    // 0.3 IP Assignment
    for assign in &iface_cfg.assignments {
        if assign.enabled {
            // Flush IP lama agar bersih (hati-hati untuk eth0/WAN)
            if assign.interface != "eth0" && !assign.interface.contains("ppp") {
                 let _ = Command::new("ip").args(["addr", "flush", "dev", &assign.interface]).output();
            }
            let _ = Command::new("ip").args(["addr", "add", &assign.address, "dev", &assign.interface]).output();
            let _ = Command::new("ip").args(["link", "set", &assign.interface, "up"]).output();
        }
    }

    // 1. Perubahan Hostname OS Linux
    let hostname = &new_config.system.hostname;
    
    // Kita panggil `hostnamectl` untuk mengubah hostname mesin lokal dan persisten
    let output = Command::new("hostnamectl")
        .arg("set-hostname")
        .arg(hostname)
        .output();
        
    match output {
        Ok(out) => {
            if !out.status.success() {
                let _ = Command::new("hostname").arg(hostname).output();
            }
        },
        Err(_) => {
             let _ = Command::new("hostname").arg(hostname).output();
        }
    }
    
    // 2. Manipulasi Routing (NAT / Masquerade / Multi-WAN)
    let net = &new_config.network;
    
    // Surgical Reset: Hapus hanya rule Multi-WAN lama (Priority 100-200)
    // agar tidak memutus koneksi sistem global (ip rule flush is too dangerous)
    for priority in 100..=200 {
        let _ = Command::new("ip").args(["rule", "del", "priority", &priority.to_string()]).output();
    }

    if net.enable_nat {
        // Aktifkan IP Forwarding
        let _ = Command::new("sysctl").arg("-w").arg("net.ipv4.ip_forward=1").output();

        // Flush NFTables
        let _ = Command::new("nft").args(["delete", "table", "ip", "rouman_nat"]).output();
        let _ = Command::new("nft").args(["delete", "table", "ip", "rouman_mangle"]).output();

        // Buat Kerangka Dasar NAT & Mangle
        let _ = Command::new("nft").args(["add", "table", "ip", "rouman_nat"]).output();
        let _ = Command::new("nft").args(["add", "table", "ip", "rouman_mangle"]).output();
        
        let _ = Command::new("nft").args(["add", "chain", "ip", "rouman_nat", "postrouting", "{ type nat hook postrouting priority 100 ; }"]).output();
        let _ = Command::new("nft").args(["add", "chain", "ip", "rouman_nat", "prerouting", "{ type nat hook prerouting priority -100 ; }"]).output();
        let _ = Command::new("nft").args(["add", "chain", "ip", "rouman_mangle", "prerouting", "{ type filter hook prerouting priority -150 ; }"]).output();

        // MSS Clamping
        let _ = Command::new("nft").args(["add", "rule", "ip", "rouman_nat", "postrouting", "tcp", "flags", "syn", "tcp", "option", "maxseg", "size", "set", "rt", "mtu"]).output();

        // 2.1 Multi-WAN Configuration
        let active_wans: Vec<_> = net.wans.iter().filter(|w| w.enabled).collect();
        let total_weight: u32 = active_wans.iter().map(|w| w.weight).sum();

        for (i, wan) in active_wans.iter().enumerate() {
            let table_id = 100 + i;
            let mark_id = 100 + i;
            let iface = if wan.pppoe_enabled { "ppp0" } else { &wan.interface };

            // A. Setup Routing Table per WAN
            let _ = Command::new("ip").args(["route", "flush", "table", &table_id.to_string()]).output();
            let _ = Command::new("ip").args(["route", "add", "default", "via", &wan.gateway, "dev", iface, "table", &table_id.to_string()]).output();
            
            // B. IP Rule for Marking (with explicit priority for surgical cleanup)
            let _ = Command::new("ip").args(["rule", "add", "fwmark", &mark_id.to_string(), "table", &table_id.to_string(), "priority", &mark_id.to_string()]).output();

            // C. PCC Logic (jhash)
            if net.lb_mode == super::LoadBalancingMode::Pcc && total_weight > 0 {
                // Sederhananya kita bagi berdasarkan modulus total_weight (PCC ala Rouman)
                let pcc_rule = format!(
                    "add rule ip rouman_mangle prerouting iifname != \"{}\" meta mark set jhash ip saddr . ip daddr mod {} offset {} ",
                    iface, total_weight, mark_id
                );
                // Note: Implementasi PCC yang lebih presisi membutuhkan penanganan per-weight. 
                // Untuk versi ini kita gunakan distribusi merata.
                let _ = Command::new("nft").args(pcc_rule.split_whitespace()).output();
            }

            // D. Masquerade untuk WAN ini
            let masquerade_rule = format!("add rule ip rouman_nat postrouting oifname \"{}\" masquerade", iface);
            let _ = Command::new("nft").args(masquerade_rule.split_whitespace()).output();

            // E. Port Forwarding untuk WAN ini
            for pf in &net.port_forwards {
                let pf_rule = format!(
                    "add rule ip rouman_nat prerouting iifname \"{}\" {} dport {} dnat to {}:{}",
                    iface, pf.proto, pf.ext_port, pf.int_ip, pf.int_port
                );
                let _ = Command::new("nft").args(pf_rule.split_whitespace()).output();
            }
        }

        // 2.2 Transparent DNS Interception
        if new_config.dns.transparent_intercept {
            let dns_redirect_udp = "add rule ip rouman_nat prerouting udp dport 53 redirect to :53";
            let dns_redirect_tcp = "add rule ip rouman_nat prerouting tcp dport 53 redirect to :53";
            let _ = Command::new("nft").args(dns_redirect_udp.split_whitespace()).output();
            let _ = Command::new("nft").args(dns_redirect_tcp.split_whitespace()).output();
        }

        // 2.3 Failover Mode (Main Route Table)
        if net.lb_mode == super::LoadBalancingMode::Failover || net.lb_mode == super::LoadBalancingMode::None {
            let _ = Command::new("ip").args(["route", "flush", "table", "main", "type", "unicast", "scope", "global"]).output();
            for wan in &active_wans {
                let iface = if wan.pppoe_enabled { "ppp0" } else { &wan.interface };
                let metric = wan.distance * 10;
                let _ = Command::new("ip").args([
                    "route", "add", "default", "via", &wan.gateway, "dev", iface, "metric", &metric.to_string()
                ]).output();
            }
        }

        // 2.4 Hotspot & Captive Portal Redirection
        // Buat set untuk MAC yang sudah terautentikasi
        let _ = Command::new("nft").args(["add", "set", "ip", "rouman_nat", "authorized_macs", "{ type ether_addr ; }"]).output();
        
        // Redirect port 80 ke 8080 (Hotspot Landing Page) untuk yang belum login
        // Note: Kita bypass traffic DNS (53) agar deteksi captive portal OS lancar
        let hotspot_redirect = "add rule ip rouman_nat prerouting tcp dport 80 ether saddr != @authorized_macs redirect to :8080";
        let _ = Command::new("nft").args(hotspot_redirect.split_whitespace()).output();

    } else {
        // Matikan Forwarding
        let _ = Command::new("sysctl").arg("-w").arg("net.ipv4.ip_forward=0").output();
        let _ = Command::new("nft").args(["delete", "table", "ip", "rouman_nat"]).output();
        let _ = Command::new("nft").args(["delete", "table", "ip", "rouman_mangle"]).output();
    }

    // 3. Traffic Shaping (QoS) - CAKE
    let qos = &new_config.network.qos;
    for wan in &new_config.network.wans {
        if !wan.enabled { continue; }
        let iface = if wan.pppoe_enabled { "ppp0" } else { &wan.interface };

        // Hapus qdisc lama jika ada
        let _ = Command::new("tc").args(["qdisc", "del", "dev", iface, "root"]).output();
        
        if qos.upload_mbps > 0 {
            // Gunakan parameter upload_mbps untuk pembatasan egress (keluar kearah internet) di interface WAN
            let bandwidth = format!("{}mbit", qos.upload_mbps);
            let _ = Command::new("tc").args([
                "qdisc", "add", "dev", iface, "root", "cake", "bandwidth", &bandwidth
            ]).output();
        }
    }

    // 6. Simple Queues (Bandwidth Management)
    for assign in &new_config.network.interfaces.assignments {
        if !assign.enabled { continue; }
        let iface = &assign.interface;

        // Reset TC pada interface lokal
        let _ = Command::new("tc").args(["qdisc", "del", "dev", iface, "root"]).output();
        let _ = Command::new("tc").args(["qdisc", "del", "dev", iface, "ingress"]).output();

        if !new_config.network.simple_queues.is_empty() {
             // 6.1 Setup Root HTB untuk Download (Egress LAN)
             let _ = Command::new("tc").args(["qdisc", "add", "dev", iface, "root", "handle", "1:", "htb", "default", "1"]).output();
             let _ = Command::new("tc").args(["qdisc", "add", "dev", iface, "ingress"]).output();

             for (idx, q) in new_config.network.simple_queues.iter().enumerate() {
                 if !q.enabled { continue; }
                 let class_id = 10 + idx;
                 
                 // Limit Download (Egress kearah klien)
                 let down_rate = format!("{}mbit", q.download_mbps);
                 let _ = Command::new("tc").args([
                     "class", "add", "dev", iface, "parent", "1:", "classid", &format!("1:{}", class_id), 
                     "htb", "rate", &down_rate, "ceil", &down_rate
                 ]).output();

                 let _ = Command::new("tc").args([
                     "filter", "add", "dev", iface, "protocol", "ip", "parent", "1:0", "prio", "1", 
                     "u32", "match", "ip", "dst", &q.target, "flowid", &format!("1:{}", class_id)
                 ]).output();

                 // Limit Upload (Ingress dari klien) - Menggunakan Policier sederhana
                 let up_rate = format!("{}mbit", q.upload_mbps);
                 let burst = format!("{}kbit", q.upload_mbps * 128); // Sederhana
                 let _ = Command::new("tc").args([
                     "filter", "add", "dev", iface, "parent", "ffff:", "protocol", "ip", "prio", "1", 
                     "u32", "match", "ip", "src", &q.target, "police", "rate", &up_rate, "burst", &burst, "drop"
                 ]).output();
             }
        }
    }

    // 4. WireGuard Synchronization
    if let Err(e) = crate::network::wireguard::apply_wireguard_settings(&new_config.wireguard).await {
        return Err(format!("WireGuard sync failed: {}", e));
    }

    // 5. Cloudflare Zero Trust Synchronization
    // Note: Kita butuh pool database di sini. Untuk sementara kita asumsikan 
    // mutator bisa mendapatkan akses ke pool melalui injeksi atau singleton jika perlu.
    // Namun dalam arsitektur Rouman, kita akan memanggilnya dari commit_config di lib.rs
    // agar lebih bersih.
    
    Ok(())
}
