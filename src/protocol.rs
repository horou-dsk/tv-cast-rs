use super::constant::SERVER_PORT;
use std::net::Ipv4Addr;

#[derive(Clone)]
pub struct DLNAHandler {
    description: String,
    pub uuid: String,
}

impl DLNAHandler {
    pub fn new(usn: &str, ip: Ipv4Addr, name: String) -> Self {
        let description = format!(
            include_str!("./xml/Description.xml"),
            uuid = usn,
            friendly_name = name,
            manufacturer = "Microsoft Corporation",
            manufacturer_url = "http://www.microsoft.com",
            model_description = "Media Renderer",
            model_name = "Windows Media Player",
            model_url = "http://go.microsoft.com/fwlink/Linkld=105927",
            model_number = "1.0",
            serial_num = 1024,
            header_extra = "",
            service_extra = "",
            url_base = format_args!(
                "http://{}:{SERVER_PORT}",
                default_net::interface::get_local_ipaddr().unwrap_or(ip.into())
            )
        );
        Self {
            description,
            uuid: usn.to_string(),
        }
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}
