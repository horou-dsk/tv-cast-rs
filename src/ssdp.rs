use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
    sync::{Arc, LazyLock},
};

use rand::Rng;
use socket2::SockAddr;
use tokio::{net::UdpSocket, sync::RwLock};

use crate::{
    constant::{SERVER_PORT, SSDP_ADDR, SSDP_PORT},
    header::parse_header,
    setting,
};

pub static ALLOW_IP: LazyLock<RwLock<Vec<Ipv4Addr>>> = LazyLock::new(|| RwLock::new(vec![]));

pub struct SSDPServer<'a> {
    udp_socket: Arc<UdpSocket>,
    known: HashMap<String, HashMap<&'a str, String>>,
    ssdp_ip: Ipv4Addr,
    sock_addr: SockAddr,
    ip_list: Vec<(Ipv4Addr, Ipv4Addr)>,
    send_socket: HashMap<Ipv4Addr, socket2::Socket>,
}

impl<'a> SSDPServer<'a> {
    pub fn new(udp_socket: Arc<UdpSocket>) -> Self {
        let ssdp_addr = format!("{}:{}", SSDP_ADDR, SSDP_PORT)
            .parse::<SocketAddr>()
            .unwrap();
        Self {
            ssdp_ip: SSDP_ADDR.parse().unwrap(),
            udp_socket,
            known: HashMap::new(),
            // ip_addr: SSDP_ADDR.parse::<Ipv4Addr>().unwrap(),
            ip_list: Vec::new(),
            sock_addr: ssdp_addr.into(),
            send_socket: HashMap::new(),
        }
    }

    fn new_socket(&self, ip: &Ipv4Addr) -> socket2::Socket {
        let skt = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None).unwrap();
        skt.join_multicast_v4(&self.ssdp_ip, ip).unwrap();
        skt.set_multicast_if_v4(ip).unwrap();
        skt
    }

    fn send_to(&self, buf: &[u8], addr: &SockAddr, ip: &Ipv4Addr) {
        // let udp_socket =
        //     socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None).unwrap();
        // if let Err(err) = udp_socket.join_multicast_v4(&SSDP_ADDR.parse().unwrap(), ip) {
        //     eprintln!("notify join multicast error = {err:?}");
        //     return;
        // }
        let Some(skt) = self.send_socket.get(ip) else {
            return;
        };
        if let Err(err) = skt.send_to(buf, addr) {
            log::error!("send to error = {:?}, interface ip = \n{}", err, ip);
        }
    }

    pub fn sync_ip_list(&mut self) {
        let ip_list = setting::get_ip().unwrap();
        let ip_list = ip_list.into_iter().map(|v| (v.0, v.1)).collect();
        if ip_list != self.ip_list {
            for ip in &self.ip_list {
                if let Err(err) = self.udp_socket.leave_multicast_v4(self.ssdp_ip, ip.0) {
                    log::error!("leave_multicast_v4 error = {err:?} ip = {}", ip.0);
                }
            }
            self.send_socket.clear();
            self.ip_list = ip_list;
            for ip in &self.ip_list {
                if let Err(err) = self.udp_socket.join_multicast_v4(self.ssdp_ip, ip.0) {
                    log::error!("join_multicast_v4 error = {err:?} ip = {}", ip.0);
                }
                self.send_socket.insert(ip.0, self.new_socket(&ip.0));
            }
        }
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

    pub async fn do_notify(&self, usn: &'a str) {
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
                .chain(["".to_string(), "".to_string()])
                .map(|v| format!("{v}\r\n"))
                .collect::<String>();
            if ALLOW_IP.read().await.is_empty() {
                return;
            }
            for (ip, _) in &self.ip_list {
                for _ in 0..2 {
                    self.send_to(
                        resp.replace("{local_ip}", &ip.to_string()).as_bytes(),
                        &self.sock_addr,
                        ip,
                    );
                }
            }
        }
    }

    pub async fn do_search(&self) {
        self.udp_socket
            .send_to(
                br#"M-SEARCH * HTTP/1.1
MX: 15
MAN: "ssdp:discover"
HOST: 239.255.255.250:1900
ST: urn:schemas-upnp-org:device:MediaRenderer:1


"#,
                self.sock_addr.as_socket().unwrap(),
            )
            .await
            .unwrap();
    }

    pub async fn do_byebye(&self, usn: &'a str) {
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
                .chain(["".to_string(), "".to_string()])
                .map(|v| format!("{v}\r\n"))
                .collect::<String>();

            self.udp_socket
                .send_to(resp.as_bytes(), self.sock_addr.as_socket().unwrap())
                .await
                .expect("send error");
        }
    }

    pub async fn datagram_received(&self, data: &[u8], src: SocketAddr) {
        let result = String::from_utf8_lossy(data);
        // if result.starts_with("M-SEARCH") {
        //     println!("M-SEARCH * Result = \n{} from ip = {}", result, src);
        // }
        let Some((method, headers)) = parse_header(&result) else {
            println!("Error Result = {}", result);
            return;
        };
        if method[0] == "NOTIFY" {
            // println!("NOTIFY Result = \n{}", result);
            // println!("SSDP command {} {} - from {}", method[0], method[1], src);
        }
        if method[0] == "M-SEARCH" && method[1] == "*" {
            // println!("M-SEARCH *");
            // println!("M-SEARCH * Result = \n{} from ip = {}", result, src);
            // self.discovery_request(headers, src);
            match src.ip() {
                IpAddr::V4(ipv4) => {
                    if ALLOW_IP.read().await.contains(&ipv4) {
                        self.discovery_request(headers, src);
                    }
                }
                IpAddr::V6(_) => (),
            }
        } else if method[0] == "NOTIFY" && method[1] == "*" {
        } else {
            println!("result = \n{}", result);
            println!("Unknown SSDP command {:?}", method);
        }
    }

    pub fn discovery_request(&self, headers: HashMap<String, &str>, src: SocketAddr) {
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
                            if get_subnet_ip(ip, netmask) == get_subnet_ip(host, netmask) {
                                // let response = response.replace("{ip}", &ip.to_string());
                                let response = response.replace("{local_ip}", &ip.to_string());
                                // println!("send to host = {}", addr);
                                // println!("send to = {} \n to host = {}", response, addr);
                                self.send_to(response.as_bytes(), &SockAddr::from(addr), ip);
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
    pub fn new(udp_socket: Arc<UdpSocket>, path: &Path) -> std::io::Result<Self> {
        let mut f = File::options()
            .write(true)
            .read(true)
            .create(true)
            .open(path.join("tp_uuid.txt"))?;
        let mut usn = String::new();
        let r = f.read_to_string(&mut usn);

        if r.is_err() || usn.is_empty() {
            usn = uuid::Uuid::new_v4().to_string();
            f.write_all(usn.as_bytes())?;
        }
        // let usn = uuid::Uuid::new_v4().to_string();
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

        Ok(Self {
            usn,
            devices,
            server: Arc::new(RwLock::new(SSDPServer::new(udp_socket))),
        })
    }

    pub async fn register(self) -> Ssdp<'a> {
        for device in &self.devices {
            let st = if device.len() <= 43 {
                device.clone()
            } else {
                device[43..].to_string()
            };
            self.server.write().await.register(
                device,
                st,
                format!("http://{{local_ip}}:{}/description.xml", SERVER_PORT),
                Some("Linux/4.9.113 HTTP/1.0".to_string()),
                Some("max-age=66".to_string()),
            );
        }
        self
    }

    pub async fn do_notify(&self) {
        for device in &self.devices {
            self.server.read().await.do_notify(device).await;
        }
    }
}

pub fn get_subnet_ip(ip: &Ipv4Addr, netmask: &Ipv4Addr) -> Ipv4Addr {
    let ip = ip
        .octets()
        .into_iter()
        .zip(netmask.octets())
        .map(|(a, b)| a & b)
        .collect::<Vec<u8>>();
    Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])
}
