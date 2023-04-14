#![feature(maybe_uninit_slice)]

use std::{
    mem::MaybeUninit,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    thread,
    time::Duration,
};

use protocol::DLNAHandler;
use ssdp::Ssdp;

use crate::constant::{SSDP_ADDR, SSDP_PORT};
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

pub fn dlna_init() -> std::io::Result<DLNAHandler> {
    let ip_addr = SSDP_ADDR.parse::<Ipv4Addr>().unwrap();
    let udp_socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    let (ip, netmask) = setting::get_ip();
    let address = format!("{}:{}", SSDP_ADDR, SSDP_PORT)
        .parse::<SocketAddr>()
        .unwrap();
    set_resue_upd(&udp_socket)?;
    udp_socket.set_multicast_loop_v4(false).unwrap();
    let address: SockAddr = address.into();
    udp_socket.bind(&address)?;
    if let Err(err) = udp_socket.join_multicast_v4(&ip_addr, &ip) {
        println!("join_multicast_v4 error = {:?}", err);
    }
    let udp_socket = Arc::new(udp_socket);
    let ssdp = Ssdp::new(udp_socket.clone(), ip).register();
    ssdp.server.write().unwrap().add_ip_list((ip, netmask));
    // if cfg!(windows) {
    //     let win_ip = Ipv4Addr::new(192, 169, 137, 1);
    //     let win_netmask = Ipv4Addr::new(255, 255, 255, 0);
    //     ssdp.server
    //         .write()
    //         .unwrap()
    //         .add_ip_list((win_ip, win_netmask));
    // }
    let dlna = DLNAHandler::new(&ssdp.usn, ip);
    {
        let udp_socket = udp_socket;
        let server = ssdp.server.clone();
        thread::spawn(move || loop {
            let mut buf = [MaybeUninit::uninit(); 1024];
            let (amt, src) = udp_socket.recv_from(&mut buf).expect("recv_from error");
            let buf = unsafe { MaybeUninit::slice_assume_init_ref(&buf[..amt]) };
            server
                .read()
                .unwrap()
                .datagram_received(buf, src.as_socket().unwrap());
        });
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(2));
            ssdp.do_notify();
        });
    }

    Ok(dlna)
}

pub mod actions;
pub mod routers;
