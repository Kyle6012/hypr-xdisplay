use libadwaita as adw;
use adw::prelude::*;
use std::sync::{Arc, Mutex};
use crate::settings::Settings;
use crate::ui::{monitor_arrangement, network_settings, toasts, screenshot_controls, recording_controls};
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    pub static MAIN_WINDOW: RefCell<Option<adw::ApplicationWindow>> = RefCell::new(None);
}

pub type AppState = Arc<Mutex<Vec<()>>>; // Replace Vec<()> with your Monitor type if needed

#[derive(Clone)]
pub struct AppDevices {
    pub wireless: Rc<RefCell<Vec<crate::wireless::WirelessDisplay>>>,
    pub android: Rc<RefCell<Vec<crate::android::AndroidDevice>>>,
}

// --- Helper: Each section returns its main widget ---
fn build_monitor_section(app_state: AppState) -> gtk::Box {
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 12);
    monitor_arrangement::add_monitor_arrangement_section(&box_, app_state);
    box_
}
fn build_network_section(app_state: AppState, settings: Arc<Settings>, devices: AppDevices) -> gtk::Box {
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 12);
    network_settings::add_network_settings_section(&box_, app_state, settings, devices);
    box_
}
fn build_screenshot_section(settings: Arc<Settings>) -> gtk::Box {
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 12);
    screenshot_controls::add_screenshot_controls_section(&box_, settings);
    box_
}
fn build_recording_section(settings: Arc<Settings>) -> gtk::Box {
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 12);
    recording_controls::add_recording_controls_section(&box_, settings);
    box_
}
// TODO: Add a real settings section if you have one
fn build_settings_section(settings: Arc<Settings>) -> gtk::Box {
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 12);
    let label = gtk::Label::new(Some("Settings"));
    box_.append(&label);
    // Screenshot directory
    let screenshot_dir_entry = gtk::Entry::new();
    screenshot_dir_entry.set_text(settings.screenshot_dir.to_str().unwrap_or(""));
    box_.append(&gtk::Label::new(Some("Screenshot Directory:")));
    box_.append(&screenshot_dir_entry);
    // Screenshot filename format
    let screenshot_fmt_entry = gtk::Entry::new();
    screenshot_fmt_entry.set_text(&settings.screenshot_filename_format);
    box_.append(&gtk::Label::new(Some("Screenshot Filename Format:")));
    box_.append(&screenshot_fmt_entry);
    // Recorder directory
    let recorder_dir_entry = gtk::Entry::new();
    recorder_dir_entry.set_text(settings.recorder_dir.to_str().unwrap_or(""));
    box_.append(&gtk::Label::new(Some("Recorder Directory:")));
    box_.append(&recorder_dir_entry);
    // Recorder filename format
    let recorder_fmt_entry = gtk::Entry::new();
    recorder_fmt_entry.set_text(&settings.recorder_filename_format);
    box_.append(&gtk::Label::new(Some("Recorder Filename Format:")));
    box_.append(&recorder_fmt_entry);
    // Save button
    let save_btn = gtk::Button::with_label("Save Settings");
    let settings_clone = settings.clone();
    save_btn.connect_clicked(move |_| {
        let mut new_settings = (*settings_clone).clone();
        new_settings.screenshot_dir = std::path::PathBuf::from(screenshot_dir_entry.text().to_string());
        new_settings.screenshot_filename_format = screenshot_fmt_entry.text().to_string();
        new_settings.recorder_dir = std::path::PathBuf::from(recorder_dir_entry.text().to_string());
        new_settings.recorder_filename_format = recorder_fmt_entry.text().to_string();
        let _ = new_settings.save();
    });
    box_.append(&save_btn);
    box_
}

pub fn build_ui(app: &adw::Application, app_state: AppState, settings: Arc<Settings>) {
    let window = adw::ApplicationWindow::new(app);
    window.set_title(Some("Hypr-XDisplay Manager"));
    window.set_default_size(1000, 700);
    MAIN_WINDOW.with(|w| *w.borrow_mut() = Some(window.clone()));

    // --- Tabbed layout ---
    let notebook = gtk::Notebook::new();
    notebook.set_hexpand(true);
    notebook.set_vexpand(true);

    // Devices state
    let devices = AppDevices {
        wireless: Rc::new(RefCell::new(vec![])),
        android: Rc::new(RefCell::new(vec![])),
    };

    // Add real sections as notebook pages
    let monitor_box = build_monitor_section(app_state.clone());
    let network_box = build_network_section(app_state.clone(), settings.clone(), devices.clone());
    let screenshot_box = build_screenshot_section(settings.clone());
    let recording_box = build_recording_section(settings.clone());
    let settings_box = build_settings_section(settings.clone());
    let section_titles = ["Monitors", "Network", "Screenshots", "Recording", "Settings"];
    let section_widgets: [&gtk::Box; 5] = [&monitor_box, &network_box, &screenshot_box, &recording_box, &settings_box];
    for (i, widget) in section_widgets.iter().enumerate() {
        notebook.append_page(widget.upcast_ref::<gtk::Widget>(), Some(&gtk::Label::new(Some(section_titles[i]))));
    }

    // --- Modal overlay logic using gtk::Dialog ---
    let overlay_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    overlay_box.append(&notebook);
    window.set_content(Some(&overlay_box));

    // Show modal dialog for each tab when selected
    let window_clone = window.clone();
    let notebook_clone = notebook.clone();
    notebook.connect_switch_page(move |_notebook, _page, page_num| {
        let idx = page_num as usize;
        if idx < section_titles.len() {
            let dialog = gtk::Dialog::new();
            dialog.set_transient_for(Some(&window_clone));
            dialog.set_modal(true);
            dialog.set_title(Some(section_titles[idx]));
            if let Some(page) = notebook_clone.nth_page(Some(page_num)) {
                let content_area = dialog.content_area();
                content_area.append(&page);
            }
            dialog.show();
            // When dialog is closed, return to previous tab (first tab)
            let notebook_clone2 = notebook_clone.clone();
            dialog.connect_close_request(move |_| {
                notebook_clone2.set_current_page(Some(0));
                gtk::glib::Propagation::Stop
            });
        }
    });

    // --- Toasts and status polling ---
    toasts::setup_toast_overlay(&window, &overlay_box);
    // status_polling::setup_status_polling(&window, app_state.clone());

    window.present();
} 