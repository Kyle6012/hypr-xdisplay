use gtk::prelude::*;
use std::sync::{Arc, Mutex};
use crate::settings::Settings;
use crate::screenshot;
use libadwaita as adw;
use std::path::PathBuf;
use std::process::Command;

pub fn add_screenshot_controls_section(content: &gtk::Box, settings: Arc<Settings>) {
    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(700);
    clamp.set_margin_top(24);
    clamp.set_margin_bottom(24);
    clamp.set_margin_start(24);
    clamp.set_margin_end(24);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 16);
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&gtk::Label::new(Some("Screenshots"))));
    vbox.append(&header);

    // Screenshot history dropdown
    let history: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));
    let history_combo = gtk::ComboBoxText::new();
    history_combo.set_hexpand(false);
    history_combo.set_valign(gtk::Align::Center);
    history_combo.set_tooltip_text(Some("Screenshot history"));
    vbox.append(&history_combo);

    // Screenshot mode cards
    let modes = [
        ("Fullscreen", "view-fullscreen-symbolic"),
        ("Region", "select-rectangle-symbolic"),
        ("Window", "window-symbolic"),
    ];
    let mode_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    for (label, icon) in modes.iter() {
        let frame = gtk::Frame::new(Some(label));
        frame.set_width_request(160);
        frame.set_height_request(100);
        let btn = gtk::Button::from_icon_name(icon);
        btn.set_tooltip_text(Some(&format!("Capture {} screenshot", label)));
        frame.set_child(Some(&btn));
        let settings_clone = settings.clone();
        let history_clone = Arc::clone(&history);
        let history_combo_clone = history_combo.clone();
        let label = label.to_string();
        btn.connect_clicked(move |_| {
            let label = label.clone();
            let settings = settings_clone.clone();
            let history = Arc::clone(&history_clone);
            let history_combo = history_combo_clone.clone();
            glib::MainContext::default().spawn_local(async move {
                let result = match label.as_str() {
                    "Fullscreen" => screenshot::capture_fullscreen(settings.as_ref()).await,
                    "Region" => screenshot::capture_region(settings.as_ref()).await,
                    "Window" => screenshot::capture_focused_window(settings.as_ref()).await,
                    _ => return,
                };
                if let Ok(path) = result {
                    {
                        let mut guard = history.lock().unwrap();
                        guard.push(path.clone());
                        history_combo.append_text(path.file_name().unwrap().to_str().unwrap());
                        history_combo.set_active(Some((guard.len() - 1) as u32));
                    }
                }
            });
        });
        mode_box.append(&frame);
    }
    vbox.append(&mode_box);

    // Preview card
    let preview_frame = gtk::Frame::new(Some("Preview"));
    preview_frame.set_width_request(320);
    preview_frame.set_height_request(180);
    let preview_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let preview_picture = gtk::Picture::new();
    preview_picture.set_can_shrink(true);
    preview_picture.set_size_request(300, 160);
    preview_box.append(&preview_picture);
    let filename_label = gtk::Label::new(Some("No screenshot selected"));
    preview_box.append(&filename_label);
    // Action buttons
    let actions_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let open_btn = gtk::Button::from_icon_name("document-open-symbolic");
    open_btn.set_tooltip_text(Some("Open screenshot"));
    let annotate_btn = gtk::Button::from_icon_name("draw-freehand-symbolic");
    annotate_btn.set_tooltip_text(Some("Annotate screenshot"));
    let share_btn = gtk::Button::from_icon_name("mail-send-symbolic");
    share_btn.set_tooltip_text(Some("Share/Upload screenshot"));
    actions_box.append(&open_btn);
    actions_box.append(&annotate_btn);
    actions_box.append(&share_btn);
    preview_box.append(&actions_box);
    preview_frame.set_child(Some(&preview_box));
    vbox.append(&preview_frame);

    // Update preview on history change
    let history_for_combo = Arc::clone(&history);
    let preview_picture_clone = preview_picture.clone();
    let filename_label_clone = filename_label.clone();
    history_combo.connect_changed(move |combo| {
        if let Some(idx) = combo.active() {
            let guard = history_for_combo.lock().unwrap();
            if let Some(path) = guard.get(idx as usize) {
                preview_picture_clone.set_filename(Some(path.to_str().unwrap()));
                filename_label_clone.set_text(path.file_name().unwrap().to_str().unwrap());
            }
        }
    });

    // Open screenshot
    let history_for_open = Arc::clone(&history);
    let history_combo_for_open = history_combo.clone();
    open_btn.connect_clicked(move |_| {
        if let Some(idx) = history_combo_for_open.active() {
            let guard = history_for_open.lock().unwrap();
            if let Some(path) = guard.get(idx as usize) {
                let _ = Command::new("xdg-open").arg(path).spawn();
            }
        }
    });

    // Annotate screenshot (modal)
    let history_for_annotate = Arc::clone(&history);
    let history_combo_for_annotate = history_combo.clone();
    annotate_btn.connect_clicked(move |_| {
        if let Some(idx) = history_combo_for_annotate.active() {
            let guard = history_for_annotate.lock().unwrap();
            if let Some(path) = guard.get(idx as usize) {
                let _ = Command::new("swappy").arg("-f").arg(path).spawn();
            }
        }
    });

    // Share/upload screenshot (modal)
    let history_for_share = Arc::clone(&history);
    let history_combo_for_share = history_combo.clone();
    share_btn.connect_clicked(move |_| {
        if let Some(idx) = history_combo_for_share.active() {
            let guard = history_for_share.lock().unwrap();
            if let Some(path) = guard.get(idx as usize) {
                let _ = Command::new("wl-copy").arg(path).status();
            }
        }
    });

    clamp.set_child(Some(&vbox));
    content.append(&clamp);
} 