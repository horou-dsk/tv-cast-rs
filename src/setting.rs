use std::net::Ipv4Addr;

pub fn get_ip() -> Result<Vec<(Ipv4Addr, Ipv4Addr)>, String> {
    if cfg!(windows) {
        let default_interface = default_net::get_default_interface()?;
        Ok(default_interface
            .ipv4
            .into_iter()
            .map(|ip| (ip.addr, ip.netmask))
            .collect())
    } else {
        let mut ip_list = Vec::new();
        let interfaces = default_net::get_interfaces();
        for interface in interfaces {
            if interface.if_type == default_net::interface::InterfaceType::Ethernet {
                for ip in interface.ipv4 {
                    ip_list.push((ip.addr, ip.netmask));
                }
            }
        }
        Ok(ip_list)
    }
    // if let Ok(interface) = default_net::get_default_interface() {
    //     let mut ip = Ipv4Addr::LOCALHOST;
    //     let mut netmask = Ipv4Addr::new(255, 255, 255, 0);
    //     for ipv4 in interface.ipv4 {
    //         ip = ipv4.addr;
    //         netmask = ipv4.netmask;
    //     }
    //     println!("inteface \nip = {}\nnetmask = {}", ip, netmask);
    //     (ip, netmask)
    // } else {
    //     panic!("get default interface Error");
    // }
    // match (
    //     default_net::get_default_gateway(),
    //     default_net::get_default_interface(),
    // ) {
    //     (Ok(gateway), Ok(interface)) => {
    //         println!("Default Gateway");
    //         let mut ip = Ipv4Addr::LOCALHOST;
    //         if let IpAddr::V4(v4) = gateway.ip_addr {
    //             ip = v4;
    //         }
    //         let mut netmask = Ipv4Addr::new(255, 255, 255, 0);
    //         // gateway.ip_addr
    //         println!("\tMAC: {}", gateway.mac_addr);
    //         println!("\tIP: {}", gateway.ip_addr);
    //         for ipv4 in interface.ipv4 {
    //             // println!("{:?}", ipv4);
    //             // if ipv4.addr == gateway.ip_addr {
    //             //     ip = ipv4.addr;
    //             //     netmask = ipv4.netmask;
    //             // }
    //             ip = ipv4.addr;
    //             netmask = ipv4.netmask;
    //         }
    //         (ip, netmask)
    //     }
    //     (Err(e), _) => {
    //         panic!("Get gateway error {}", e);
    //     }
    //     (_, Err(e)) => {
    //         panic!("Get interface error {}", e);
    //     }
    // }
}

// pub fn get_usn() -> String {
//     let dlna_id = uuid::Uuid::new_v4();
// }
