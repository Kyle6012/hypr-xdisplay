use crate::display_manager::monitor_types::{Monitor, Workspace};
use std::process::Stdio;
use tokio::process::Command;

pub async fn get_monitors() -> anyhow::Result<Vec<Monitor>> {
    let output = Command::new("hyprctl")
        .arg("monitors")
        .arg("-j")
        .output()
        .await?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "hyprctl command failed: {}",
            error_message
        ));
    }

    let monitors: Vec<Monitor> = serde_json::from_slice(&output.stdout)?;
    Ok(monitors)
}

pub async fn apply_monitor_layout(monitors: &[Monitor]) -> anyhow::Result<()> {
    let mut command_batch = String::new();
    let mut current_x = 0;
    let mut extended_geometry = None;

    // First pass: find the geometry of the first Extended monitor (for mirroring)
    for monitor in monitors {
        if monitor.mode.as_deref() == Some("Extended") {
            extended_geometry = Some((monitor.width, monitor.height, monitor.refresh_rate, current_x, monitor.y));
            break;
        }
        current_x += monitor.width;
    }

    // Second pass: generate commands
    let mut current_x = 0;
    for monitor in monitors {
        let mode = monitor.mode.as_deref().unwrap_or("Extended");
        let orientation = monitor.orientation.as_deref().unwrap_or("Landscape");
        let scaling = monitor.scaling.unwrap_or(monitor.scale);
        let transform = if orientation == "Portrait" { 1 } else { 0 };
        let (width, height, refresh_rate, x, y) = if mode == "Copy" {
            if let Some((w, h, r, x, y)) = extended_geometry {
                (w, h, r, x, y)
            } else {
                (monitor.width, monitor.height, monitor.refresh_rate, current_x, monitor.y)
            }
        } else {
            (monitor.width, monitor.height, monitor.refresh_rate, current_x, monitor.y)
        };
        let command = format!(
            "keyword monitor {},{}x{}@{},{}x{},{}; keyword monitor {} scale {};",
            monitor.name, width, height, refresh_rate, x, y, monitor.name, scaling
        );
        command_batch.push_str(&command);
        if mode == "Extended" {
            current_x += width;
        }
    }

    if command_batch.is_empty() {
        return Ok(());
    }

    let output = Command::new("hyprctl")
        .arg("--batch")
        .arg(&command_batch)
        .output()
        .await?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "hyprctl batch command failed: {}",
            error_message
        ));
    }

    Ok(())
}

// Set brightness for a physical monitor using ddcutil
pub async fn set_brightness(monitor: &Monitor, value: f64) -> anyhow::Result<()> {
    // Only for physical monitors
    if monitor.device_type.as_deref() == Some("Physical") {
        let pct = (value * 100.0).round() as u8;
        let _ = Command::new("ddcutil")
            .arg("setvcp").arg("10")
            .arg(format!("{}", pct))
            .arg("--sn").arg(&monitor.serial)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .await?;
    }
    Ok(())
}

// Set rotation for a physical monitor using Hyprland
pub async fn set_rotation(monitor: &Monitor, orientation: &str) -> anyhow::Result<()> {
    if monitor.device_type.as_deref() == Some("Physical") {
        let transform = if orientation == "Portrait" { 1 } else { 0 };
        let cmd = format!("keyword monitor {} transform {}", monitor.name, transform);
        let _ = Command::new("hyprctl")
            .arg(&cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .await?;
    }
    Ok(())
} 