use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::sync::{Arc, Mutex};
use crate::settings::Settings;
use crate::ui::{monitor_arrangement, network_settings, toasts, status_polling, screenshot_controls, recording_controls};
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    pub static MAIN_WINDOW: RefCell<Option<adw::ApplicationWindow>> = RefCell::new(None);
}

pub type AppState = Arc<Mutex<Vec<crate::display_manager::Monitor>>>;

#[derive(Clone)]
pub struct AppDevices {
    pub wireless: Rc<RefCell<Vec<crate::wireless::WirelessDisplay>>>,
    pub android: Rc<RefCell<Vec<crate::android::AndroidDevice>>>,
}

pub fn build_ui(app: &adw::Application, app_state: AppState, settings: Arc<Settings>) {
    let window = adw::ApplicationWindow::new(app);
    window.set_title(Some("Hypr-XDisplay Manager"));
    window.set_default_size(800, 600);
    MAIN_WINDOW.with(|w| *w.borrow_mut() = Some(window.clone()));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    // Devices state
    let devices = AppDevices {
        wireless: Rc::new(RefCell::new(vec![])),
        android: Rc::new(RefCell::new(vec![])),
    };

    // --- Compose UI sections from submodules ---
    monitor_arrangement::add_monitor_arrangement_section(&content, app_state.clone());
    network_settings::add_network_settings_section(&content, app_state.clone(), settings.clone(), devices.clone());
    screenshot_controls::add_screenshot_controls_section(&content, settings.clone());
    recording_controls::add_recording_controls_section(&content, settings.clone());
    toasts::setup_toast_overlay(&window, &content);
    status_polling::setup_status_polling(&window, app_state.clone());

    window.set_content(Some(&content));
    window.show();
} 