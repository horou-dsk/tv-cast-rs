use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{Arc, LazyLock, RwLock},
};

use rand::Rng;
use socket2::{SockAddr, Socket};

use crate::{
    constant::{SSDP_ADDR, SSDP_PORT},
    header::parse_header,
};

pub static mut ALLOW_IP: LazyLock<RwLock<Vec<Ipv4Addr>>> =
    LazyLock::new(|| RwLock::new(Vec::new()));

#[derive(Clone)]
pub struct SSDPServer<'a> {
    udp_socket: Arc<Socket>,
    known: HashMap<String, HashMap<&'a str, String>>,
    // ip_addr: Ipv4Addr,
    sock_addr: SockAddr,
    ip_list: Vec<(Ipv4Addr, Ipv4Addr)>,
    send_socket: Arc<UdpSocket>,
}

impl<'a> SSDPServer<'a> {
    pub fn new(udp_socket: Arc<Socket>, local_ip: Ipv4Addr) -> Self {
        Self {
            udp_socket,
            known: HashMap::new(),
            // ip_addr: SSDP_ADDR.parse::<Ipv4Addr>().unwrap(),
            ip_list: Vec::new(),
            sock_addr: format!("{}:{}", SSDP_ADDR, SSDP_PORT)
                .parse::<SocketAddr>()
                .unwrap()
                .into(),
            send_socket: Arc::new(UdpSocket::bind((local_ip, 19565)).unwrap()),
        }
    }

    pub fn add_ip_list(&mut self, ip: (Ipv4Addr, Ipv4Addr)) {
        self.ip_list.push(ip);
    }

    pub fn register(
        &mut self,
        usn: &str,
        st: String,
        location: String,
        server: Option<String>,
        cache_control: Option<String>,
    ) {
        let mut map: HashMap<&str, String> = HashMap::new();
        map.insert("USN", usn.to_string());
        map.insert("LOCATION", location);
        map.insert("ST", st);
        map.insert("EXT", "".to_string());
        map.insert("SERVER", server.unwrap_or("SSDP Server".to_string()));
        map.insert(
            "CACHE-CONTROL",
            cache_control.unwrap_or("max-age=1800".to_string()),
        );
        self.known.insert(usn.to_string(), map);
    }

    pub fn unregister(&mut self, usn: &'a str) {
        self.known.remove(usn);
    }

    pub fn do_notify(&self, usn: &'a str) {
        if let Some(map) = self.known.get(usn) {
            let mut map = map.clone();
            let resp = vec![
                "NOTIFY * HTTP/1.1".to_string(),
                format!("HOST: {}:{}", SSDP_ADDR, SSDP_PORT),
                "NTS: ssdp:alive".to_string(),
                format!("01-NLS: {}", uuid::Uuid::new_v4()),
            ];
            let st = map.remove("ST").unwrap();
            map.insert("NT", st);
            let resp = resp
                .into_iter()
                .chain(map.into_iter().map(|(k, v)| format!("{k}: {v}")))
                .chain(["".to_string(), "".to_string()].into_iter())
                .map(|v| format!("{v}\r\n"))
                .collect::<String>()
                .replace("{ip}", &self.ip_list[0].0.to_string());
            // println!("==============notify = \n{}", resp);
            // for allow_ip in unsafe { &*(ALLOW_IP.read().unwrap()) } {
            //     let allow_addr = SocketAddrV4::new(*allow_ip, SSDP_PORT);
            //     self.udp_socket
            //         .send_to(resp.as_bytes(), &allow_addr.into())
            //         .expect("send error");
            // }
            self.udp_socket
                .send_to(resp.as_bytes(), &self.sock_addr)
                .expect("send error");
        }
    }

    pub fn do_search(&self) {
        self.udp_socket
            .send_to(
                br#"M-SEARCH * HTTP/1.1
MX: 15
MAN: "ssdp:discover"
HOST: 239.255.255.250:1900
ST: urn:schemas-upnp-org:device:MediaRenderer:1


"#,
                &self.sock_addr,
            )
            .unwrap();
    }

    pub fn do_byebye(&self, usn: &'a str) {
        if let Some(map) = self.known.get(usn) {
            let mut map = map.clone();
            let st = map.remove("ST").unwrap();
            map.insert("NT", st);
            let resp = vec![
                "NOTIFY * HTTP/1.1".to_string(),
                format!("HOST: {}:{}", SSDP_ADDR, SSDP_PORT),
                "NTS: ssdp:byebye".to_string(),
            ];
            let resp = resp
                .into_iter()
                .chain(map.into_iter().map(|(k, v)| format!("{k}: {v}")))
                .chain(["".to_string(), "".to_string()].into_iter())
                .map(|v| format!("{v}\r\n"))
                .collect::<String>();

            self.udp_socket
                .send_to(resp.as_bytes(), &self.sock_addr)
                .expect("send error");
        }
    }

    pub fn datagram_received(&self, data: &[u8], src: SocketAddr) {
        let result = String::from_utf8_lossy(data);
        let (method, headers) = parse_header(&result);
        if method[0] == "NOTIFY" {
            // println!("result = \n{}", result);
            // println!("SSDP command {} {} - from {}", method[0], method[1], src);
        }
        if method[0] == "M-SEARCH" && method[1] == "*" {
            // println!("M-SEARCH *");
            // println!("M-SEARCH * Result = \n{} from ip = {}", result, src);
            match src.ip() {
                IpAddr::V4(ipv4) => {
                    if unsafe { ALLOW_IP.read().unwrap().contains(&ipv4) } {
                        self.discovery_request(headers, src);
                    }
                }
                IpAddr::V6(_) => (),
            }
            // unimplemented!()
        } else if method[0] == "NOTIFY" && method[1] == "*" {
        } else {
            println!("result = \n{}", result);
            println!("Unknown SSDP command {} {}", method[0], method[1]);
        }
    }

    pub fn discovery_request(&self, headers: HashMap<String, String>, src: SocketAddr) {
        if let SocketAddr::V4(addr) = src {
            for v in self.known.values() {
                if v["ST"] == headers["st"] || headers["st"] == "ssdp:all" {
                    let mut response = vec!["HTTP/1.1 200 OK".to_string()];
                    let mut usn = None;
                    for (k, v) in v {
                        if *k == "USN" {
                            usn = Some(k);
                        }
                        response.push(format!("{k}: {v}"));
                    }
                    if usn.is_some() {
                        response.push(format!("DATE: {}", chrono::Local::now().to_rfc2822()));
                        response
                            .push("OPT: \"http://schemas.upnp.org/upnp/1/0/\"; ns=01".to_string());
                        response.push(format!(
                            "01-NLS: {}",
                            chrono::Local::now().timestamp_nanos()
                        ));
                        response.push("".to_string());
                        response.push("".to_string());
                        let response = response.join("\r\n");
                        let mx = headers["mx"].parse::<i32>().unwrap();
                        let _delay = rand::thread_rng().gen_range(0..mx);
                        let (host, _port) = (addr.ip(), addr.port());

                        for (ip, netmask) in &self.ip_list {
                            if get_subnet_ip(*ip, *netmask) == get_subnet_ip(*host, *netmask) {
                                let response = response.replace("{ip}", &ip.to_string());
                                // println!("send to = {} \n to host = {}", response, addr);
                                self.send_socket.send_to(response.as_bytes(), addr).unwrap();
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Ssdp<'a> {
    pub usn: String,
    devices: Vec<String>,
    pub server: Arc<RwLock<SSDPServer<'a>>>,
}

impl<'a> Ssdp<'a> {
    pub fn new(udp_socket: Arc<Socket>, local_ip: Ipv4Addr) -> Self {
        let mut f = File::options()
            .write(true)
            .read(true)
            .create(true)
            .open("./hztp_uuid.txt")
            .unwrap();
        let mut usn = String::new();
        let r = f.read_to_string(&mut usn);

        if r.is_err() || usn.is_empty() {
            usn = uuid::Uuid::new_v4().to_string();
            f.write_all(usn.as_bytes()).unwrap();
        }

        println!("\nuuid = {}\n", usn);

        // let usn = uuid::Uuid::new_v4().to_string();
        let devices = vec![
            format!("uuid:{usn}::upnp:rootdevice"),
            format!("uuid:{usn}"),
            format!("uuid:{usn}::urn:schemas-upnp-org:device:MediaRenderer:1"),
            format!("uuid:{usn}::urn:schemas-upnp-org:service:RenderingControl:1"),
            format!("uuid:{usn}::urn:schemas-upnp-org:service:ConnectionManager:1"),
            format!("uuid:{usn}::urn:schemas-upnp-org:service:AVTransport:1"),
        ];

        Self {
            usn,
            devices,
            server: Arc::new(RwLock::new(SSDPServer::new(udp_socket, local_ip))),
        }
    }

    pub fn register(self) -> Self {
        for device in &self.devices {
            let st = if device.len() <= 43 {
                device.clone()
            } else {
                device[43..].to_string()
            };
            self.server.write().unwrap().register(
                device,
                st,
                format!("http://{{ip}}:{}/description.xml", 8080),
                Some("Linux/4.9.113 HTTP/1.0".to_string()),
                Some("max-age=66".to_string()),
            );
        }
        self
    }

    pub fn do_notify(&self) {
        for device in &self.devices {
            self.server.read().unwrap().do_notify(device);
        }
    }
}

fn get_subnet_ip(ip: Ipv4Addr, netmask: Ipv4Addr) -> Ipv4Addr {
    let ip = ip
        .octets()
        .into_iter()
        .zip(netmask.octets())
        .map(|(a, b)| a & b)
        .collect::<Vec<u8>>();
    Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])
}
