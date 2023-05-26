use std::{
    net::{IpAddr, Ipv4Addr},
    process::{self, Stdio},
    time::Duration, sync::{atomic::{AtomicBool, Ordering}, Arc, LazyLock},
};

use pnet::packet::{
    arp::{ArpHardwareTypes, ArpOperations, MutableArpPacket, ArpPacket},
    ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket},
    MutablePacket, Packet,
};
use pnet_datalink::{DataLinkSender, MacAddr, NetworkInterface};
use regex::Regex;
use tokio::process::Command;

pub async fn arp_scan(target_ip: Ipv4Addr) -> std::io::Result<(Ipv4Addr, MacAddr)> {
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
    let mut join_handle = Vec::new();
    let timed_out = Arc::new(AtomicBool::new(false));
    for selected_interface in selected_interfaces {
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
        let cloned_timed_out = Arc::clone(&timed_out);
        let recv_thread = tokio::task::spawn_blocking(move || {
            loop {
                if cloned_timed_out.load(Ordering::Relaxed) {
                    println!("scan {select_interface_ip} -> {target_ip} 超时...");
                    return None;
                }
                let arp_buffer = match rx.next() {
                    Ok(buffer) => buffer,
                    Err(error) => {
                        match error.kind() {
                            // The 'next' call will only block the thread for a given
                            // amount of microseconds. The goal is to avoid long blocks
                            // due to the lack of packets received.
                            std::io::ErrorKind::TimedOut => continue,
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
                            continue;
                        }
                        
                        return Some((sender_ipv4, sender_mac));
                    }
                    continue;
                }
            }
        });
        send_arp_packet(tx, &selected_interface, target_ip)?;
        join_handle.push(recv_thread);
    }
    tokio::time::sleep(Duration::from_secs(5)).await;
    timed_out.store(true, Ordering::Relaxed);
    for handle in join_handle {
        if let Ok(Some(device)) = handle.await {
            return Ok(device)
        }
    }
    Err(std::io::ErrorKind::NotFound.into())
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

static IP_MATCH: LazyLock<Regex> = LazyLock::new(|| Regex::new("\\?\\s\\(([\\d\\.]+)\\)").unwrap());
static MAC_MATCH: LazyLock<Regex> = LazyLock::new(|| Regex::new("at\\s([a-z0-9:]+)").unwrap());

pub async fn linux_arp_scan(target_ip: Ipv4Addr) -> std::io::Result<(Ipv4Addr, MacAddr)> {
    let output = Command::new("su")
        .arg("root")
        .args(["arp", "-a", &target_ip.to_string()])
        .stdout(Stdio::null())
        .output()
        .await?;
    let result = String::from_utf8_lossy(&output.stdout);
    let ip_caps = IP_MATCH.captures(&result);
    let mac_caps = MAC_MATCH.captures(&result);
    let err = Err(std::io::ErrorKind::NotFound.into());
    let (Some(ip_cap), Some(mac_cap)) = (ip_caps, mac_caps) else {return err};
    let (Some(ip), Some(mac)) = (ip_cap.get(1).map(|ip| ip.as_str()), mac_cap.get(1).map(|mac| mac.as_str())) else {return err};
    Ok((ip.parse().unwrap(), mac.parse().unwrap()))
}
