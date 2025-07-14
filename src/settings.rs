use serde::Deserialize;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub screenshot_dir: PathBuf,
    pub screenshot_filename_format: String,
    pub recorder_dir: PathBuf,
    pub recorder_filename_format: String,
    pub recorder_codec: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            screenshot_dir: dirs::picture_dir().unwrap_or_else(|| PathBuf::from("/tmp")),
            screenshot_filename_format: "screenshot_%Y-%m-%d_%H-%M-%S.png".to_string(),
            recorder_dir: dirs::video_dir().unwrap_or_else(|| PathBuf::from("/tmp")),
            recorder_filename_format: "recording_%Y-%m-%d_%H-%M-%S.mp4".to_string(),
            recorder_codec: "libx264".to_string(),
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .map(|p| p.join("hypr-xdisplay"))
            .unwrap();
        let config_file = config_dir.join("settings.toml");

        info!("Loading settings from {:?}", config_file);

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
}