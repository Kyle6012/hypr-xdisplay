use anyhow::Result;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::info;
use once_cell::sync::Lazy;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use crate::settings::Settings;
use std::time::Instant;
use std::fs;

pub static RECORDER_PROCESS: Lazy<Arc<Mutex<Option<(Child, Instant)>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

pub async fn start_recording(settings: &Settings) -> Result<()> {
    let mut process_lock = RECORDER_PROCESS.lock().await;
    if process_lock.is_some() {
        info!("Recording is already in progress.");
        return Ok(());
    }

    let now = chrono::Local::now();
    let timestamp = now.format(&settings.recorder_filename_format).to_string();
    let path = settings.recorder_dir.join(timestamp);

    if let Err(e) = fs::create_dir_all(&settings.recorder_dir) {
        tracing::warn!("Failed to create recording directory: {}", e);
    }

    info!("Starting recording to {:?} with codec {}", path, settings.recorder_codec);

    let mut cmd = Command::new("wf-recorder");
    cmd.arg("-f").arg(path.to_str().unwrap());
    cmd.arg("-c").arg(&settings.recorder_codec);
    // Framerate
    if let Some(fps) = settings.recorder_framerate {
        cmd.arg("-r").arg(fps.to_string());
    }
    // Resolution
    if let Some(ref res) = settings.recorder_resolution {
        cmd.arg("-g").arg(res);
    }
    // Hardware acceleration (example: --hwaccel)
    if settings.recorder_hardware_accel.unwrap_or(false) {
        cmd.arg("--hwaccel");
    }
    // Audio device
    if let Some(ref dev) = settings.recorder_audio_device {
        cmd.arg("--audio").arg(dev);
    }
    // Audio source
    if let Some(ref audio) = settings.recorder_audio_source {
        if audio != "None" {
            cmd.arg("--audio");
            if audio != "Default" {
                cmd.arg(audio);
            }
        }
    }
    // Advanced encoder args
    if let Some(ref adv) = settings.recorder_advanced_args {
        for arg in adv.split_whitespace() {
            cmd.arg(arg);
        }
    }
    let child = cmd.spawn()?;
    *process_lock = Some((child, Instant::now()));
    Ok(())
}

pub async fn stop_recording() -> Result<()> {
    let mut process_lock = RECORDER_PROCESS.lock().await;
    if let Some((mut child, _)) = process_lock.take() {
        info!("Stopping recording...");
        child.kill().await?;
        info!("Recording stopped.");
    } else {
        info!("No recording in progress to stop.");
    }
    Ok(())
}

pub async fn pause_recording() -> Result<()> {
    let process_lock = RECORDER_PROCESS.lock().await;
    if let Some((child, _)) = process_lock.as_ref() {
        let pid = Pid::from_raw(child.id().unwrap() as i32);
        info!("Pausing recording (PID: {})", pid);
        signal::kill(pid, Signal::SIGUSR1)?;
    } else {
        info!("No recording in progress to pause.");
    }
    Ok(())
}

pub async fn resume_recording() -> Result<()> {
    let process_lock = RECORDER_PROCESS.lock().await;
    if let Some((child, _)) = process_lock.as_ref() {
        let pid = Pid::from_raw(child.id().unwrap() as i32);
        info!("Resuming recording (PID: {})", pid);
        signal::kill(pid, Signal::SIGUSR2)?;
    } else {
        info!("No recording in progress to resume.");
    }
    Ok(())
}