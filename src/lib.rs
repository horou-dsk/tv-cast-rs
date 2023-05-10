#![feature(maybe_uninit_slice)]
#![feature(lazy_cell)]
#![feature(result_option_inspect)]

use std::{
    mem::MaybeUninit,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    thread,
    time::Duration,
};

use futures::{stream::FuturesUnordered, StreamExt};
use protocol::DLNAHandler;
use ssdp::{Ssdp, ALLOW_IP};
use surge_ping::{PingIdentifier, PingSequence};

use crate::{
    constant::{SSDP_ADDR, SSDP_PORT},
    net::arp::arp_scan,
};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

pub mod constant;
pub mod header;
pub mod protocol;
pub mod setting;
pub mod ssdp;

#[cfg(all(
    feature = "all",
    not(any(target_os = "solaris", target_os = "illumos"))
))]
pub fn set_resue_upd(udp_socket: &Socket) -> std::io::Result<()> {
    udp_socket.set_reuse_port(true)
}

#[cfg(all(not(any(target_os = "solaris", target_os = "illumos"))))]
pub fn set_resue_upd(udp_socket: &Socket) -> std::io::Result<()> {
    udp_socket.set_reuse_address(true)
}

pub fn dlna_init(name: String) -> std::io::Result<DLNAHandler> {
    let ip_addr = SSDP_ADDR.parse::<Ipv4Addr>().unwrap();
    let udp_socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    let ip_list = setting::get_ip().unwrap();
    for local_ip in &ip_list {
        println!("local_ip = {local_ip:?}");
        if let Err(err) = udp_socket.join_multicast_v4(&ip_addr, &local_ip.0) {
            println!("1.join_multicast_v4 error = {:?}", err);
        }
    }

    let address = format!("0.0.0.0:{}", SSDP_PORT)
        .parse::<SocketAddr>()
        .unwrap();
    set_resue_upd(&udp_socket)?;
    udp_socket.set_multicast_loop_v4(false)?;
    let address: SockAddr = address.into();
    udp_socket.bind(&address)?;

    let udp_socket = Arc::new(udp_socket);

    let local_ip = ip_list[0];

    let ssdp = Ssdp::new(udp_socket.clone(), local_ip.0).register();
    {
        let mut server = ssdp.server.write().unwrap();
        for local_ip in ip_list {
            server.add_ip_list(local_ip);
        }
    }
    // if cfg!(windows) {
    //     let win_ip = Ipv4Addr::new(192, 169, 137, 1);
    //     let win_netmask = Ipv4Addr::new(255, 255, 255, 0);
    //     ssdp.server
    //         .write()
    //         .unwrap()
    //         .add_ip_list((win_ip, win_netmask));
    // }
    let dlna = DLNAHandler::new(&ssdp.usn, local_ip.0, name);
    {
        // let udp_socket = udp_socket;
        let server = ssdp.server.clone();
        thread::Builder::new()
            .name("ssdp recv".to_string())
            .spawn(move || {
                let mut buf = [MaybeUninit::uninit(); 1024];
                loop {
                    let (amt, src) = udp_socket.recv_from(&mut buf).expect("recv_from error");
                    let buf = unsafe { MaybeUninit::slice_assume_init_ref(&buf[..amt]) };
                    server
                        .read()
                        .unwrap()
                        .datagram_received(buf, src.as_socket().unwrap());
                }
            })?;
        thread::Builder::new()
            .name("ssdp notify".to_string())
            .spawn(move || loop {
                thread::sleep(Duration::from_secs(3));
                ssdp.do_notify();
            })?;
    }

    Ok(dlna)
}

pub async fn ip_online_check() -> std::io::Result<()> {
    if ALLOW_IP.read().unwrap().is_empty() {
        return Ok(());
    }
    // let allow_ip = std::mem::take(ALLOW_IP.write().as_deref_mut().unwrap());
    let allow_ip = { ALLOW_IP.read().unwrap().clone() };
    let config = surge_ping::Config::default();
    let client = surge_ping::Client::new(&config)?;
    let payload = [0; 56];
    let mut remove_ip = Vec::new();
    let mut tasks = FuturesUnordered::new();
    for ip in allow_ip {
        let client = client.clone();
        tasks.push(async move {
            let mut pinger = client
                .pinger(std::net::IpAddr::V4(ip), PingIdentifier(rand::random()))
                .await;
            if pinger.ping(PingSequence(0), &payload).await.is_err() {
                if let Ok((recv_ip, _)) = arp_scan(ip).await {
                    if recv_ip == ip {
                        return (true, ip);
                    }
                }
            } else {
                return (true, ip);
            }
            println!("剔除 device_ip = {ip}");
            (false, ip)
        });
    }
    while let Some((connect, ip)) = tasks.next().await {
        if !connect {
            remove_ip.push(ip);
        }
    }
    let mut allow_ip = ALLOW_IP.write().unwrap();
    allow_ip.retain(|ip| !remove_ip.contains(ip));
    Ok(())
}

pub mod actions;
pub mod net;
pub mod routers;
