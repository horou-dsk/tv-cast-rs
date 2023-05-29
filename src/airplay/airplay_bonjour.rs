use mdns_sd::{ServiceDaemon, ServiceInfo};

use crate::{constant::SERVER_PORT, setting};

const AIRPLAY_SERVICE_TYPE: &str = "._airplay._tcp.local";

const AIRTUNES_SERVICE_TYPE: &str = "._raop._tcp.local";

pub struct AirPlayBonjour {
    server_name: String,
    service_daemon: ServiceDaemon,
    service_names: Vec<String>, // mdns_list: Vec<>
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
            service_names: Vec::new(),
            service_daemon: ServiceDaemon::new()
                .inspect_err(|err| log::error!("{err:?}"))
                .unwrap(),
        }
    }

    pub fn start(&mut self) {
        let service_hostname = format!("{}.local.", self.server_name);
        let port = SERVER_PORT;
        log::info!("mDNS port = {port}");
        let interfaces = setting::get_ip().unwrap();
        for interface in interfaces {
            let mac = interface.2.to_string();
            let service_info = ServiceInfo::new(
                AIRPLAY_SERVICE_TYPE,
                &self.server_name,
                &service_hostname,
                interface.0,
                port,
                &Self::airplay_mdns_props(mac.clone())[..],
            )
            .expect("valid service info");
            // let monitor = self.service_daemon.monitor().expect("Failed to monitor the daemon");
            let service_fullname = service_info.get_fullname().to_string();
            self.service_daemon
                .register(service_info)
                .expect("Failed to register mDNS service");
            self.service_names.push(service_fullname);

            let air_tunes_server_name = format!("{}@{}", mac.replace(':', ""), self.server_name);
            let service_info = ServiceInfo::new(
                AIRTUNES_SERVICE_TYPE,
                &air_tunes_server_name,
                &service_hostname,
                interface.0,
                port,
                &Self::air_tunes_mdns_props()[..],
            )
            .expect("valid service info");
            let service_fullname = service_info.get_fullname().to_string();
            self.service_daemon
                .register(service_info)
                .expect("Failed to register mDNS service");
            self.service_names.push(service_fullname);
        }
        log::warn!("mDNS 完成 ...............................");
    }

    fn airplay_mdns_props(device_id: String) -> Vec<(&'static str, String)> {
        vec![
            ("device_id", device_id),
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
            ("st", "44100"),
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

impl Drop for AirPlayBonjour {
    fn drop(&mut self) {
        for service_name in &self.service_names {
            if let Err(err) = self.service_daemon.unregister(service_name) {
                log::error!("unregister error {:?}", err);
            }
        }
    }
}
