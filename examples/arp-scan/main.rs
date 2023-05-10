use std::{
    net::{IpAddr, Ipv4Addr},
    process,
    thread,
    time::Duration,
};

use pnet::packet::{
    arp::{ArpHardwareTypes, ArpOperations, MutableArpPacket, ArpPacket},
    ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket},
    MutablePacket, Packet,
};
use pnet_datalink::{DataLinkSender, MacAddr, NetworkInterface};

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    args.next();
    let target_ip = args.next().map(|v| v.parse().expect("ip地址格式错误")).expect("请输入目标Ip地址");
    let interfaces = pnet_datalink::interfaces();
    let selected_interfaces: Vec<NetworkInterface> = interfaces
        .into_iter()
        .filter(|interface| {
            interface.mac.is_some()
                && !interface.ips.is_empty()
                && interface.is_up()
                && !interface.is_loopback()
        })
        .collect();
    for selected_interface in selected_interfaces {
        // println!("{:?}", selected_interface.ips);
        let IpAddr::V4(select_interface_ip) = selected_interface.ips
            .iter()
            .find(|v| v.is_ipv4())
            .unwrap()
            .ip() else {continue};
        let channel_config = pnet_datalink::Config {
            read_timeout: Some(Duration::from_millis(500)),
            ..pnet_datalink::Config::default()
        };
        let pnet_datalink::Channel::Ethernet(tx, mut rx) = pnet_datalink::channel(&selected_interface, channel_config)? else { 
            eprintln!("Datalink channel creation failed ");
            process::exit(1);
        };
        let recv_thread = thread::spawn(move || {
            let mut timeout_num = 0;
            loop {
                if timeout_num >= 3 {
                    println!("scan {select_interface_ip} -> {target_ip} 超时...");
                    break;
                }
                let arp_buffer = match rx.next() {
                    Ok(buffer) => buffer,
                    Err(error) => {
                        match error.kind() {
                            // The 'next' call will only block the thread for a given
                            // amount of microseconds. The goal is to avoid long blocks
                            // due to the lack of packets received.
                            std::io::ErrorKind::TimedOut => {
                                timeout_num += 1;
                                continue;
                            },
                            _ => {
                                eprintln!("Failed to receive ARP requests ({})", error);
                                process::exit(1);
                            }
                        };
                    }
                };
                let ethernet_packet = match EthernetPacket::new(arp_buffer) {
                    Some(packet) => packet,
                    None => continue,
                };
                let is_arp_type = matches!(ethernet_packet.get_ethertype(), EtherTypes::Arp);
                if is_arp_type {
                    let arp_packet = ArpPacket::new(&arp_buffer[MutableEthernetPacket::minimum_packet_size()..]);
                    if let Some(arp) = arp_packet {
    
                        let sender_ipv4 = arp.get_sender_proto_addr();
                        let sender_mac = arp.get_sender_hw_addr();

                        if sender_ipv4 != target_ip {
                            timeout_num += 1;
                            continue;
                        }
    
                        println!("sender_ipv4 = {sender_ipv4}");
                        println!("sender_mac = {sender_mac}");
                    }
                    break;
                }
            }
        });
        send_arp_packet(tx, &selected_interface, target_ip)?;
        recv_thread.join().unwrap();
    }
    
    Ok(())
}

fn send_arp_packet(
    mut tx: Box<dyn DataLinkSender>,
    interface: &NetworkInterface,
    target_ip: Ipv4Addr,
) -> std::io::Result<()> {
    let mut ethernet_buffer = [0u8; 42];

    let mut ethernet_packet =
        MutableEthernetPacket::new(&mut ethernet_buffer).unwrap_or_else(|| {
            eprintln!("Could not build Ethernet packet");
            process::exit(1);
        });
    let target_mac = MacAddr::broadcast();
    let potential_network = interface.ips.iter().find(|network| network.is_ipv4());
    let source_ip = match potential_network.map(|network| network.ip()) {
        Some(IpAddr::V4(ipv4_addr)) => ipv4_addr,
        _ => {
            eprintln!("Expected IPv4 address on network interface");
            process::exit(1);
        }
    };
    let source_mac = interface.mac.unwrap_or_else(|| {
        eprintln!("Interface should have a MAC address");
        process::exit(1);
    });

    ethernet_packet.set_destination(target_mac);
    ethernet_packet.set_source(source_mac);

    let selected_ethertype = EtherTypes::Arp;

    ethernet_packet.set_ethertype(selected_ethertype);

    let mut arp_buffer = [0u8; 28];

    let mut arp_packet = MutableArpPacket::new(&mut arp_buffer).unwrap_or_else(|| {
        eprintln!("Could not build ARP packet");
        process::exit(1);
    });

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_operation(ArpOperations::Request);
    arp_packet.set_sender_hw_addr(source_mac);
    arp_packet.set_sender_proto_addr(source_ip);
    arp_packet.set_target_hw_addr(target_mac);
    arp_packet.set_target_proto_addr(target_ip);

    ethernet_packet.set_payload(arp_packet.packet_mut());

    // println!("source_ip = {}", source_ip);

    tx.send_to(
        ethernet_packet.to_immutable().packet(),
        Some(interface.clone()),
    )
    .unwrap()?;
    // skt.send_to(
    //     ethernet_packet.to_immutable().packet(),
    //     &SockAddr::from(SocketAddr::new(IpAddr::V4(source_ip), 2054)),
    // )?;

    Ok(())
}
