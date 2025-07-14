use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{info, error, warn};
use glib::Sender;

#[derive(Debug)]
pub enum HyprlandEvent {
    MonitorAdded,
    MonitorRemoved,
    Unknown(String),
}

pub async fn listen(sender: Sender<HyprlandEvent>) -> Result<()> {
    info!("Starting hyprctl event listener");
    let mut cmd = Command::new("hyprctl")
        .arg("events")
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let stdout = cmd.stdout.take().expect("Failed to get stdout");
    let mut reader = BufReader::new(stdout).lines();

    while let Some(line) = reader.next_line().await? {
        info!("Received event: {}", line);
        let event = parse_event(&line);
        if let Err(_) = sender.send(event) {
            error!("Failed to send event to UI thread. Receiver dropped.");
            break;
        }
        // TODO: Optionally handle event here if needed
    }
    
    warn!("Hyprctl event stream ended.");
    Ok(())
}

fn parse_event(line: &str) -> HyprlandEvent {
    if line.starts_with("monitoradded>>") {
        HyprlandEvent::MonitorAdded
    } else if line.starts_with("monitorremoved>>") {
        HyprlandEvent::MonitorRemoved
    } else {
        HyprlandEvent::Unknown(line.to_string())
    }
}