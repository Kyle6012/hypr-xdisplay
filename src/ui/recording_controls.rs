use gtk::prelude::*;
use std::sync::Arc;
use crate::settings::Settings;
use gtk::{self, glib};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;
use crate::recorder;
use glib::ControlFlow::Continue;
use glib::clone;

pub fn add_recording_controls_section(content: &gtk::Box, settings: Arc<Settings>) {
    let recording_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    recording_box.set_halign(gtk::Align::Center);
    let recording_label = gtk::Label::builder().label("<b>Screen Recording</b>").use_markup(true).build();

    // --- Control Buttons ---
    let start_btn = gtk::Button::with_label("Start");
    let pause_btn = gtk::Button::with_label("Pause");
    let resume_btn = gtk::Button::with_label("Resume");
    let stop_btn = gtk::Button::with_label("Stop");
    recording_box.append(&start_btn);
    recording_box.append(&pause_btn);
    recording_box.append(&resume_btn);
    recording_box.append(&stop_btn);

    // --- Format, Codec, Quality ---
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
    // Show entry only if Custom is selected
    quality_combo.connect_changed(clone!(@weak quality_entry => move |combo| {
        let custom = combo.active_text().map(|t| t == "Custom").unwrap_or(false);
        quality_entry.set_sensitive(custom);
    }));

    // --- Audio Source ---
    let audio_combo = gtk::ComboBoxText::new();
    audio_combo.append_text("None");
    audio_combo.append_text("Default");
    audio_combo.append_text("Monitor of Built-in Audio");
    audio_combo.append_text("Built-in Audio");
    audio_combo.set_active(Some(1));

    // --- Advanced Encoder Settings ---
    let advanced_entry = gtk::Entry::new();
    advanced_entry.set_placeholder_text(Some("Extra encoder args (optional)"));

    // --- Preview & Last Recording ---
    let preview_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    preview_box.set_halign(gtk::Align::Center);
    let last_label = gtk::Label::new(Some("Last Recording: None"));
    let preview_btn = gtk::Button::with_label("Preview");
    preview_box.append(&last_label);
    preview_box.append(&preview_btn);

    // --- Toast overlay ---
    let toast_overlay = adw::ToastOverlay::new();
    content.append(&toast_overlay);

    // --- Timer label ---
    let timer_label = gtk::Label::new(Some("00:00:00"));
    timer_label.set_halign(gtk::Align::Center);
    content.append(&timer_label);

    // --- Layout ---
    let settings_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    settings_box.append(&gtk::Label::new(Some("Format:")));
    settings_box.append(&format_combo);
    settings_box.append(&gtk::Label::new(Some("Codec:")));
    settings_box.append(&codec_combo);
    settings_box.append(&gtk::Label::new(Some("Quality:")));
    settings_box.append(&quality_combo);
    settings_box.append(&quality_entry);
    settings_box.append(&gtk::Label::new(Some("Audio:")));
    settings_box.append(&audio_combo);
    settings_box.append(&advanced_entry);

    // --- Advanced Options ---
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
    // Add to settings_box
    settings_box.append(&gtk::Label::new(Some("Framerate:")));
    settings_box.append(&framerate_entry);
    settings_box.append(&gtk::Label::new(Some("Resolution:")));
    settings_box.append(&resolution_entry);
    settings_box.append(&gtk::Label::new(Some("HWAccel:")));
    settings_box.append(&hwaccel_switch);
    settings_box.append(&audio_device_entry);

    content.append(&recording_label);
    content.append(&recording_box);
    content.append(&settings_box);
    content.append(&preview_box);

    // --- Load settings into UI ---
    let recorder_format = settings.recorder_filename_format.split('.').last().unwrap_or("mp4");
    format_combo.set_active_id(Some(recorder_format));
    codec_combo.set_active_id(Some(&settings.recorder_codec));
    // Quality/bitrate: try to match, else default to High
    if settings.recorder_filename_format.contains("high") {
        quality_combo.set_active(Some(0));
    } else if settings.recorder_filename_format.contains("medium") {
        quality_combo.set_active(Some(1));
    } else if settings.recorder_filename_format.contains("low") {
        quality_combo.set_active(Some(2));
    } else {
        quality_combo.set_active(Some(0));
    }
    // Audio source
    if let Some(ref audio) = settings.recorder_audio_source {
        for (i, opt) in ["None", "Default", "Monitor of Built-in Audio", "Built-in Audio"].iter().enumerate() {
            if audio == opt {
                audio_combo.set_active(Some(i as u32));
                break;
            }
        }
    }
    // Advanced args
    if let Some(ref adv) = settings.recorder_advanced_args {
        advanced_entry.set_text(adv);
    }
    // Framerate
    framerate_entry.set_value(settings.recorder_framerate.unwrap_or(30) as f64);
    // Resolution
    if let Some(ref res) = settings.recorder_resolution {
        resolution_entry.set_text(res);
    }
    // HWAccel
    hwaccel_switch.set_active(settings.recorder_hardware_accel.unwrap_or(false));
    // Audio device
    if let Some(ref dev) = settings.recorder_audio_device {
        audio_device_entry.set_text(dev);
    }

    // --- Save settings on change ---
    let settings_clone = settings.clone();
    format_combo.connect_changed(move |combo| {
        if let Some(fmt) = combo.active_text() {
            let mut s = (*settings_clone).clone();
            s.recorder_filename_format = format!("recording_%Y-%m-%d_%H-%M-%S.{}", fmt);
            let _ = s.save();
        }
    });
    let settings_clone = settings.clone();
    codec_combo.connect_changed(move |combo| {
        if let Some(codec) = combo.active_text() {
            let mut s = (*settings_clone).clone();
            s.recorder_codec = codec.to_string();
            let _ = s.save();
        }
    });
    let settings_clone = settings.clone();
    quality_combo.connect_changed(move |combo| {
        let mut s = (*settings_clone).clone();
        let idx = combo.active().unwrap_or(0);
        let quality = match idx {
            0 => "high",
            1 => "medium",
            2 => "low",
            3 => quality_entry.text().as_str(),
            _ => "high",
        };
        s.recorder_filename_format = format!("recording_%Y-%m-%d_%H-%M-%S_{}.mp4", quality);
        let _ = s.save();
    });
    let settings_clone = settings.clone();
    audio_combo.connect_changed(move |combo| {
        if let Some(audio) = combo.active_text() {
            let mut s = (*settings_clone).clone();
            s.recorder_audio_source = Some(audio.to_string());
            let _ = s.save();
        }
    });
    let settings_clone = settings.clone();
    advanced_entry.connect_changed(move |entry| {
        let mut s = (*settings_clone).clone();
        s.recorder_advanced_args = Some(entry.text().to_string());
        let _ = s.save();
    });
    let settings_clone = settings.clone();
    framerate_entry.connect_value_changed(move |spin| {
        let mut s = (*settings_clone).clone();
        s.recorder_framerate = Some(spin.value() as u32);
        let _ = s.save();
    });
    let settings_clone = settings.clone();
    resolution_entry.connect_changed(move |entry| {
        let mut s = (*settings_clone).clone();
        s.recorder_resolution = Some(entry.text().to_string());
        let _ = s.save();
    });
    let settings_clone = settings.clone();
    hwaccel_switch.connect_state_set(move |sw, state| {
        let mut s = (*settings_clone).clone();
        s.recorder_hardware_accel = Some(state);
        let _ = s.save();
        Continue
    });
    let settings_clone = settings.clone();
    audio_device_entry.connect_changed(move |entry| {
        let mut s = (*settings_clone).clone();
        s.recorder_audio_device = Some(entry.text().to_string());
        let _ = s.save();
    });
    // TODO: Save audio and advanced settings on change

    // --- State ---
    let last_recording: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));
    let is_recording = Rc::new(RefCell::new(false));
    let is_paused = Rc::new(RefCell::new(false));
    let elapsed_seconds = Rc::new(RefCell::new(0u64));
    // Timer update
    {
        let timer_label = timer_label.clone();
        let is_recording = is_recording.clone();
        let is_paused = is_paused.clone();
        let elapsed_seconds = elapsed_seconds.clone();
        glib::timeout_add_seconds_local(1, move || {
            if *is_recording.borrow() && !*is_paused.borrow() {
                *elapsed_seconds.borrow_mut() += 1;
                let secs = *elapsed_seconds.borrow();
                let h = secs / 3600;
                let m = (secs % 3600) / 60;
                let s = secs % 60;
                timer_label.set_text(&format!("{:02}:{:02}:{:02}", h, m, s));
            }
            Continue
        });
    }
    // --- Button state logic ---
    let update_buttons = {
        let start_btn = start_btn.clone();
        let pause_btn = pause_btn.clone();
        let resume_btn = resume_btn.clone();
        let stop_btn = stop_btn.clone();
        let is_recording = is_recording.clone();
        let is_paused = is_paused.clone();
        move || {
            let rec = *is_recording.borrow();
            let paused = *is_paused.borrow();
            start_btn.set_sensitive(!rec);
            pause_btn.set_sensitive(rec && !paused);
            resume_btn.set_sensitive(rec && paused);
            stop_btn.set_sensitive(rec);
        }
    };
    update_buttons();
    // --- Button handlers update state ---
    let is_recording_clone = is_recording.clone();
    let is_paused_clone = is_paused.clone();
    let elapsed_seconds_clone = elapsed_seconds.clone();
    let update_buttons_clone = update_buttons.clone();
    start_btn.connect_clicked(move |_| {
        *is_recording_clone.borrow_mut() = true;
        *is_paused_clone.borrow_mut() = false;
        *elapsed_seconds_clone.borrow_mut() = 0;
        update_buttons_clone();
    });
    let is_paused_clone = is_paused.clone();
    let update_buttons_clone = update_buttons.clone();
    pause_btn.connect_clicked(move |_| {
        *is_paused_clone.borrow_mut() = true;
        update_buttons_clone();
    });
    let is_paused_clone = is_paused.clone();
    let update_buttons_clone = update_buttons.clone();
    resume_btn.connect_clicked(move |_| {
        *is_paused_clone.borrow_mut() = false;
        update_buttons_clone();
    });
    let is_recording_clone = is_recording.clone();
    let is_paused_clone = is_paused.clone();
    let update_buttons_clone = update_buttons.clone();
    stop_btn.connect_clicked(move |_| {
        *is_recording_clone.borrow_mut() = false;
        *is_paused_clone.borrow_mut() = false;
        update_buttons_clone();
    });

    // --- Button handlers ---
    let settings_clone = settings.clone();
    let last_recording_clone = last_recording.clone();
    let last_label_clone = last_label.clone();
    let toast_overlay_clone = toast_overlay.clone();
    start_btn.connect_clicked(move |_| {
        let settings = settings_clone.clone();
        let last_recording = last_recording_clone.clone();
        let last_label = last_label_clone.clone();
        let toast_overlay = toast_overlay_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            match recorder::start_recording(settings.as_ref()).await {
                Ok(_) => {
                    let toast = adw::Toast::builder().title("Recording started").timeout(3).build();
                    toast_overlay.add_toast(toast);
                },
                Err(e) => {
                    let toast = adw::Toast::builder().title(&format!("Failed to start: {}", e)).timeout(5).build();
                    toast_overlay.add_toast(toast);
                }
            }
        });
    });
    let toast_overlay_clone = toast_overlay.clone();
    stop_btn.connect_clicked(move |_| {
        let last_recording = last_recording.clone();
        let last_label = last_label.clone();
        let toast_overlay = toast_overlay_clone.clone();
        let settings = settings.clone();
        glib::MainContext::default().spawn_local(async move {
            match recorder::stop_recording().await {
                Ok(_) => {
                    // Find the most recent file in the recording dir
                    let dir = &settings.recorder_dir;
                    if let Ok(mut entries) = std::fs::read_dir(dir) {
                        let mut files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                        files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
                        if let Some(entry) = files.last() {
                            let path = entry.path();
                            *last_recording.borrow_mut() = Some(path.clone());
                            last_label.set_text(&format!("Last Recording: {}", path.file_name().unwrap().to_string_lossy()));
                        }
                    }
                    let toast = adw::Toast::builder().title("Recording stopped").timeout(3).build();
                    toast_overlay.add_toast(toast);
                },
                Err(e) => {
                    let toast = adw::Toast::builder().title(&format!("Failed to stop: {}", e)).timeout(5).build();
                    toast_overlay.add_toast(toast);
                }
            }
        });
    });
    let toast_overlay_clone = toast_overlay.clone();
    pause_btn.connect_clicked(move |_| {
        let toast_overlay = toast_overlay_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            match recorder::pause_recording().await {
                Ok(_) => {
                    let toast = adw::Toast::builder().title("Recording paused").timeout(3).build();
                    toast_overlay.add_toast(toast);
                },
                Err(e) => {
                    let toast = adw::Toast::builder().title(&format!("Failed to pause: {}", e)).timeout(5).build();
                    toast_overlay.add_toast(toast);
                }
            }
        });
    });
    let toast_overlay_clone = toast_overlay.clone();
    resume_btn.connect_clicked(move |_| {
        let toast_overlay = toast_overlay_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            match recorder::resume_recording().await {
                Ok(_) => {
                    let toast = adw::Toast::builder().title("Recording resumed").timeout(3).build();
                    toast_overlay.add_toast(toast);
                },
                Err(e) => {
                    let toast = adw::Toast::builder().title(&format!("Failed to resume: {}", e)).timeout(5).build();
                    toast_overlay.add_toast(toast);
                }
            }
        });
    });

    // TODO: Wire up start, pause, resume, stop to backend and update last_recording, last_label, toasts, and settings as needed.
} 