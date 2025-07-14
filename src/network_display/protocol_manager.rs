use crate::network_display::protocol_types::{ProtocolType, Role, ProtocolStatus};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

static PROTOCOL_PROCESSES: Lazy<Mutex<HashMap<(ProtocolType, Role), Child>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static PROTOCOL_STATUS: Lazy<Mutex<HashMap<(ProtocolType, Role), ProtocolStatus>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub async fn start_protocol(protocol: ProtocolType, role: Role, port: Option<u16>, extra: Option<&str>) -> anyhow::Result<()> {
    let mut processes = PROTOCOL_PROCESSES.lock().await;
    let mut status_map = PROTOCOL_STATUS.lock().await;
    if processes.contains_key(&(protocol, role)) {
        return Ok(()); // Already running
    }
    let mut cmd = match (protocol, role) {
        (ProtocolType::AirPlay, Role::Receiver) => {
            let mut c = Command::new("miracle-sinkctl");
            c.arg("--daemon");
            c
        },
        (ProtocolType::Miracast, Role::Receiver) => {
            let mut c = Command::new("miracle-sinkctl");
            c.arg("--daemon");
            c
        },
        (ProtocolType::VNC, Role::Receiver) => {
            let mut c = Command::new("wayvnc");
            if let Some(port) = port {
                c.arg(format!("0.0.0.0:{}", port));
            }
            c
        },
        (ProtocolType::Browser, Role::Receiver) => {
            let mut c = Command::new("waypipe");
            c.arg("server");
            c.arg("--websocket");
            if let Some(port) = port {
                c.arg("--port");
                c.arg(port.to_string());
            }
            c
        },
        (ProtocolType::AirPlay, Role::Sender) => {
            let address = extra.unwrap_or("");
            let mut c = Command::new("gst-launch-1.0");
            c.arg("ximagesrc");
            c.arg("!");
            c.arg("videoconvert");
            c.arg("!");
            c.arg("jpegenc");
            c.arg("!");
            c.arg(format!("rtpjpegpay ! udpsink host={} port=5000", address));
            c
        },
        (ProtocolType::Miracast, Role::Sender) => {
            let address = extra.unwrap_or("");
            let mut c = Command::new("gst-launch-1.0");
            c.arg("ximagesrc");
            c.arg("!");
            c.arg("videoconvert");
            c.arg("!");
            c.arg("jpegenc");
            c.arg("!");
            c.arg(format!("rtpjpegpay ! udpsink host={} port=5000", address));
            c
        },
        (ProtocolType::VNC, Role::Sender) => {
            let mut c = Command::new("x11vnc");
            if let Some(port) = port {
                c.arg("-rfbport");
                c.arg(port.to_string());
            }
            c
        },
        (ProtocolType::Browser, Role::Sender) => {
            let mut c = Command::new("ffmpeg");
            c.arg("-f");
            c.arg("x11grab");
            c.arg("-i");
            c.arg(":0");
            c.arg("-f");
            c.arg("mpeg1video");
            c.arg("-b");
            c.arg("800k");
            c.arg("-r");
            c.arg("30");
            if let Some(port) = port {
                c.arg(format!("ws://localhost:{}/", port));
            } else {
                c.arg("ws://localhost:8082/");
            }
            c
        },
    };
    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    match cmd.spawn() {
        Ok(child) => {
            processes.insert((protocol, role), child);
            status_map.insert((protocol, role), ProtocolStatus {
                running: true,
                port,
                error: None,
                last_status_message: Some(format!("{} {:?} started successfully.", match protocol {
                    ProtocolType::AirPlay => "AirPlay",
                    ProtocolType::Miracast => "Miracast",
                    ProtocolType::VNC => "VNC",
                    ProtocolType::Browser => "Browser",
                }, role)),
            });
            Ok(())
        },
        Err(e) => {
            status_map.insert((protocol, role), ProtocolStatus {
                running: false,
                port,
                error: Some(format!("Failed to start: {}", e)),
                last_status_message: Some("Check if required dependencies are installed and not already running.".to_string()),
            });
            Err(e.into())
        }
    }
}

pub async fn stop_protocol(protocol: ProtocolType, role: Role) -> anyhow::Result<()> {
    let mut processes = PROTOCOL_PROCESSES.lock().await;
    let mut status_map = PROTOCOL_STATUS.lock().await;
    if let Some(mut child) = processes.remove(&(protocol, role)) {
        match child.kill().await {
            Ok(_) => {
                status_map.insert((protocol, role), ProtocolStatus {
                    running: false,
                    port: None,
                    error: None,
                    last_status_message: Some(format!("{} {:?} stopped.", match protocol {
                        ProtocolType::AirPlay => "AirPlay",
                        ProtocolType::Miracast => "Miracast",
                        ProtocolType::VNC => "VNC",
                        ProtocolType::Browser => "Browser",
                    }, role)),
                });
            },
            Err(e) => {
                status_map.insert((protocol, role), ProtocolStatus {
                    running: false,
                    port: None,
                    error: Some(format!("Failed to stop: {}", e)),
                    last_status_message: Some("Process may not have been running.".to_string()),
                });
            }
        }
    }
    Ok(())
}

pub async fn get_protocol_status(protocol: ProtocolType, role: Role) -> ProtocolStatus {
    let status_map = PROTOCOL_STATUS.lock().await;
    status_map.get(&(protocol, role)).cloned().unwrap_or(ProtocolStatus { running: false, port: None, error: None, last_status_message: None })
} 