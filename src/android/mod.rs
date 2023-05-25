use std::process::Stdio;

use tokio::{io, process::Command};

pub async fn uninstall_package(package_name: &str) -> io::Result<()> {
    let output = Command::new("pm")
        .args(["list", "packages"])
        .stdout(Stdio::null())
        .output()
        .await?;
    let list = String::from_utf8_lossy(&output.stdout);
    if list.contains(package_name) {
        Command::new("pm")
            .args(["uninstall", package_name])
            .status()
            .await?;
    }
    Ok(())
}
