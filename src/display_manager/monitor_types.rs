use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub make: String,
    pub model: String,
    pub serial: String,
    pub width: i32,
    pub height: i32,
    #[serde(rename = "refreshRate")]
    pub refresh_rate: f64,
    pub x: i32,
    pub y: i32,
    pub active_workspace: Workspace,
    pub special_workspace: Workspace,
    pub reserved: [i32; 4],
    pub scale: f64,
    pub transform: i32, // 0=landscape, 1=portrait
    pub focused: bool,
    #[serde(rename = "dpmsStatus")]
    pub dpms_status: bool,
    pub vrr: bool,
    #[serde(default)]
    pub mode: Option<String>, // "Extended" or "Copy"
    #[serde(default)]
    pub orientation: Option<String>, // "Landscape" or "Portrait"
    #[serde(default)]
    pub scaling: Option<f64>,
    #[serde(default)]
    pub brightness: Option<f64>, // 0.0-1.0, physical monitors only
    #[serde(default)]
    pub device_type: Option<String>, // "Physical", "Wireless", "Android", "AirPlay", "VNC"
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
} 