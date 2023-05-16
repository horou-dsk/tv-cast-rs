#![feature(lazy_cell)]

use std::{process::Stdio, sync::LazyLock};

use regex::Regex;
use tokio::process::Command;

static IP_MATCH: LazyLock<Regex> = LazyLock::new(|| Regex::new("\\?\\s\\(([\\d\\.]+)\\)").unwrap());
static MAC_MATCH: LazyLock<Regex> = LazyLock::new(|| Regex::new("at\\s([a-z0-9:]+)").unwrap());

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    args.next();
    let target_ip = args.next().expect("请输入目标Ip地址");
    let output = Command::new("arp")
        .args(["-a", &target_ip])
        .stdout(Stdio::null())
        .output()
        .await?;
    let result = String::from_utf8_lossy(&output.stdout);
    let ip_caps = IP_MATCH.captures(&result);
    let mac_caps = MAC_MATCH.captures(&result);
    let err = Err(std::io::ErrorKind::NotFound.into());
    let (Some(ip_cap), Some(mac_cap)) = (ip_caps, mac_caps) else {return err};
    let (Some(ip), Some(mac)) = (ip_cap.get(1), mac_cap.get(1)) else {return err};
    println!("ip = {} mac = {}", ip.as_str(), mac.as_str());
    Ok(())
}
