use anyhow::Result;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::info;
use crate::settings::Settings;

fn get_screenshot_path(settings: &Settings) -> PathBuf {
    let now = chrono::Local::now();
    let timestamp = now.format(&settings.screenshot_filename_format).to_string();
    settings.screenshot_dir.join(timestamp)
}

pub async fn capture_fullscreen(settings: &Settings) -> Result<PathBuf> {
    let path = get_screenshot_path(settings);
    info!("Capturing full screen to {:?}", path);
    let status = Command::new("grim")
        .arg(path.to_str().unwrap())
        .status()
        .await?;

    if !status.success() {
        return Err(anyhow::anyhow!("grim command failed for fullscreen capture"));
    }
    Ok(path)
}

pub async fn capture_region(settings: &Settings) -> Result<PathBuf> {
    let path = get_screenshot_path(settings);
    info!("Capturing region to {:?}", path);

    let slurp_output = Command::new("slurp").output().await?;
    if !slurp_output.status.success() {
        return Err(anyhow::anyhow!("slurp command failed to select region"));
    }
    let region = String::from_utf8_lossy(&slurp_output.stdout).trim().to_string();

    let status = Command::new("grim")
        .arg("-g")
        .arg(region)
        .arg(path.to_str().unwrap())
        .status()
        .await?;

    if !status.success() {
        return Err(anyhow::anyhow!("grim command failed for region capture"));
    }
    Ok(path)
}

pub async fn capture_focused_window(settings: &Settings) -> Result<PathBuf> {
    let path = get_screenshot_path(settings);
    info!("Capturing focused window to {:?}", path);

    let hyprctl_output = Command::new("hyprctl")
        .arg("activewindow")
        .output()
        .await?;
    if !hyprctl_output.status.success() {
        return Err(anyhow::anyhow!("hyprctl activewindow command failed"));
    }

    // This is a simplified parser. A more robust solution would use regex or a proper parser.
    let output_str = String::from_utf8_lossy(&hyprctl_output.stdout);
    let at_line = output_str.lines().find(|line| line.contains("at:"));
    let size_line = output_str.lines().find(|line| line.contains("size:"));

    if let (Some(at_line), Some(size_line)) = (at_line, size_line) {
        let at = at_line.split("at:").nth(1).unwrap_or("").trim();
        let size = size_line.split("size:").nth(1).unwrap_or("").trim().replace(',', "x");
        let geometry = format!("{},{}", at, size);

        let status = Command::new("grim")
            .arg("-g")
            .arg(geometry)
            .arg(path.to_str().unwrap())
            .status()
            .await?;

        if !status.success() {
            return Err(anyhow::anyhow!("grim command failed for focused window capture"));
        }
    } else {
        return Err(anyhow::anyhow!("Could not determine focused window geometry"));
    }

    Ok(path)
}