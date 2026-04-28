use super::RoumanConfig;
use std::process::Command;


pub fn run_net_cmd(cmd: &str, args: &[&str], critical: bool) -> Result<(), String> {
    let output = std::process::Command::new(cmd).args(args).output().map_err(|e| format!("Failed to execute '{}': {}", cmd, e))?;
    
    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        let msg = format!("Command '{} {:?}' failed: {}", cmd, args, err_msg.trim());
        
        if critical {
            return Err(msg);
        } else {
            eprintln!("Non-critical network error: {}", msg);
        }
    }
    Ok(())
}

pub async fn apply_config(new_config: &RoumanConfig) -> Result<(), String> {
    // 0. Manajemen Interface (VLAN & Bridge)
    let iface_cfg = &new_config.network.interfaces;

    // 0.1 Buat Bridge
    for br in &iface_cfg.bridges {
        if br.enabled {
            // Cek jika sudah ada
            run_net_cmd("ip", &["link", "add", "name", &br.name, "type", "bridge"], false)?;
            run_net_cmd("ip", &["link", "set", &br.name, "up"], false)?;

            for port in &br.members {
                run_net_cmd("ip", &["link", "set", port, "master", &br.name], false)?;
                run_net_cmd("ip", &["link", "set", port, "up"], false)?;
            }
        }
    }

    // 0.2 Buat VLAN
    for vlan in &iface_cfg.vlans {
        if vlan.enabled {
            let _ = Command::new("ip").args([
                "link", "add", "link", &vlan.parent, "name", &vlan.name, "type", "vlan", "id", &vlan.vlan_id.to_string()
            ]).output();
            run_net_cmd("ip", &["link", "set", &vlan.name, "up"], false)?;
        }
    }

    // 0.3 IP Assignment
    for assign in &iface_cfg.assignments {
        if assign.enabled {
            // Flush IP lama agar bersih (hati-hati untuk eth0/WAN)
            if assign.interface != "eth0" && !assign.interface.contains("ppp") {
                 run_net_cmd("ip", &["addr", "flush", "dev", &assign.interface], false)?;
            }
            run_net_cmd("ip", &["addr", "add", &assign.address, "dev", &assign.interface], false)?;
            run_net_cmd("ip", &["link", "set", &assign.interface, "up"], false)?;
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
                run_net_cmd("hostname", &[hostname], false)?;
            }
        },
        Err(_) => {
             run_net_cmd("hostname", &[hostname], false)?;
        }
    }
    
    // 2. Manipulasi Routing (NAT / Masquerade / Multi-WAN)
    let net = &new_config.network;
    
    // Surgical Reset: Hapus hanya rule Multi-WAN lama (Priority 100-200)
    // agar tidak memutus koneksi sistem global (ip rule flush is too dangerous)
    for priority in 100..=200 {
        run_net_cmd("ip", &["rule", "del", "priority", &priority.to_string()], false)?;
    }

    if net.enable_nat {
        // Aktifkan IP Forwarding
        run_net_cmd("sysctl", &["-w", "net.ipv4.ip_forward=1"], false)?;

        // Flush NFTables
        run_net_cmd("nft", &["delete", "table", "ip", "rouman_nat"], false)?;
        run_net_cmd("nft", &["delete", "table", "ip", "rouman_mangle"], false)?;

        // Buat Kerangka Dasar NAT & Mangle
        run_net_cmd("nft", &["add", "table", "ip", "rouman_nat"], false)?;
        run_net_cmd("nft", &["add", "table", "ip", "rouman_mangle"], false)?;
        
        run_net_cmd("nft", &["add", "chain", "ip", "rouman_nat", "postrouting", "{ type nat hook postrouting priority 100 ; }"], false)?;
        run_net_cmd("nft", &["add", "chain", "ip", "rouman_nat", "prerouting", "{ type nat hook prerouting priority -100 ; }"], false)?;
        run_net_cmd("nft", &["add", "chain", "ip", "rouman_mangle", "prerouting", "{ type filter hook prerouting priority -150 ; }"], false)?;

        // MSS Clamping
        run_net_cmd("nft", &["add", "rule", "ip", "rouman_nat", "postrouting", "tcp", "flags", "syn", "tcp", "option", "maxseg", "size", "set", "rt", "mtu"], false)?;

        // 2.1 Multi-WAN Configuration
        let active_wans: Vec<_> = net.wans.iter().filter(|w| w.enabled).collect();
        let total_weight: u32 = active_wans.iter().map(|w| w.weight).sum();

        for (i, wan) in active_wans.iter().enumerate() {
            let table_id = 100 + i;
            let mark_id = 100 + i;
            let iface = if wan.pppoe_enabled { "ppp0" } else { &wan.interface };

            // A. Setup Routing Table per WAN
            run_net_cmd("ip", &["route", "flush", "table", &table_id.to_string()], false)?;
            run_net_cmd("ip", &["route", "add", "default", "via", &wan.gateway, "dev", iface, "table", &table_id.to_string()], false)?;
            
            // B. IP Rule for Marking (with explicit priority for surgical cleanup)
            run_net_cmd("ip", &["rule", "add", "fwmark", &mark_id.to_string(), "table", &table_id.to_string(), "priority", &mark_id.to_string()], false)?;

            // C. PCC Logic (jhash)
            if net.lb_mode == super::LoadBalancingMode::Pcc && total_weight > 0 {
                // Sederhananya kita bagi berdasarkan modulus total_weight (PCC ala Rouman)
                let pcc_rule = format!(
                    "add rule ip rouman_mangle prerouting iifname != \"{}\" meta mark set jhash ip saddr . ip daddr mod {} offset {} ",
                    iface, total_weight, mark_id
                );
                // Note: Implementasi PCC yang lebih presisi membutuhkan penanganan per-weight. 
                // Untuk versi ini kita gunakan distribusi merata.
                let args: Vec<&str> = pcc_rule.split_whitespace().collect();
                run_net_cmd("nft", &args, false)?;
            }

            // D. Masquerade untuk WAN ini
            let masquerade_rule = format!("add rule ip rouman_nat postrouting oifname \"{}\" masquerade", iface);
            let args: Vec<&str> = masquerade_rule.split_whitespace().collect();
            run_net_cmd("nft", &args, false)?;

            // E. Port Forwarding untuk WAN ini
            for pf in &net.port_forwards {
                let pf_rule = format!(
                    "add rule ip rouman_nat prerouting iifname \"{}\" {} dport {} dnat to {}:{}",
                    iface, pf.proto, pf.ext_port, pf.int_ip, pf.int_port
                );
                let args: Vec<&str> = pf_rule.split_whitespace().collect();
                run_net_cmd("nft", &args, false)?;
            }
        }

        // 2.2 Transparent DNS Interception
        if new_config.dns.transparent_intercept {
            let dns_redirect_udp = "add rule ip rouman_nat prerouting udp dport 53 redirect to :53";
            let dns_redirect_tcp = "add rule ip rouman_nat prerouting tcp dport 53 redirect to :53";
            
            let args_udp: Vec<&str> = dns_redirect_udp.split_whitespace().collect();
            run_net_cmd("nft", &args_udp, false)?;
            
            let args_tcp: Vec<&str> = dns_redirect_tcp.split_whitespace().collect();
            run_net_cmd("nft", &args_tcp, false)?;
        }

        // 2.3 Failover Mode (Main Route Table)
        if net.lb_mode == super::LoadBalancingMode::Failover || net.lb_mode == super::LoadBalancingMode::None {
            run_net_cmd("ip", &["route", "flush", "table", "main", "type", "unicast", "scope", "global"], false)?;
            for wan in &active_wans {
                let iface = if wan.pppoe_enabled { "ppp0" } else { &wan.interface };
                let metric = wan.distance * 10;
                let metric_str = metric.to_string();
                run_net_cmd("ip", &[
                    "route", "add", "default", "via", &wan.gateway, "dev", iface, "metric", &metric_str
                ], false)?;
            }
        }

        // 2.4 Hotspot & Captive Portal Redirection
        // Buat set untuk MAC yang sudah terautentikasi
        run_net_cmd("nft", &["add", "set", "ip", "rouman_nat", "authorized_macs", "{ type ether_addr ; }"], false)?;
        
        // Redirect port 80 ke 8080 (Hotspot Landing Page) untuk yang belum login
        // Note: Kita bypass traffic DNS (53) agar deteksi captive portal OS lancar
        let hotspot_redirect = "add rule ip rouman_nat prerouting tcp dport 80 ether saddr != @authorized_macs redirect to :8080";
        let args: Vec<&str> = hotspot_redirect.split_whitespace().collect();
        run_net_cmd("nft", &args, false)?;

    } else {
        // Matikan Forwarding
        run_net_cmd("sysctl", &["-w", "net.ipv4.ip_forward=0"], false)?;
        run_net_cmd("nft", &["delete", "table", "ip", "rouman_nat"], false)?;
        run_net_cmd("nft", &["delete", "table", "ip", "rouman_mangle"], false)?;
    }

    // 3. Traffic Shaping (QoS) - CAKE
    let qos = &new_config.network.qos;
    for wan in &new_config.network.wans {
        if !wan.enabled { continue; }
        let iface = if wan.pppoe_enabled { "ppp0" } else { &wan.interface };

        // Hapus qdisc lama jika ada
        run_net_cmd("tc", &["qdisc", "del", "dev", iface, "root"], false)?;
        
        if qos.upload_mbps > 0 {
            // Gunakan parameter upload_mbps untuk pembatasan egress (keluar kearah internet) di interface WAN
            let bandwidth = format!("{}mbit", qos.upload_mbps);
            run_net_cmd("tc", &[
                "qdisc", "add", "dev", iface, "root", "cake", "bandwidth", &bandwidth
            ], false)?;
        }
    }

    // 6. Simple Queues (Bandwidth Management)
    for assign in &new_config.network.interfaces.assignments {
        if !assign.enabled { continue; }
        let iface = &assign.interface;

        // Reset TC pada interface lokal
        run_net_cmd("tc", &["qdisc", "del", "dev", iface, "root"], false)?;
        run_net_cmd("tc", &["qdisc", "del", "dev", iface, "ingress"], false)?;

        if !new_config.network.simple_queues.is_empty() {
             // 6.1 Setup Root HTB untuk Download (Egress LAN)
             run_net_cmd("tc", &["qdisc", "add", "dev", iface, "root", "handle", "1:", "htb", "default", "1"], false)?;
             run_net_cmd("tc", &["qdisc", "add", "dev", iface, "ingress"], false)?;

             for (idx, q) in new_config.network.simple_queues.iter().enumerate() {
                 if !q.enabled { continue; }
                 let class_id = 10 + idx;
                 
                 // Limit Download (Egress kearah klien)
                 let down_rate = format!("{}mbit", q.download_mbps);
                 let class_id_str = format!("1:{}", class_id);
                 run_net_cmd("tc", &[
                     "class", "add", "dev", iface, "parent", "1:", "classid", &class_id_str, 
                     "htb", "rate", &down_rate, "ceil", &down_rate
                 ], false)?;

                 run_net_cmd("tc", &[
                     "filter", "add", "dev", iface, "protocol", "ip", "parent", "1:0", "prio", "1", 
                     "u32", "match", "ip", "dst", &q.target, "flowid", &class_id_str
                 ], false)?;

                 // Limit Upload (Ingress dari klien) - Menggunakan Policier sederhana
                 let up_rate = format!("{}mbit", q.upload_mbps);
                 let burst = format!("{}kbit", q.upload_mbps * 128); // Sederhana
                 run_net_cmd("tc", &[
                     "filter", "add", "dev", iface, "parent", "ffff:", "protocol", "ip", "prio", "1", 
                     "u32", "match", "ip", "src", &q.target, "police", "rate", &up_rate, "burst", &burst, "drop"
                 ], false)?;
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
