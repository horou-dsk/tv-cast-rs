#![feature(maybe_uninit_slice)]
// use network_interface::{NetworkInterface, NetworkInterfaceConfig};

use std::{
    mem::MaybeUninit,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    thread,
    time::Duration,
};

use hztp::{
    constant::{SSDP_ADDR, SSDP_PORT},
    setting,
};
use socket2::{Domain, Protocol, Socket, Type};

fn main() -> std::io::Result<()> {
    let ip_addr = SSDP_ADDR.parse::<Ipv4Addr>().unwrap();
    let sock_addr = format!("239.255.255.250:{}", SSDP_PORT)
        .parse::<SocketAddr>()
        .unwrap()
        .into();
    let udp_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    udp_socket.set_reuse_address(true)?;
    udp_socket.set_multicast_loop_v4(false)?;
    udp_socket.bind(&"192.169.1.12:15642".parse::<SocketAddr>().unwrap().into())?;
    // let udp_socket = UdpSocket::bind("0.0.0.0:1900")?;
    // udp_socket.set_multicast_loop_v4(false)?;
    let (ip, _netmask) = setting::get_ip();
    if let Err(err) = udp_socket.join_multicast_v4(&ip_addr, &ip) {
        println!("join_multicast_v4 error = {:?}", err);
    }
    // if let Err(err) = udp_socket.join_multicast_v4(&ip_addr, &"192.168.137.1".parse().unwrap()) {
    //     println!("join_multicast_v4 error = {:?}", err);
    // }

    let udp_socket = Arc::new(udp_socket);

    {
        let socket = udp_socket.clone();
        thread::spawn(move || loop {
            let mut buf = [MaybeUninit::uninit(); 1024];
            let (size, src) = udp_socket.recv_from(&mut buf).unwrap();
            let buf = unsafe { MaybeUninit::slice_assume_init_ref(&buf[..size]) };
            let result = String::from_utf8_lossy(buf);
            if !result.starts_with("NOTIFY") && !result.starts_with("M-SEARCH") {
                println!(
                    "===========Result = \n{} from ip = {:?}",
                    result,
                    src.as_socket()
                );
            }
        });
        let t = thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(3));
            let buf = br#"M-SEARCH * HTTP/1.1
MX: 15
MAN: "ssdp:discover"
HOST: 239.255.255.250:1900
ST: urn:schemas-upnp-org:device:MediaRenderer:1

"#;
            // println!("{}", String::from_utf8_lossy(buf));
            socket.send_to(buf, &sock_addr).unwrap();
            println!("send ok");
        });
        t.join().unwrap();
    }

    // let network_interfaces = NetworkInterface::show().unwrap();

    // for itf in network_interfaces.iter() {
    //     println!("{:?}", itf.);
    // }
    // println!("{:#?}", setting::get_ip());
    // for iface in get_if_addrs::get_if_addrs().unwrap() {
    //     println!("{:#?}", iface);
    // }
    Ok(())
}
