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
    let sock_addr = format!("0.0.0.0:{}", SSDP_PORT)
        .parse::<SocketAddr>()
        .unwrap()
        .into();
    let udp_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    udp_socket.set_reuse_address(true)?;
    // udp_socket.set_multicast_loop_v4(false)?;
    // let ip = IpAddr::V4(Ipv4Addr::LOCALHOST); // local_ip_address::local_ip().unwrap();
    // println!("local_ip = {ip}");
    // let local_addr = SocketAddr::new(ip, 15642);
    // udp_socket.bind(&SockAddr::from(local_addr))?;
    udp_socket.bind(&sock_addr)?;
    // let udp_socket = UdpSocket::bind("0.0.0.0:1900")?;
    // udp_socket.set_multicast_loop_v4(false)?;
    let ip_list = setting::get_ip().unwrap();
    for local_ip in &ip_list {
        println!("local_ip = {local_ip:?}");
        if let Err(err) = udp_socket.join_multicast_v4(&ip_addr, &local_ip.0) {
            println!("1.join_multicast_v4 error = {:?}", err);
        }
    }

    // if let Err(err) = udp_socket.join_multicast_v4(&ip_addr, &"192.168.137.1".parse().unwrap()) {
    //     println!("join_multicast_v4 error = {:?}", err);
    // }

    let udp_socket = Arc::new(udp_socket);

    {
        let _socket = udp_socket.clone();
        thread::spawn(move || loop {
            let mut buf = [MaybeUninit::uninit(); 1024];
            let (size, src) = udp_socket.recv_from(&mut buf).unwrap();
            let buf = unsafe { MaybeUninit::slice_assume_init_ref(&buf[..size]) };
            let result = String::from_utf8_lossy(buf);
            // println!(
            //     "NOTIFY ===========Result = \n{} from ip = {:?}",
            //     result,
            //     src.as_socket()
            // );
            if result.starts_with("NOTIFY") && result.contains("41937") {
                println!(
                    "NOTIFY ===========Result = \n{} from ip = {:?}",
                    result,
                    src.as_socket()
                );
            }
            if !result.starts_with("NOTIFY") && !result.starts_with("M-SEARCH")
            // && result.contains("41937")
            {
                println!(
                    "M-SEARCH ===========Result = \n{} from ip = {:?}",
                    result,
                    src.as_socket()
                );
            }
        });
        let t = thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(3));
            let _buf = br#"M-SEARCH * HTTP/1.1
MX: 15
MAN: "ssdp:discover"
HOST: 239.255.255.250:1900
ST: urn:schemas-upnp-org:device:MediaRenderer:1

"#;
            // println!("{}", String::from_utf8_lossy(buf));
            // socket.send_to(buf, &sock_addr).unwrap();
            // println!("send ok");
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
