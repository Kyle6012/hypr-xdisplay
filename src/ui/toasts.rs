use libadwaita as adw;
use gtk::prelude::*;

pub fn setup_toast_overlay(_window: &adw::ApplicationWindow, content: &gtk::Box) {
    let overlay = adw::ToastOverlay::new();
    overlay.set_hexpand(true);
    overlay.set_vexpand(true);
    content.append(&overlay);
}

pub fn show_success_toast(overlay: &adw::ToastOverlay, message: &str) {
    let toast = adw::Toast::builder()
        .title(message)
        .timeout(3)
        .build();
    overlay.add_toast(toast);
}

pub fn show_error_toast(overlay: &adw::ToastOverlay, message: &str) {
    let toast = adw::Toast::builder()
        .title(message)
        .timeout(5)
        .build();
    overlay.add_toast(toast);
}

pub fn show_info_toast(overlay: &adw::ToastOverlay, message: &str) {
    let toast = adw::Toast::builder()
        .title(message)
        .timeout(4)
        .build();
    overlay.add_toast(toast);
}

pub fn show_toast(msg: &str) {
    println!("[TOAST] {}", msg);
}