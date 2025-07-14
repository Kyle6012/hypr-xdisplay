use libadwaita as adw;
use adw::prelude::*;
use std::sync::{Arc, Mutex};
use crate::settings::Settings;
use crate::ui::{monitor_arrangement, network_settings, toasts, status_polling, screenshot_controls, recording_controls};
use std::cell::RefCell;
use std::rc::Rc;
use gtk::gdk;
use crate::screenshot;
use crate::recorder;
use crate::display_manager::monitor_types::Monitor;
use gtk::gio;

thread_local! {
    pub static MAIN_WINDOW: RefCell<Option<adw::ApplicationWindow>> = RefCell::new(None);
}

pub type AppState = Arc<Mutex<Vec<Monitor>>>;

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

    // --- App Logo ---
    let logo = gtk::Picture::for_file(&gio::File::for_path("img.png"));
    logo.set_halign(gtk::Align::Center);
    content.append(&logo);

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

    // --- Global PrintScreen handler ---
    let toast_overlay = adw::ToastOverlay::new();
    content.append(&toast_overlay);
    window.add_controller({
        let key_controller = gtk::EventControllerKey::new();
        let overlay_for_screenshot = toast_overlay.clone();
        let settings_for_screenshot = settings.clone();
        let overlay_for_recording = toast_overlay.clone();
        let settings_for_recording = settings.clone();
        key_controller.connect_key_pressed(move |_, keyval, _keycode, _state| {
            if keyval == gdk::Key::Print.into() {
                let _show_screenshot = {
                    let settings = settings_for_screenshot.clone();
                    let toast_overlay = overlay_for_screenshot.clone();
                    move |_mode: &'static str| {
                        let settings = settings.clone();
                        let toast_overlay = toast_overlay.clone();
                        glib::MainContext::default().spawn_local(async move {
                            let result = match _mode {
                                "fullscreen" => screenshot::capture_fullscreen(settings.as_ref()).await,
                                "region" => screenshot::capture_region(settings.as_ref()).await,
                                "window" => screenshot::capture_focused_window(settings.as_ref()).await,
                                _ => return,
                            };
                            match result {
                                Ok(_path) => {
                                    let toast = adw::Toast::builder().title("Screenshot saved").timeout(3).build();
                                    toast_overlay.add_toast(toast);
                                },
                                Err(e) => {
                                    let toast = adw::Toast::builder().title(&format!("Screenshot failed: {}", e)).timeout(5).build();
                                    toast_overlay.add_toast(toast);
                                }
                            }
                        });
                    }
                };
                let _start_recording = {
                    let settings = settings_for_recording.clone();
                    let toast_overlay = overlay_for_recording.clone();
                    move |_mode: &'static str| {
                        let settings = settings.clone();
                        let toast_overlay = toast_overlay.clone();
                        glib::MainContext::default().spawn_local(async move {
                            let result = recorder::start_recording(settings.as_ref()).await;
                            match result {
                                Ok(_) => {
                                    let toast = adw::Toast::builder().title("Recording started").timeout(3).build();
                                    toast_overlay.add_toast(toast);
                                },
                                Err(e) => {
                                    let toast = adw::Toast::builder().title(&format!("Recording failed: {}", e)).timeout(5).build();
                                    toast_overlay.add_toast(toast);
                                }
                            }
                        });
                    }
                };
                let toast = adw::Toast::builder()
                    .title("Choose action: Screenshot or Screen Record?")
                    .timeout(6)
                    .button_label("Screenshot")
                    // .button_action(...) // Not supported in this version
                    .build();
                overlay_for_screenshot.add_toast(toast);
                return true.into();
            }
            false.into()
        });
        key_controller
    });

    window.set_content(Some(&content));
    window.show();
} 