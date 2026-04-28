use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{SystemTime, Duration};
use std::collections::HashMap;
use tokio::sync::RwLock;
use rouman_api::config::RoumanConfig;
use rouman_api::network::dhcp::{SharedLeasePool, DhcpLease};

pub async fn run_dhcp_server(
    config: Arc<RwLock<RoumanConfig>>,
    lease_pool: SharedLeasePool,
    db: Arc<crate::database::Database>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Tunggu sampai konfigurasi valid
    let (enabled, interface, range_start, range_end, gateway, dns_servers, lease_time, subnet_mask) = {
        let cfg = config.read().await;
        let d = &cfg.network.dhcp;
        (
            d.enabled,
            d.interface.clone(),
            d.range_start.parse::<Ipv4Addr>().unwrap_or(Ipv4Addr::new(192, 168, 1, 100)),
            d.range_end.parse::<Ipv4Addr>().unwrap_or(Ipv4Addr::new(192, 168, 1, 200)),
            d.gateway.parse::<Ipv4Addr>().unwrap_or(Ipv4Addr::new(192, 168, 1, 1)),
            d.dns_servers.iter().filter_map(|s| s.parse::<Ipv4Addr>().ok()).collect::<Vec<_>>(),
            d.lease_time_secs,
            d.subnet_mask.parse::<Ipv4Addr>().unwrap_or(Ipv4Addr::new(255, 255, 255, 0)),
        )
    };

    if !enabled {
        println!("DHCP Server is disabled in config.");
        return Ok(());
    }

    // Bind to port 67 (DHCP Server)
    // Note: Kita bind ke 0.0.0.0 agar bisa menerima broadcast
    let addr = SocketAddr::from(([0, 0, 0, 0], 67));
    let socket = match std::net::UdpSocket::bind(addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to bind DHCP socket: {}", e);
            return Err(e.into());
        }
    };
    socket.set_broadcast(true)?;
    socket.set_nonblocking(true)?;
    let socket = tokio::net::UdpSocket::from_std(socket)?;

    println!("Native DHCP Server listening on {} (interface: {})", addr, interface);

    let mut buf = [0u8; 1500];
    loop {
        let (size, _src) = match socket.recv_from(&mut buf).await {
            Ok(res) => res,
            Err(_) => continue,
        };

        let packet_data = &buf[..size];
        if let Some(packet) = DhcpPacket::decode(packet_data) {
            // Hanya proses BootRequest (OP=1)
            if packet.op != 1 { continue; }

            // Extract Message Type
            let msg_type = packet.get_option(53).and_then(|v| v.first()).cloned();
            
            match msg_type {
                Some(1) => { // DHCPDISCOVER
                    // println!("DHCP DISCOVER from {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", 
                    //        packet.chaddr[0], packet.chaddr[1], packet.chaddr[2], packet.chaddr[3], packet.chaddr[4], packet.chaddr[5]);
                    
                    // Offer an IP
                    let offered_ip = find_available_ip(&lease_pool, range_start, range_end).await;
                    if let Some(ip) = offered_ip {
                        let response = packet.build_response(2, ip, 2, gateway, &dns_servers, lease_time, subnet_mask);
                        let dest = SocketAddr::from(([255, 255, 255, 255], 68));
                        let _ = socket.send_to(&response, dest).await;
                    }
                },
                Some(3) => { // DHCPREQUEST
                    let requested_ip = packet.get_option(50).and_then(|v| {
                        if v.len() == 4 {
                            Some(Ipv4Addr::new(v[0], v[1], v[2], v[3]))
                        } else { None }
                    });

                    if let Some(ip) = requested_ip {
                        // Confirm Lease
                        let mut pool = lease_pool.leases.lock().await;
                        let mut mac = [0u8; 6];
                        mac.copy_from_slice(&packet.chaddr[..6]);

                        let lease = DhcpLease {
                            mac,
                            ip,
                            hostname: None, 
                            expires: SystemTime::now() + Duration::from_secs(lease_time as u64),
                        };
                        let expires_at = lease.expires.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
                        pool.insert(mac, lease);

                        // Save to Database for Persistence
                        let mac_str = format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", 
                                             mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
                        let _ = db.save_dhcp_lease(&mac_str, &ip.to_string(), None, expires_at).await;

                        // Send ACK
                        let response = packet.build_response(2, ip, 5, gateway, &dns_servers, lease_time, subnet_mask);
                        let dest = SocketAddr::from(([255, 255, 255, 255], 68));
                        let _ = socket.send_to(&response, dest).await;
                        println!("DHCP ACK: assigned {} to {}", ip, mac_str);
                    }
                },
                _ => {}
            }
        }
    }
}

async fn find_available_ip(pool: &SharedLeasePool, start: Ipv4Addr, end: Ipv4Addr) -> Option<Ipv4Addr> {
    let leases = pool.leases.lock().await;
    let start_u32 = u32::from(start);
    let end_u32 = u32::from(end);

    for ip_u32 in start_u32..=end_u32 {
        let candidate = Ipv4Addr::from(ip_u32);
        let mut used = false;
        for lease in leases.values() {
            if lease.ip == candidate && SystemTime::now() < lease.expires {
                used = true;
                break;
            }
        }
        if !used {
            return Some(candidate);
        }
    }
    None
}

struct DhcpPacket {
    op: u8,
    xid: [u8; 4],
    chaddr: [u8; 16],
    options: HashMap<u8, Vec<u8>>,
}

impl DhcpPacket {
    fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 240 { return None; }
        
        let op = data[0];
        let mut xid = [0u8; 4];
        xid.copy_from_slice(&data[4..8]);
        let mut chaddr = [0u8; 16];
        chaddr.copy_from_slice(&data[28..44]);

        // Magic Cookie
        if &data[236..240] != &[99, 130, 83, 99] { return None; }

        let mut options = HashMap::new();
        let mut pos = 240;
        while pos < data.len() {
            let tag = data[pos];
            if tag == 255 { break; }
            if tag == 0 { pos += 1; continue; }
            
            if pos + 1 >= data.len() { break; }
            let len = data[pos + 1] as usize;
            if pos + 2 + len > data.len() { break; }
            
            let val = data[pos + 2 .. pos + 2 + len].to_vec();
            options.insert(tag, val);
            pos += 2 + len;
        }

        Some(DhcpPacket { op, xid, chaddr, options })
    }

    fn get_option(&self, tag: u8) -> Option<&Vec<u8>> {
        self.options.get(&tag)
    }

    fn build_response(
        &self, 
        op: u8, 
        yiaddr: Ipv4Addr, 
        msg_type: u8, 
        gateway: Ipv4Addr, 
        dns: &[Ipv4Addr],
        lease_time: u32,
        subnet_mask: Ipv4Addr
    ) -> Vec<u8> {
        let mut resp = vec![0u8; 236];
        resp[0] = op;
        resp[1] = 1; // Ethernet
        resp[2] = 6; // MAC Length
        resp[4..8].copy_from_slice(&self.xid);
        resp[16..20].copy_from_slice(&yiaddr.octets());
        resp[28..44].copy_from_slice(&self.chaddr);

        // Magic Cookie
        resp.extend_from_slice(&[99, 130, 83, 99]);

        // Option 53: Message Type
        resp.push(53); resp.push(1); resp.push(msg_type);

        // Option 1: Subnet Mask
        resp.push(1); resp.push(4); resp.extend_from_slice(&subnet_mask.octets());

        // Option 3: Router
        resp.push(3); resp.push(4); resp.extend_from_slice(&gateway.octets());

        // Option 51: Lease Time
        resp.push(51); resp.push(4); resp.extend_from_slice(&lease_time.to_be_bytes());

        // Option 54: Server Identifier (Our IP - assuming gateway IP for now)
        resp.push(54); resp.push(4); resp.extend_from_slice(&gateway.octets());

        // Option 6: DNS Servers
        if !dns.is_empty() {
            resp.push(6);
            resp.push((dns.len() * 4) as u8);
            for d in dns {
                resp.extend_from_slice(&d.octets());
            }
        }

        // End
        resp.push(255);

        resp
    }
}
