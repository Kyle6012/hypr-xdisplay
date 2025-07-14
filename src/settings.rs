use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;
use std::fs;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Settings {
    pub screenshot_dir: PathBuf,
    pub screenshot_filename_format: String,
    pub recorder_dir: PathBuf,
    pub recorder_filename_format: String,
    pub recorder_codec: String,
    pub recorder_audio_source: Option<String>,
    pub recorder_advanced_args: Option<String>,
    pub recorder_framerate: Option<u32>,
    pub recorder_resolution: Option<String>,
    pub recorder_hardware_accel: Option<bool>,
    pub recorder_audio_device: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        let screenshot_dir = dirs::picture_dir()
            .map(|p| p.join("Screenshots"))
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        let recorder_dir = dirs::video_dir()
            .map(|p| p.join("Screenrecord"))
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        Self {
            screenshot_dir,
            screenshot_filename_format: "screenshot_%Y-%m-%d_%H-%M-%S.png".to_string(),
            recorder_dir,
            recorder_filename_format: "recording_%Y-%m-%d_%H-%M-%S.mp4".to_string(),
            recorder_codec: "libx264".to_string(),
            recorder_audio_source: Some("Default".to_string()),
            recorder_advanced_args: None,
            recorder_framerate: Some(30),
            recorder_resolution: None,
            recorder_hardware_accel: Some(false),
            recorder_audio_device: None,
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .map(|p| p.join("hypr-xdisplay"))
            .unwrap();
        let config_file = config_dir.join("settings.toml");
        let settings = config::Config::builder()
            .add_source(config::File::from(config_file).required(false))
            .build();

        match settings {
            Ok(config) => config.try_deserialize().unwrap_or_else(|e| {
                info!("Failed to deserialize settings, using defaults: {}", e);
                Settings::default()
            }),
            Err(e) => {
                info!("Failed to build settings, using defaults: {}", e);
                Settings::default()
            }
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let config_dir = dirs::config_dir()
            .map(|p| p.join("hypr-xdisplay"))
            .unwrap();
        fs::create_dir_all(&config_dir)?;
        let config_file = config_dir.join("settings.toml");
        let toml = toml::to_string_pretty(self).unwrap();
        fs::write(config_file, toml)
    }
}