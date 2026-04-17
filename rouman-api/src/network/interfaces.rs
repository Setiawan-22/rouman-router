use rtnetlink::new_connection;
use futures::stream::TryStreamExt;
use serde::Serialize;
use netlink_packet_route::link::LinkAttribute;

#[derive(Serialize)]
pub struct NetworkInterface {
    pub index: u32,
    pub name: String,
    pub mac: String,
    pub mtu: u32,
    pub is_up: bool,
    pub oper_state: String,
}

pub async fn get_all_interfaces() -> Result<Vec<NetworkInterface>, String> {
    let (connection, handle, _) = new_connection().map_err(|e| format!("Netlink Error: {}", e))?;
    tokio::spawn(connection);

    let mut links = handle.link().get().execute();
    let mut result = Vec::new();

    while let Ok(Some(link)) = links.try_next().await {
        let mut iface_name = String::new();
        let mut mac = String::new();
        let mut mtu = 0;
        let mut oper_state = String::new();
        
        let index = link.header.index;
        // Gunakan string representation untuk mengecek BitFlags (Aman dari API break)
        let is_up = format!("{:?}", link.header.flags).to_uppercase().contains("UP");

        for nla in link.attributes.into_iter() {
            match nla {
                LinkAttribute::IfName(name) => iface_name = name,
                LinkAttribute::Address(addr) => {
                    mac = addr.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(":");
                },
                LinkAttribute::Mtu(m) => mtu = m,
                LinkAttribute::OperState(state) => {
                    oper_state = format!("{:?}", state).to_uppercase();
                },
                _ => {}
            }
        }

        result.push(NetworkInterface {
            index,
            name: iface_name,
            mac,
            mtu,
            is_up,
            oper_state,
        });
    }

    Ok(result)
}
