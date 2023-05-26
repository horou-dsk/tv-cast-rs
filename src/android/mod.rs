use std::{path::Path, process::Stdio};

use tokio::{io, process::Command};

pub async fn uninstall_package(package_name: &str) -> io::Result<()> {
    let output = Command::new("su")
        .arg("root")
        .args(["pm", "list", "packages"])
        .stdout(Stdio::null())
        .output()
        .await?;
    let list = String::from_utf8_lossy(&output.stdout);
    if list.contains(package_name) {
        Command::new("su")
            .arg("root")
            .args(["pm", "uninstall", package_name])
            .status()
            .await?;
    }
    Ok(())
}

pub async fn remove_file(path: &Path) -> io::Result<()> {
    Command::new("rm").arg(path).status().await?;
    Ok(())
}
