use crate::ui::main_window::AppState;
use crate::display_manager::monitor_types::Monitor;
use crate::display_manager::monitor_control;
use gtk::prelude::*;
use gtk::{DropTarget, DragSource};
use gtk::gdk::{DragAction, ContentProvider};
use glib::value::ToValue;
use glib::types::Type;
use libadwaita as adw;
use std::sync::{Arc, Mutex};
use async_channel;

fn update_grid(
    grid: &gtk::Box,
    monitors: &[Monitor],
    sender: Arc<async_channel::Sender<Vec<Monitor>>>,
) {
    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }
    for (idx, monitor) in monitors.iter().enumerate() {
        let frame = gtk::Frame::new(Some(&monitor.name));
        frame.set_margin_bottom(8);
        frame.set_margin_top(8);
        frame.set_margin_start(8);
        frame.set_margin_end(8);
        frame.set_width_request(180);
        frame.set_height_request(120);
        frame.set_valign(gtk::Align::Center);
        frame.set_halign(gtk::Align::Center);

        // Status badges
        let badge_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        if monitor.focused {
            let badge = adw::Avatar::new(16, Some("Focused"), false);
            badge.set_icon_name(Some("starred-symbolic"));
            badge_box.append(&badge);
        }
        if monitor.dpms_status {
            let badge = adw::Avatar::new(16, Some("DPMS On"), false);
            badge.set_icon_name(Some("display-brightness-symbolic"));
            badge_box.append(&badge);
        }
        if monitor.vrr {
            let badge = adw::Avatar::new(16, Some("VRR"), false);
            badge.set_icon_name(Some("media-playback-start-symbolic"));
            badge_box.append(&badge);
        }
        frame.set_label_align(0.5);
        frame.set_child(Some(&badge_box));

        // Popover for settings (async backend calls)
        let popover_btn = gtk::Button::with_label("Settings");
        let popover = gtk::Popover::new();
        let popover_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        let res_label = gtk::Label::new(Some(&format!("{}x{} @ {}Hz", monitor.width, monitor.height, monitor.refresh_rate)));
        let scale_spin = gtk::SpinButton::with_range(0.5, 3.0, 0.01);
        scale_spin.set_value(monitor.scaling.unwrap_or(monitor.scale));
        let brightness_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.01);
        brightness_scale.set_value(monitor.brightness.unwrap_or(1.0));
        let orientation_combo = gtk::ComboBoxText::new();
        orientation_combo.append_text("Landscape");
        orientation_combo.append_text("Portrait");
        orientation_combo.set_active(match monitor.orientation.as_deref() { Some("Portrait") => Some(1), _ => Some(0) });
        popover_box.append(&res_label);
        popover_box.append(&gtk::Label::new(Some("Scale:")));
        popover_box.append(&scale_spin);
        popover_box.append(&gtk::Label::new(Some("Brightness:")));
        popover_box.append(&brightness_scale);
        popover_box.append(&gtk::Label::new(Some("Orientation:")));
        popover_box.append(&orientation_combo);
        popover.set_child(Some(&popover_box));
        popover.set_parent(&popover_btn);
        let monitor_for_scale = monitor.clone();
        let sender_for_scale = sender.clone();
        scale_spin.connect_value_changed(move |spin| {
            let value = spin.value();
            let mut updated = monitor_for_scale.clone();
            updated.scaling = Some(value);
            let sender = sender_for_scale.clone();
            gtk::glib::MainContext::default().spawn_local(async move {
                let result = monitor_control::apply_monitor_layout(&[updated.clone()]).await;
                let monitors = monitor_control::get_monitors().await.unwrap_or_default();
                sender.send(monitors).await.ok();
                if let Err(e) = result {
                    crate::ui::toasts::show_toast(&format!("Monitor layout error: {}", e));
                } else {
                    crate::ui::toasts::show_toast("Monitor layout applied");
                }
            });
        });
        let monitor_for_brightness = monitor.clone();
        let sender_for_brightness = sender.clone();
        brightness_scale.connect_value_changed(move |scale| {
            let value = scale.value();
            let monitor = monitor_for_brightness.clone();
            let sender = sender_for_brightness.clone();
            gtk::glib::MainContext::default().spawn_local(async move {
                let result = monitor_control::set_brightness(&monitor, value).await;
                let monitors = monitor_control::get_monitors().await.unwrap_or_default();
                sender.send(monitors).await.ok();
                if let Err(e) = result {
                    crate::ui::toasts::show_toast(&format!("Brightness error: {}", e));
                } else {
                    crate::ui::toasts::show_toast("Brightness set");
                }
            });
        });
        let monitor_for_orientation = monitor.clone();
        let sender_for_orientation = sender.clone();
        orientation_combo.connect_changed(move |combo| {
            let orientation = match combo.active() {
                Some(1) => "Portrait",
                _ => "Landscape",
            };
            let monitor = monitor_for_orientation.clone();
            let sender = sender_for_orientation.clone();
            gtk::glib::MainContext::default().spawn_local(async move {
                let result = monitor_control::set_rotation(&monitor, orientation).await;
                let monitors = monitor_control::get_monitors().await.unwrap_or_default();
                sender.send(monitors).await.ok();
                if let Err(e) = result {
                    crate::ui::toasts::show_toast(&format!("Rotation error: {}", e));
                } else {
                    crate::ui::toasts::show_toast("Rotation set");
                }
            });
        });
        popover_btn.connect_clicked(move |_| {
            popover.popup();
        });
        frame.set_child(Some(&popover_btn));

        // --- Drag-and-drop support ---
        let drag_source = DragSource::new();
        drag_source.set_actions(DragAction::MOVE);
        frame.add_controller(drag_source.clone());

        let drop_target = DropTarget::new(Type::STRING, DragAction::MOVE);
        frame.add_controller(drop_target.clone());

        drag_source.connect_prepare(move |_, _, _| {
            Some(ContentProvider::for_value(&idx.to_string().to_value()))
        });

        let monitors_vec = monitors.to_vec();
        let sender_clone = sender.clone();
        drop_target.connect_drop(move |_, value, _, _| {
            if let Ok(target_idx) = value.get::<String>() {
                if let Ok(target_idx) = target_idx.parse::<usize>() {
                    let mut new_order = monitors_vec.clone();
                    let monitor = new_order.remove(idx);
                    new_order.insert(target_idx, monitor);
                    let sender_clone = sender_clone.clone();
                    gtk::glib::MainContext::default().spawn_local(async move {
                        if let Err(e) = monitor_control::apply_monitor_layout(&new_order).await {
                            crate::ui::toasts::show_toast(&format!("Monitor layout error: {}", e));
                        } else {
                            crate::ui::toasts::show_toast("Monitor layout applied");
                            sender_clone.send(new_order).await.ok();
                        }
                    });
                }
            }
            true
        });

        grid.append(&frame);
    }
}

pub fn add_monitor_arrangement_section(content: &gtk::Box, _app_state: AppState) {
    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(900);
    clamp.set_margin_top(24);
    clamp.set_margin_bottom(24);
    clamp.set_margin_start(24);
    clamp.set_margin_end(24);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 16);
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&gtk::Label::new(Some("Monitor Arrangement"))));
    vbox.append(&header);

    // Monitor grid
    let grid = gtk::Box::new(gtk::Orientation::Horizontal, 24);
    vbox.append(&grid);

    // Buttons
    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let apply_btn = gtk::Button::with_label("Apply Layout");
    let reset_btn = gtk::Button::with_label("Reset");
    btn_box.append(&apply_btn);
    btn_box.append(&reset_btn);
    vbox.append(&btn_box);

    clamp.set_child(Some(&vbox));
    content.append(&clamp);

    // State for monitors (thread-safe)
    let monitors_state: Arc<Mutex<Vec<Monitor>>> = Arc::new(Mutex::new(vec![]));
    let grid_clone = grid.clone();

    // Use async-channel for thread-to-main communication
    let (sender, receiver) = async_channel::unbounded::<Vec<Monitor>>();
    let sender = Arc::new(sender);
    let sender_for_grid = sender.clone();
    gtk::glib::MainContext::default().spawn_local(async move {
        while let Ok(monitors) = receiver.recv().await {
            update_grid(&grid_clone, &monitors, sender_for_grid.clone());
        }
    });

    // Initial load: async, non-blocking
    let monitors_state_clone = Arc::clone(&monitors_state);
    let sender_clone2 = sender.clone();
    gtk::glib::MainContext::default().spawn_local(async move {
        match monitor_control::get_monitors().await {
            Ok(monitors) => {
                {
                    let mut guard = monitors_state_clone.lock().unwrap();
                    guard.clear();
                    guard.extend(monitors.clone());
                }
                sender_clone2.send(monitors).await.expect("Failed to send monitors");
            }
            Err(e) => {
                crate::ui::toasts::show_toast(&format!("Failed to load monitors: {}", e));
            }
        }
    });

    // Apply button (async)
    let monitors_state_apply = Arc::clone(&monitors_state);
    apply_btn.connect_clicked(move |_| {
        let monitors = {
            let guard = monitors_state_apply.lock().unwrap();
            guard.clone()
        };
        gtk::glib::MainContext::default().spawn_local(async move {
            let _ = monitor_control::apply_monitor_layout(&monitors).await;
        });
    });

    // Reset button (async)
    let monitors_state_reset = Arc::clone(&monitors_state);
    let grid_reset = grid.clone();
    let (reset_sender, reset_receiver) = async_channel::unbounded::<Vec<Monitor>>();
    let reset_sender = Arc::new(reset_sender);
    let reset_sender_for_async = reset_sender.clone();
    let reset_sender_for_btn = reset_sender.clone();
    gtk::glib::MainContext::default().spawn_local(async move {
        while let Ok(monitors) = reset_receiver.recv().await {
            update_grid(&grid_reset, &monitors, reset_sender_for_async.clone());
        }
    });
    reset_btn.connect_clicked(move |_| {
        let monitors_state_reset = Arc::clone(&monitors_state_reset);
        let reset_sender_clone2 = reset_sender_for_btn.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            match monitor_control::get_monitors().await {
                Ok(monitors) => {
                    {
                        let mut guard = monitors_state_reset.lock().unwrap();
                        guard.clear();
                        guard.extend(monitors.clone());
                    }
                    reset_sender_clone2.send(monitors).await.expect("Failed to send monitors");
                }
                Err(e) => {
                    crate::ui::toasts::show_toast(&format!("Failed to reset monitors: {}", e));
                }
            }
        });
    });
}