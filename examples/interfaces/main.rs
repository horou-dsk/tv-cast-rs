fn main() {
    let interfaces = default_net::get_interfaces();
    for interface in interfaces {
        println!(
            "ip = {:?} mac = {}",
            interface.ipv4,
            interface.mac_addr.map(|mac| mac.to_string()).unwrap()
        );
    }
}
