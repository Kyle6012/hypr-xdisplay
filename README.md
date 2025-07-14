<div align="center">
  <img src="img.png" alt="Hypr-XDisplay Logo" width="180"/>
</div>

# Hypr-XDisplay

A modern, modular display and network streaming manager for Hyprland (Wayland) compositors. Supports physical, wireless, Android/mobile, and network displays, with advanced screenshot and screen recording utilities.

---

## Features
- **Physical, Wireless, Android/Mobile, and Network Display Management**
- **Network Display Protocols:** AirPlay, Miracast, VNC, Browser/WebRTC
- **Advanced Screenshot Utility:** Fullscreen, region, window, clipboard, annotation, upload
- **Screen Recording Utility:** Format, codec, quality, audio, advanced options, live timer
- **Global PrintScreen Handler:** Quick overlay for screenshot/record selection
- **Modern GTK4 + Libadwaita UI:** Responsive, modular
- **Status feedback, toasts, and real-time device discovery**

---

## Supported OS/Compositor
- **Arch Linux, Athena OS, and derivatives**
- **Debian/Ubuntu (22.04+, 23.04+)**
- **Requires Hyprland (Wayland, wlroots-based)**

---

## Dependencies
### **Core**
- `gtk4`, `libadwaita`, `tokio`, `serde`, `tracing`, `dirs`, `config`, `glib`, `nix`, `chrono`

### **System/Runtime**
- **Wayland/Hyprland** (compositor)
- **PipeWire** (`pipewire`, `wireplumber`)
- **xdg-desktop-portal-hyprland** (Arch: official, Ubuntu: may need to build or use `xdg-desktop-portal-wlr`)
- **grim** and **slurp** (screenshots)
- **wf-recorder** (screen recording)
- **wayvnc** (VNC server for Wayland)
- **waypipe** (Wayland app forwarding)
- **miraclecast** (Miracast, experimental)
- **adb**, **scrcpy** (Android device streaming)
- **avahi** (network discovery)
- **curl**, **wl-copy**, **swappy** (optional: upload, clipboard, annotation)

### **Install on Arch Linux / Athena OS**
```sh
sudo pacman -Syu \
  gtk4 libadwaita pipewire wireplumber xdg-desktop-portal xdg-desktop-portal-hyprland \
  grim slurp wf-recorder wayvnc waypipe avahi curl wl-clipboard swappy adb scrcpy
# For Miracast (optional, experimental):
yay -S miraclecast
```

### **Install on Debian/Ubuntu**
```sh
sudo apt update && sudo apt install \
  gtk4 libadwaita-1-dev pipewire wireplumber xdg-desktop-portal xdg-desktop-portal-wlr \
  grim slurp wf-recorder wayvnc waypipe avahi-daemon curl wl-clipboard swappy adb scrcpy
# For Miracast (optional, experimental):
sudo apt install miraclecast
```
- For `xdg-desktop-portal-hyprland`, you may need to build from source on Ubuntu, or use `xdg-desktop-portal-wlr` for basic sharing.

---

## Building & Running
```sh
git clone https://github.com/Kyle6012/hypr-xdisplay.git
cd hypr-xdisplay
cargo build --release
./target/release/hypr-xdisplay
```

---

## Usage
- **Launch the app:**
  - From terminal: `hypr-xdisplay`
  - Or add to your autostart/session
- **UI:**
  - Drag and arrange displays, toggle protocols, configure network display settings
  - Use the screenshot and recording panels for advanced capture
  - Press `PrintScreen` for a quick overlay to choose screenshot or screen record (with mode selection)
- **Settings:**
  - All settings are saved in `~/.config/hypr-xdisplay/settings.toml`

---

## How It Works
- **Physical displays** are managed via Hyprland and ddcutil
- **Wireless displays** use MiracleCast (Miracast) and Avahi for discovery
- **Android devices** use ADB and scrcpy for streaming
- **Network display protocols** (AirPlay, VNC, Browser/WebRTC) are managed via protocol daemons (wayvnc, waypipe, etc.)
- **Screen sharing** is enabled via PipeWire and xdg-desktop-portal-hyprland
- **Screenshot/recording** use grim, slurp, wf-recorder, and integrate with clipboard, annotation, and upload tools


---

## Troubleshooting
- **Missing packages:**
  - Update your system and mirrors (`sudo pacman -Syu` or `sudo apt update`)
  - For AUR packages, use `yay` or `paru`
- **Screen sharing not working:**
  - Ensure `pipewire`, `wireplumber`, and `xdg-desktop-portal-hyprland` are running
  - On Ubuntu, you may need to build `xdg-desktop-portal-hyprland` from source
- **Miracast unreliable:**
  - Linux Miracast is experimental; try different Wi-Fi adapters or kernel versions
- **Wayland/Hyprland only:**
  - This app does not support X11, GNOME, or KDE

---

## Contributing
- PRs and issues welcome! See [CONTRIBUTING.md] if available.

---

## Author
- Meshack Bahati

--

## License
MIT 