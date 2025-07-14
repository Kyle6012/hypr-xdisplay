use gtk::prelude::*;
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

    // --- App Logo ---
    let logo = gtk::Picture::from_file("img.png");
    logo.set_halign(gtk::Align::Center);
    logo.set_pixel_size(96);
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
    window.add_controller(&{
        let key_controller = gtk::EventControllerKey::new();
        let toast_overlay = toast_overlay.clone();
        let settings = settings.clone();
        key_controller.connect_key_pressed(move |_, keyval, keycode, state| {
            if keyval == gdk::Key::Print.into() {
                let settings = settings.clone();
                let toast_overlay = toast_overlay.clone();
                let show_screenshot = move |mode: &'static str| {
                    let settings = settings.clone();
                    let toast_overlay = toast_overlay.clone();
                    glib::MainContext::default().spawn_local(async move {
                        let result = match mode {
                            "fullscreen" => screenshot::capture_fullscreen(&settings).await,
                            "region" => screenshot::capture_region(&settings).await,
                            "window" => screenshot::capture_focused_window(&settings).await,
                            _ => return,
                        };
                        match result {
                            Ok(path) => {
                                let toast = adw::Toast::builder().title("Screenshot saved").timeout(3).build();
                                toast_overlay.add_toast(toast);
                            },
                            Err(e) => {
                                let toast = adw::Toast::builder().title(&format!("Screenshot failed: {}", e)).timeout(5).build();
                                toast_overlay.add_toast(toast);
                            }
                        }
                    });
                };
                let start_recording = move |mode: &'static str| {
                    let settings = settings.clone();
                    let toast_overlay = toast_overlay.clone();
                    // For now, just start normal recording; mode-specific logic can be added
                    glib::MainContext::default().spawn_local(async move {
                        let result = recorder::start_recording(&settings).await;
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
                };
                let toast = adw::Toast::builder()
                    .title("Choose action: Screenshot or Screen Record?")
                    .timeout(6)
                    .button_label("Screenshot")
                    .button_action(clone!(@weak toast_overlay => move || {
                        let mode_toast = adw::Toast::builder()
                            .title("Screenshot mode:")
                            .timeout(6)
                            .button_label("Fullscreen")
                            .button_action(clone!(@weak toast_overlay => move || show_screenshot("fullscreen")))
                            .button_label2("Region")
                            .button_action2(clone!(@weak toast_overlay => move || show_screenshot("region")))
                            .button_label3("Window")
                            .button_action3(clone!(@weak toast_overlay => move || show_screenshot("window")))
                            .build();
                        toast_overlay.add_toast(mode_toast);
                    }))
                    .button_label2("Screen Record")
                    .button_action2(clone!(@weak toast_overlay => move || {
                        let mode_toast = adw::Toast::builder()
                            .title("Screen Record mode:")
                            .timeout(6)
                            .button_label("Fullscreen")
                            .button_action(clone!(@weak toast_overlay => move || start_recording("fullscreen")))
                            .button_label2("Region")
                            .button_action2(clone!(@weak toast_overlay => move || start_recording("region")))
                            .button_label3("Window")
                            .button_action3(clone!(@weak toast_overlay => move || start_recording("window")))
                            .build();
                        toast_overlay.add_toast(mode_toast);
                    }))
                    .build();
                toast_overlay.add_toast(toast);
                return true;
            }
            false
        });
        key_controller
    });

    window.set_content(Some(&content));
    window.show();
} 