use gtk::prelude::*;
use std::sync::Arc;
use crate::settings::Settings;
use crate::recorder;
use libadwaita as adw;
use gtk::glib;
use std::sync::mpsc;

pub fn add_recording_controls_section(content: &gtk::Box, settings: Arc<Settings>) {
    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(700);
    clamp.set_margin_top(24);
    clamp.set_margin_bottom(24);
    clamp.set_margin_start(24);
    clamp.set_margin_end(24);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 16);
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&gtk::Label::new(Some("Screen Recording"))));
    vbox.append(&header);

    // Quick Record Card
    let quick_frame = gtk::Frame::new(Some("Quick Record"));
    quick_frame.set_width_request(320);
    quick_frame.set_height_request(120);
    let quick_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let start_btn = gtk::Button::from_icon_name("media-record-symbolic");
    start_btn.set_tooltip_text(Some("Start Recording"));
    let stop_btn = gtk::Button::from_icon_name("media-playback-stop-symbolic");
    stop_btn.set_tooltip_text(Some("Stop Recording"));
    let status_icon = gtk::Image::from_icon_name("media-record-symbolic");
    let status_label = gtk::Label::new(Some("Idle"));
    let timer_label = gtk::Label::new(Some("00:00:00"));
    quick_box.append(&start_btn);
    quick_box.append(&stop_btn);
    quick_box.append(&status_icon);
    quick_box.append(&status_label);
    quick_box.append(&timer_label);
    quick_frame.set_child(Some(&quick_box));
    vbox.append(&quick_frame);

    // Advanced Settings (collapsible)
    let expander = gtk::Expander::new(Some("Advanced Settings"));
    let adv_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let format_combo = gtk::ComboBoxText::new();
    format_combo.append_text("mp4");
    format_combo.append_text("mkv");
    format_combo.append_text("webm");
    format_combo.set_active(Some(0));
    let codec_combo = gtk::ComboBoxText::new();
    codec_combo.append_text("libx264");
    codec_combo.append_text("vp9");
    codec_combo.append_text("libx265");
    codec_combo.set_active(Some(0));
    let quality_combo = gtk::ComboBoxText::new();
    quality_combo.append_text("High");
    quality_combo.append_text("Medium");
    quality_combo.append_text("Low");
    quality_combo.append_text("Custom");
    quality_combo.set_active(Some(0));
    let quality_entry = gtk::Entry::new();
    quality_entry.set_placeholder_text(Some("Bitrate (e.g. 8000k)"));
    quality_entry.set_sensitive(false);
    // Best practice: For widgets that don't implement Downgrade, move clones into closure
    let quality_entry_clone = quality_entry.clone();
    quality_combo.connect_changed(move |combo| {
        let custom = combo.active_text().map(|t| t == "Custom").unwrap_or(false);
        quality_entry_clone.set_sensitive(custom);
    });
    let audio_combo = gtk::ComboBoxText::new();
    audio_combo.append_text("None");
    audio_combo.append_text("Default");
    audio_combo.append_text("Monitor of Built-in Audio");
    audio_combo.append_text("Built-in Audio");
    audio_combo.set_active(Some(1));
    let advanced_entry = gtk::Entry::new();
    advanced_entry.set_placeholder_text(Some("Extra encoder args (optional)"));
    let framerate_entry = gtk::SpinButton::with_range(1.0, 240.0, 1.0);
    framerate_entry.set_value(settings.recorder_framerate.unwrap_or(30) as f64);
    let resolution_entry = gtk::Entry::new();
    resolution_entry.set_placeholder_text(Some("e.g. 1920x1080"));
    if let Some(ref res) = settings.recorder_resolution {
        resolution_entry.set_text(res);
    }
    let hwaccel_switch = gtk::Switch::new();
    hwaccel_switch.set_active(settings.recorder_hardware_accel.unwrap_or(false));
    let audio_device_entry = gtk::Entry::new();
    audio_device_entry.set_placeholder_text(Some("Audio device (optional)"));
    if let Some(ref dev) = settings.recorder_audio_device {
        audio_device_entry.set_text(dev);
    }
    adv_box.append(&gtk::Label::new(Some("Format:")));
    adv_box.append(&format_combo);
    adv_box.append(&gtk::Label::new(Some("Codec:")));
    adv_box.append(&codec_combo);
    adv_box.append(&gtk::Label::new(Some("Quality:")));
    adv_box.append(&quality_combo);
    adv_box.append(&quality_entry);
    adv_box.append(&gtk::Label::new(Some("Audio:")));
    adv_box.append(&audio_combo);
    adv_box.append(&advanced_entry);
    adv_box.append(&gtk::Label::new(Some("Framerate:")));
    adv_box.append(&framerate_entry);
    adv_box.append(&gtk::Label::new(Some("Resolution:")));
    adv_box.append(&resolution_entry);
    adv_box.append(&gtk::Label::new(Some("HWAccel:")));
    adv_box.append(&hwaccel_switch);
    adv_box.append(&audio_device_entry);
    expander.set_child(Some(&adv_box));
    vbox.append(&expander);

    clamp.set_child(Some(&vbox));
    content.append(&clamp);

    // Start/Stop recording logic
    let status_label_clone = status_label.clone();
    let timer_label_clone = timer_label.clone();
    start_btn.connect_clicked({
        let settings = settings.clone();
        let status_label = status_label_clone.clone();
        let timer_label = timer_label_clone.clone();
        move |_| {
            let (tx, rx) = mpsc::channel();
            let settings = settings.clone();
            std::thread::spawn(move || {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let result = runtime.block_on(recorder::start_recording(&settings));
                tx.send(result).unwrap();
            });
            let status_label = status_label.clone();
            let timer_label = timer_label.clone();
            glib::idle_add_local(move || {
                if let Ok(result) = rx.try_recv() {
                    if let Err(e) = result {
                        crate::ui::toasts::show_toast(&format!("Recording error: {}", e));
                        status_label.set_text("Idle");
                    } else {
                        crate::ui::toasts::show_toast("Recording started");
                        status_label.set_text("Recording");
                        timer_label.set_text("00:00:01");
                    }
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            });
        }
    });
    let status_label_clone = status_label.clone();
    let timer_label_clone = timer_label.clone();
    stop_btn.connect_clicked({
        let status_label = status_label_clone.clone();
        let timer_label = timer_label_clone.clone();
        move |_| {
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let result = runtime.block_on(recorder::stop_recording());
                tx.send(result).unwrap();
            });
            let status_label = status_label.clone();
            let timer_label = timer_label.clone();
            glib::idle_add_local(move || {
                if let Ok(result) = rx.try_recv() {
                    if let Err(e) = result {
                        crate::ui::toasts::show_toast(&format!("Stop error: {}", e));
                    } else {
                        crate::ui::toasts::show_toast("Recording stopped");
                    }
                    status_label.set_text("Idle");
                    timer_label.set_text("00:00:00");
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            });
        }
    });
} 