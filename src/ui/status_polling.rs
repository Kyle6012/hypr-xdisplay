use libadwaita as adw;
use crate::ui::main_window::AppState;
use gtk::prelude::*;
use gtk::glib;

pub fn setup_status_polling(_window: &adw::ApplicationWindow, _app_state: AppState) {
    // Create a status bar at the bottom
    let status_bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    status_bar.set_hexpand(true);
    status_bar.set_valign(gtk::Align::End);
    status_bar.set_margin_bottom(4);
    status_bar.set_margin_start(12);
    status_bar.set_margin_end(12);

    // Example status widgets
    let rec_icon = gtk::Image::from_icon_name("media-record-symbolic");
    let rec_label = gtk::Label::new(Some("Not Recording"));
    let monitor_icon = gtk::Image::from_icon_name("display-symbolic");
    let monitor_label = gtk::Label::new(Some("Monitors OK"));
    let net_icon = gtk::Image::from_icon_name("network-wireless-symbolic");
    let net_label = gtk::Label::new(Some("Network OK"));
    status_bar.append(&rec_icon);
    status_bar.append(&rec_label);
    status_bar.append(&monitor_icon);
    status_bar.append(&monitor_label);
    status_bar.append(&net_icon);
    status_bar.append(&net_label);

    // TODO: Add status_bar to main layout in main_window.rs
    // Example: main_box.append(&status_bar);

    // Example: update recording status every 2s
    let rec_label_clone = rec_label.clone();
    glib::timeout_add_seconds_local(2, move || {
        // TODO: Query real recording status
        rec_label_clone.set_text("Not Recording");
        glib::ControlFlow::Continue
    });
    // TODO: Add real monitor/network polling and update labels/icons/colors
}