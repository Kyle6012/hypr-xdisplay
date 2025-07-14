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

    info!("Starting recording to {:?} with codec {}", path, settings.recorder_codec);

    let child = Command::new("wf-recorder")
        .arg("-f")
        .arg(path.to_str().unwrap())
        .arg("-c")
        .arg(&settings.recorder_codec)
        .spawn()?;
    
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