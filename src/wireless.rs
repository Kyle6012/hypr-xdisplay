use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct WirelessDisplay {
    pub name: String,
    pub address: String,
    pub connected: bool,
}

pub async fn scan_wireless_displays() -> Result<Vec<WirelessDisplay>> {
    // Try MiracleCast (miracle-sinkctl list) or wds (wds list)
    let output = Command::new("miracle-sinkctl")
        .arg("list")
        .stdout(Stdio::piped())
        .output()
        .await;
    let mut displays = Vec::new();
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            // Example: 1  00:11:22:33:44:55  Wireless Display 1
            let parts: Vec<_> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let address = parts[1].to_string();
                let name = parts[2..].join(" ");
                displays.push(WirelessDisplay { name, address, connected: false });
            }
        }
    }
    // TODO: Mark connected devices if possible
    Ok(displays)
}

pub async fn connect_wireless_display(address: &str) -> Result<()> {
    // Example: miracle-sinkctl connect <address>
    let _ = Command::new("miracle-sinkctl")
        .arg("connect")
        .arg(address)
        .spawn()?;
    Ok(())
}

pub async fn disconnect_wireless_display(address: &str) -> Result<()> {
    // Example: miracle-sinkctl disconnect <address>
    let _ = Command::new("miracle-sinkctl")
        .arg("disconnect")
        .arg(address)
        .spawn()?;
    Ok(())
} 