use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct AndroidDevice {
    pub id: String,
    pub name: String,
    pub connected: bool,
}

pub async fn scan_android_devices() -> Result<Vec<AndroidDevice>> {
    // Use adb devices
    let output = Command::new("adb")
        .arg("devices")
        .stdout(Stdio::piped())
        .output()
        .await;
    let mut devices = Vec::new();
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.ends_with("device") && !line.starts_with("List of devices") {
                let parts: Vec<_> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let id = parts[0].to_string();
                    // Optionally, get device name with adb -s <id> shell getprop ro.product.model
                    let name = id.clone();
                    devices.push(AndroidDevice { id, name, connected: false });
                }
            }
        }
    }
    Ok(devices)
}

pub async fn connect_android_device(id: &str) -> Result<()> {
    // Start scrcpy for this device
    let _ = Command::new("scrcpy")
        .arg("-s")
        .arg(id)
        .spawn()?;
    Ok(())
}

pub async fn disconnect_android_device(_id: &str) -> Result<()> {
    // TODO: Kill the scrcpy process for this device (requires tracking PIDs)
    // For now, killall scrcpy (not ideal)
    let _ = Command::new("killall")
        .arg("scrcpy")
        .spawn()?;
    Ok(())
} 