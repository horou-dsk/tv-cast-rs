use std::net::Ipv4Addr;

pub fn get_ip() -> (Ipv4Addr, Ipv4Addr) {
    if let Ok(interface) = default_net::get_default_interface() {
        let mut ip = Ipv4Addr::LOCALHOST;
        let mut netmask = Ipv4Addr::new(255, 255, 255, 0);
        for ipv4 in interface.ipv4 {
            ip = ipv4.addr;
            netmask = ipv4.netmask;
        }
        println!("inteface \nip = {}\nnetmask={}", ip, netmask);
        (ip, netmask)
    } else {
        panic!("get default interface Error");
    }
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
