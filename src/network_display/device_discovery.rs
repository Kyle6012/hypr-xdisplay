use crate::network_display::protocol_types::ProtocolType;
use tokio::process::Command;

pub async fn discover_sender_targets(protocol: ProtocolType) -> anyhow::Result<Vec<(String, String)>> {
    match protocol {
        ProtocolType::AirPlay | ProtocolType::Miracast => {
            let output = Command::new("avahi-browse")
                .arg("-rt")
                .arg("_airplay._tcp")
                .output()
                .await?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut devices = Vec::new();
            for line in stdout.lines() {
                if line.contains("IPv4") && line.contains("_airplay._tcp") {
                    let parts: Vec<_> = line.split(';').collect();
                    if parts.len() > 7 {
                        let name = parts[3].trim().to_string();
                        let address = parts[7].trim().to_string();
                        devices.push((name, address));
                    }
                }
            }
            Ok(devices)
        },
        ProtocolType::VNC => {
            let mut devices = Vec::new();
            for port in 5900..=5910 {
                let addr = format!("127.0.0.1:{}", port);
                devices.push((format!("VNC on {}", addr), addr));
            }
            Ok(devices)
        },
        ProtocolType::Browser => {
            let mut devices = Vec::new();
            devices.push(("Local Browser (ws://localhost:8082/)".to_string(), "ws://localhost:8082/".to_string()));
            Ok(devices)
        },
    }
} 