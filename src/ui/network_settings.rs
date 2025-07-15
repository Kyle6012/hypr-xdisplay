use std::sync::Arc;
use crate::settings::Settings;
use crate::ui::main_window::{AppState, AppDevices};
use gtk::prelude::*;
use libadwaita as adw;
use crate::network_display::protocol_manager;
use crate::network_display::protocol_types::{ProtocolType, Role};
use tokio::runtime::Runtime;
use crate::wireless;
use crate::android;
use gtk::glib;

fn clear_list_box(list_box: &gtk::ListBox) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }
}

pub fn add_network_settings_section(content: &gtk::Box, _app_state: AppState, _settings: Arc<Settings>, _devices: AppDevices) {
    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(900);
    clamp.set_margin_top(24);
    clamp.set_margin_bottom(24);
    clamp.set_margin_start(24);
    clamp.set_margin_end(24);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 16);
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&gtk::Label::new(Some("Network Display"))));
    vbox.append(&header);

    // Protocols
    let protocols = vec![
        (ProtocolType::AirPlay, "AirPlay", "airplay-symbolic"),
        (ProtocolType::Miracast, "Miracast", "network-wireless-symbolic"),
        (ProtocolType::VNC, "VNC", "network-vnc-symbolic"),
        (ProtocolType::Browser, "Browser", "web-browser-symbolic"),
    ];

    let main_context = glib::MainContext::default();

    for (ptype, label, icon) in protocols {
        let frame = gtk::Frame::new(Some(label));
        frame.set_margin_bottom(8);
        frame.set_margin_top(8);
        frame.set_margin_start(8);
        frame.set_margin_end(8);
        frame.set_width_request(350);
        frame.set_height_request(120);
        frame.set_valign(gtk::Align::Center);
        frame.set_halign(gtk::Align::Center);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let icon_img = gtk::Image::from_icon_name(icon);
        hbox.append(&icon_img);
        let role_combo = gtk::ComboBoxText::new();
        role_combo.append_text("Receiver");
        role_combo.append_text("Sender");
        role_combo.set_active(Some(0));
        hbox.append(&role_combo);
        let port_entry = gtk::Entry::new();
        port_entry.set_placeholder_text(Some("Port (optional)"));
        port_entry.set_width_chars(6);
        hbox.append(&port_entry);
        let extra_entry = gtk::Entry::new();
        extra_entry.set_placeholder_text(Some("Address/Extra (sender only)"));
        extra_entry.set_width_chars(12);
        hbox.append(&extra_entry);
        let start_btn = gtk::Button::with_label("Start");
        let stop_btn = gtk::Button::with_label("Stop");
        hbox.append(&start_btn);
        hbox.append(&stop_btn);
        let status_label = gtk::Label::new(Some("Status: Unknown"));
        hbox.append(&status_label);
        frame.set_child(Some(&hbox));
        vbox.append(&frame);

        // State for status polling
        let ptype_clone = ptype.clone();
        let status_label_clone = status_label.clone();
        let role_combo_clone = role_combo.clone();
        let runtime_status = Runtime::new().unwrap();
        main_context.spawn_local(async move {
            loop {
                let role = if role_combo_clone.active() == Some(1) { Role::Sender } else { Role::Receiver };
                let status = runtime_status.block_on(protocol_manager::get_protocol_status(ptype_clone, role)).clone();
                let text = if status.running {
                    format!("Status: Running{}", status.port.map(|p| format!(" on :{}", p)).unwrap_or_default())
                } else if let Some(ref err) = status.error {
                    format!("Error: {}", err)
                } else {
                    "Status: Stopped".to_string()
                };
                status_label_clone.set_text(&text);
                glib::timeout_future_seconds(2).await;
            }
        });

        // Start button
        let ptype_start = ptype.clone();
        let role_combo_start = role_combo.clone();
        let port_entry_start = port_entry.clone();
        let extra_entry_start = extra_entry.clone();
        start_btn.connect_clicked(move |_| {
            let role = if role_combo_start.active() == Some(1) { Role::Sender } else { Role::Receiver };
            let port = port_entry_start.text().parse::<u16>().ok();
            let extra = if role == Role::Sender { Some(extra_entry_start.text().to_string()) } else { None };
            std::thread::spawn(move || {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let _ = runtime.block_on(protocol_manager::start_protocol(ptype_start, role, port, extra.as_deref()));
            });
        });
        // Stop button
        let ptype_stop = ptype.clone();
        let role_combo_stop = role_combo.clone();
        stop_btn.connect_clicked(move |_| {
            let role = if role_combo_stop.active() == Some(1) { Role::Sender } else { Role::Receiver };
            std::thread::spawn(move || {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let _ = runtime.block_on(protocol_manager::stop_protocol(ptype_stop, role));
            });
        });
    }

    // Device discovery UI
    let device_list = gtk::ListBox::new();
    vbox.append(&gtk::Label::new(Some("Discovered Devices:")));
    vbox.append(&device_list);
    let main_context = glib::MainContext::default();
    main_context.spawn_local(async move {
        loop {
            let wireless_devices = wireless::scan_wireless_displays().await.unwrap_or_default();
            let android_devices = android::scan_android_devices().await.unwrap_or_default();
            // Clear device_list
            while let Some(child) = device_list.first_child() {
                device_list.remove(&child);
            }
            for dev in wireless_devices {
                let row = gtk::ListBoxRow::new();
                let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                let label = gtk::Label::new(Some(&format!("Wireless: {} ({})", dev.name, dev.address)));
                let connect_btn = gtk::Button::with_label(if dev.connected { "Disconnect" } else { "Connect" });
                let dev_clone = dev.clone();
                connect_btn.connect_clicked(move |_| {
                    let dev = dev_clone.clone();
                    std::thread::spawn(move || {
                        let runtime = tokio::runtime::Runtime::new().unwrap();
                        if dev.connected {
                            let _ = runtime.block_on(wireless::disconnect_wireless_display(&dev.address));
                        } else {
                            let _ = runtime.block_on(wireless::connect_wireless_display(&dev.address));
                        }
                    });
                });
                hbox.append(&label);
                hbox.append(&connect_btn);
                row.set_child(Some(&hbox));
                device_list.append(&row);
            }
            for dev in android_devices {
                let row = gtk::ListBoxRow::new();
                let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                let label = gtk::Label::new(Some(&format!("Android: {} ({})", dev.name, dev.id)));
                let connect_btn = gtk::Button::with_label(if dev.connected { "Disconnect" } else { "Connect" });
                let dev_clone = dev.clone();
                connect_btn.connect_clicked(move |_| {
                    let dev = dev_clone.clone();
                    std::thread::spawn(move || {
                        let runtime = tokio::runtime::Runtime::new().unwrap();
                        if dev.connected {
                            let _ = runtime.block_on(android::disconnect_android_device(&dev.id));
                        } else {
                            let _ = runtime.block_on(android::connect_android_device(&dev.id));
                        }
                    });
                });
                hbox.append(&label);
                hbox.append(&connect_btn);
                row.set_child(Some(&hbox));
                device_list.append(&row);
            }
            glib::timeout_future_seconds(5).await;
        }
    });

    // Android device section
    let android_list = gtk::ListBox::new();
    vbox.append(&gtk::Label::new(Some("Android Devices:")));
    vbox.append(&android_list);
    let main_context = glib::MainContext::default();
    main_context.spawn_local(async move {
        loop {
            let android_devices = android::scan_android_devices().await.unwrap_or_default();
            clear_list_box(&android_list);
            for dev in android_devices {
                let row = gtk::ListBoxRow::new();
                let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                let label = gtk::Label::new(Some(&format!("Android: {} ({})", dev.name, dev.id)));
                let connect_btn = gtk::Button::with_label(if dev.connected { "Disconnect" } else { "Connect" });
                let dev_clone = dev.clone();
                connect_btn.connect_clicked(move |_| {
                    let dev = dev_clone.clone();
                    std::thread::spawn(move || {
                        let runtime = tokio::runtime::Runtime::new().unwrap();
                        let result = if dev.connected {
                            runtime.block_on(android::disconnect_android_device(&dev.id))
                        } else {
                            runtime.block_on(android::connect_android_device(&dev.id))
                        };
                        gtk::glib::MainContext::default().spawn_local(async move {
                            if let Err(e) = result {
                                crate::ui::toasts::show_toast(&format!("Android device error: {}", e));
                            } else {
                                crate::ui::toasts::show_toast("Android device action successful");
                            }
                        });
                    });
                });
                hbox.append(&label);
                hbox.append(&connect_btn);
                row.set_child(Some(&hbox));
                android_list.append(&row);
            }
            gtk::glib::timeout_future_seconds(5).await;
        }
    });

    clamp.set_child(Some(&vbox));
    content.append(&clamp);
}