use std::net::Ipv4Addr;

#[derive(Clone)]
pub struct DLNAHandler {
    description: String,
    pub uuid: String,
}

impl DLNAHandler {
    pub fn new(usn: &str, ip: Ipv4Addr) -> Self {
        let description = format!(
            include_str!("./xml/Description.xml"),
            uuid = usn,
            friendly_name = format!("Rust盒子投屏-{}", ip),
            manufacturer = "Microsoft Corporation",
            manufacturer_url = "http://www.microsoft.com",
            model_description = "Media Renderer",
            model_name = "Windows Media Player",
            model_url = "http://go.microsoft.com/fwlink/Linkld=105927",
            model_number = "1.0",
            serial_num = 1024,
            header_extra = "",
            service_extra = "",
            url_base = format!("http://{}:8080", ip)
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
