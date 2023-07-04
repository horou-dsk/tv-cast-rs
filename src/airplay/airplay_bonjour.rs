use std::net::IpAddr;

use libmdns::{Responder, Service};

use crate::{constant::SERVER_PORT, setting};

const AIRPLAY_SERVICE_TYPE: &str = "_airplay._tcp";

const AIRTUNES_SERVICE_TYPE: &str = "_raop._tcp";

pub struct AirPlayBonjour {
    server_name: String,
    services: Vec<Service>, // mdns_list: Vec<>
}

// fn log_err<T: std::fmt::Debug + Sized>(msg: &'static str) -> Box<dyn FnOnce(&T)> {
//     Box::new(move |err| {
//         log::error!("{msg} {err:?}");
//     })
// }

impl AirPlayBonjour {
    pub fn new(server_name: String) -> Self {
        Self {
            server_name,
            services: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        let port = SERVER_PORT;
        log::info!("mDNS port = {port}");
        let interfaces = setting::get_ip().unwrap();
        let mac = default_net::get_default_interface()
            .unwrap()
            .mac_addr
            .unwrap()
            .to_string();
        let responder =
            Responder::new_with_ip_list(interfaces.into_iter().map(|v| IpAddr::V4(v.0)).collect())
                .unwrap();

        let txt: Vec<String> = Self::airplay_mdns_props(mac.clone())
            .into_iter()
            .map(|v| format!("{}={}", v.0, v.1))
            .collect();
        let txt: Vec<&str> = txt.iter().map(|v| v.as_str()).collect();
        let svc = responder.register(
            AIRPLAY_SERVICE_TYPE.into(),
            self.server_name.clone(),
            port,
            &txt,
        );

        self.services.push(svc);

        let txt: Vec<String> = Self::air_tunes_mdns_props()
            .into_iter()
            .map(|v| format!("{}={}", v.0, v.1))
            .collect();
        let txt: Vec<&str> = txt.iter().map(|v| v.as_str()).collect();
        let service_name = format!("{}@{}", mac.replace(':', ""), self.server_name);
        let svc = responder.register(AIRTUNES_SERVICE_TYPE.into(), service_name, port, &txt);
        self.services.push(svc);
        log::warn!("mDNS 完成 ...............................");
    }

    fn airplay_mdns_props(device_id: String) -> Vec<(&'static str, String)> {
        vec![
            ("deviceid", device_id),
            ("features", "0x5A7FFFF7,0x1E".to_string()),
            ("srcvers", "220.68".to_string()),
            ("flags", "0x44".to_string()),
            ("vv", "2".to_string()),
            ("model", "AppleTV3,2C".to_string()),
            ("rhd", "5.6.0.0".to_string()),
            ("pw", "false".to_string()),
            (
                "pk",
                "f3769a660475d27b4f6040381d784645e13e21c53e6d2da6a8c3d757086fc336".to_string(),
            ),
            ("rmodel", "PC1.0".to_string()),
            ("rrv", "1.01".to_string()),
            ("rsv", "1.00".to_string()),
            ("pcversion", "1715".to_string()),
        ]
    }

    fn air_tunes_mdns_props() -> Vec<(&'static str, &'static str)> {
        vec![
            ("ch", "2"),
            ("cn", "1,3"),
            ("da", "true"),
            ("et", "0,3,5"),
            ("ek", "1"),
            ("ft", "0x5A7FFFF7,0x1E"),
            ("am", "AppleTV3,2C"),
            ("md", "0,1,2"),
            ("sr", "44100"),
            ("ss", "16"),
            ("sv", "false"),
            ("sm", "false"),
            ("tp", "UDP"),
            ("txtvers", "1"),
            ("sf", "0x44"),
            ("vs", "220.68"),
            ("vn", "65537"),
            (
                "pk",
                "f3769a660475d27b4f6040381d784645e13e21c53e6d2da6a8c3d757086fc336",
            ),
        ]
    }
}
