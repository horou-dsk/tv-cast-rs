#![feature(maybe_uninit_slice)]

use std::{
    mem::MaybeUninit,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    process,
    sync::Arc,
    thread,
};

use pnet::packet::{
    arp::{ArpHardwareTypes, ArpOperations, MutableArpPacket},
    ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket},
    MutablePacket, Packet,
};
use pnet_datalink::{MacAddr, NetworkInterface};
use socket2::Domain;

fn main() -> std::io::Result<()> {
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
    let skt = socket2::Socket::new(
        Domain::IPV4,
        socket2::Type::RAW,
        Some(socket2::Protocol::ICMPV4),
    )?;
    let ifc = &selected_interfaces[0];
    skt.bind(&SocketAddr::from((ifc.ips[0].ip(), 0)).into())?;
    let skt = Arc::new(skt);
    let recv_thread = {
        let skt = skt.clone();
        thread::spawn(move || {
            let mut buf = [MaybeUninit::uninit(); 1024];
            loop {
                let (size, src) = skt.recv_from(&mut buf).unwrap();
                let buf = unsafe { MaybeUninit::slice_assume_init_ref(&buf[..size]) };
                println!("数据进入 size = {size}");
                let ethernet_packet = match EthernetPacket::new(buf) {
                    Some(packet) => packet,
                    None => continue,
                };
                let is_arp_type = matches!(ethernet_packet.get_ethertype(), EtherTypes::Arp);
                if is_arp_type {
                    println!("收到一个arp响应 src = {:?}", src.as_socket());
                    break;
                }
            }
        })
    };
    send_arp_packet(skt, ifc)?;
    recv_thread.join().unwrap();
    Ok(())
}

fn send_arp_packet(skt: Arc<socket2::Socket>, interface: &NetworkInterface) -> std::io::Result<()> {
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
    arp_packet.set_target_proto_addr("172.31.64.1".parse::<Ipv4Addr>().unwrap());

    ethernet_packet.set_payload(arp_packet.packet_mut());

    println!("source_ip = {}", source_ip);

    skt.send(ethernet_packet.to_immutable().packet())?;
    // skt.send_to(
    //     ethernet_packet.to_immutable().packet(),
    //     &SockAddr::from(SocketAddr::new(IpAddr::V4(source_ip), 2054)),
    // )?;

    Ok(())
}
