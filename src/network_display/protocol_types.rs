#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolType {
    AirPlay,
    Miracast,
    VNC,
    Browser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Receiver,
    Sender,
}

#[derive(Debug, Clone)]
pub struct ProtocolStatus {
    pub running: bool,
    pub port: Option<u16>,
    pub error: Option<String>,
    pub last_status_message: Option<String>,
} 