use gtk::prelude::*;
use std::sync::Arc;
use crate::settings::Settings;
use crate::screenshot;
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;
use std::process::Command;
use glib::clone;
use glib::MainContext;

pub fn add_screenshot_controls_section(content: &gtk::Box, settings: Arc<Settings>) {
    let screenshot_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    screenshot_box.set_halign(gtk::Align::Center);
    let screenshot_label = gtk::Label::builder().label("<b>Screenshot</b>").use_markup(true).build();

    let fullscreen_button = gtk::Button::with_label("Fullscreen");
    let region_button = gtk::Button::with_label("Region");
    let window_button = gtk::Button::with_label("Focused Window");

    screenshot_box.append(&fullscreen_button);
    screenshot_box.append(&region_button);
    screenshot_box.append(&window_button);

    content.append(&screenshot_label);
    content.append(&screenshot_box);

    // Action buttons below preview
    let actions_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    actions_box.set_halign(gtk::Align::Center);
    let copy_btn = gtk::Button::with_label("Copy to Clipboard");
    let annotate_btn = gtk::Button::with_label("Annotate");
    let share_btn = gtk::Button::with_label("Share/Upload");
    actions_box.append(&copy_btn);
    actions_box.append(&annotate_btn);
    actions_box.append(&share_btn);
    content.append(&actions_box);

    // Preview area
    let preview_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    preview_box.set_halign(gtk::Align::Center);
    let preview_label = gtk::Label::new(Some("Last Screenshot Preview:"));
    let preview_picture = gtk::Picture::new();
    preview_picture.set_can_shrink(true);
    preview_picture.set_size_request(200, 120);
    preview_box.append(&preview_label);
    preview_box.append(&preview_picture);
    content.append(&preview_box);

    // Toast overlay (find parent window)
    let toast_overlay = adw::ToastOverlay::new();
    content.append(&toast_overlay);

    // Shared state for last screenshot path
    let last_screenshot: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));

    // Helper to update preview and show toast
    let show_screenshot = {
        let preview_picture = preview_picture.clone();
        let toast_overlay = toast_overlay.clone();
        let last_screenshot = last_screenshot.clone();
        move |path: PathBuf| {
            preview_picture.set_filename(Some(path.to_str().unwrap()));
            *last_screenshot.borrow_mut() = Some(path.clone());
            let toast = adw::Toast::builder()
                .title("Screenshot Saved")
                .timeout(3)
                .build();
            toast_overlay.add_toast(toast);
        }
    };

    // Button handlers
    let settings_clone = settings.clone();
    let show_screenshot_clone = show_screenshot.clone();
    fullscreen_button.connect_clicked(move |_| {
        let settings = settings_clone.clone();
        let show_screenshot = show_screenshot_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            match screenshot::capture_fullscreen(settings.as_ref()).await {
                Ok(path) => show_screenshot(path),
                Err(e) => {
                    let toast = adw::Toast::builder().title(&format!("Screenshot failed: {}", e)).timeout(5).build();
                    toast_overlay.add_toast(toast);
                }
            }
        });
    });
    let settings_clone = settings.clone();
    let show_screenshot_clone = show_screenshot.clone();
    region_button.connect_clicked(move |_| {
        let settings = settings_clone.clone();
        let show_screenshot = show_screenshot_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            match screenshot::capture_region(settings.as_ref()).await {
                Ok(path) => show_screenshot(path),
                Err(e) => {
                    let toast = adw::Toast::builder().title(&format!("Screenshot failed: {}", e)).timeout(5).build();
                    toast_overlay.add_toast(toast);
                }
            }
        });
    });
    let settings_clone = settings.clone();
    let show_screenshot_clone = show_screenshot.clone();
    window_button.connect_clicked(move |_| {
        let settings = settings_clone.clone();
        let show_screenshot = show_screenshot_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            match screenshot::capture_focused_window(settings.as_ref()).await {
                Ok(path) => show_screenshot(path),
                Err(e) => {
                    let toast = adw::Toast::builder().title(&format!("Screenshot failed: {}", e)).timeout(5).build();
                    toast_overlay.add_toast(toast);
                }
            }
        });
    });

    // Click preview to open screenshot
    let last_screenshot_clone = last_screenshot.clone();
    // If using GTK4, connect_gesture_click is preferred. If not available, comment this out.
    // preview_picture.connect_gesture_click(move |_, _| {
    //     if let Some(path) = last_screenshot_clone.borrow().as_ref() {
    //         let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    //     }
    //     gtk4::glib::Propagation::Proceed
    // });

    // Copy to Clipboard logic
    let last_screenshot_clone = last_screenshot.clone();
    let toast_overlay_clone = toast_overlay.clone();
    copy_btn.connect_clicked(move |_| {
        if let Some(path) = last_screenshot_clone.borrow().as_ref() {
            // Try wl-copy (Wayland), fallback to xclip/xsel
            let result = if Command::new("which").arg("wl-copy").output().map(|o| o.status.success()).unwrap_or(false) {
                Command::new("wl-copy").arg(path).status()
            } else if Command::new("which").arg("xclip").output().map(|o| o.status.success()).unwrap_or(false) {
                Command::new("xclip").arg("-selection").arg("clipboard").arg("-t").arg("image/png").arg("-i").arg(path).status()
            } else if Command::new("which").arg("xsel").output().map(|o| o.status.success()).unwrap_or(false) {
                Command::new("xsel").arg("--clipboard").arg("--input").arg(path).status()
            } else {
                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No clipboard tool found"))
            };
            match result {
                Ok(status) if status.success() => {
                    let toast = adw::Toast::builder().title("Copied screenshot to clipboard").timeout(3).build();
                    toast_overlay_clone.add_toast(toast);
                }
                _ => {
                    let toast = adw::Toast::builder().title("Failed to copy to clipboard").timeout(5).build();
                    toast_overlay_clone.add_toast(toast);
                }
            }
        }
    });

    // Annotate logic (using swappy)
    let last_screenshot_clone = last_screenshot.clone();
    let toast_overlay_clone = toast_overlay.clone();
    annotate_btn.connect_clicked(move |_| {
        if let Some(path) = last_screenshot_clone.borrow().as_ref() {
            let result = Command::new("swappy").arg("-f").arg(path).spawn();
            match result {
                Ok(_) => {
                    let toast = adw::Toast::builder().title("Opened in annotation tool").timeout(3).build();
                    toast_overlay_clone.add_toast(toast);
                }
                Err(_) => {
                    let toast = adw::Toast::builder().title("Failed to open annotation tool (is swappy installed?)").timeout(5).build();
                    toast_overlay_clone.add_toast(toast);
                }
            }
        }
    });

    // Share/Upload logic (imgur, anonymous)
    let last_screenshot_clone = last_screenshot.clone();
    let toast_overlay_clone = toast_overlay.clone();

    let (sender, receiver) = MainContext::channel(glib::PRIORITY_DEFAULT);

    receiver.attach(None, clone!(@weak toast_overlay_clone => @default-return glib::Continue(false), move |msg| {
        toast_overlay_clone.add_toast(adw::Toast::builder().title(&msg).timeout(6).build());
        glib::Continue(true)
    }));

    share_btn.connect_clicked(move |_| {
        if let Some(path) = last_screenshot_clone.borrow().as_ref() {
            let path = path.clone();
            let sender = sender.clone();
            std::thread::spawn(move || {
                let output = Command::new("curl")
                    .arg("-s")
                    .arg("-F")
                    .arg(format!("image=@{}", path.display()))
                    .arg("https://api.imgur.com/3/image")
                    .arg("-H")
                    .arg("Authorization: Client-ID 546b2e6e0b1b1b1") // Demo client ID, replace for production
                    .output();

                let message = match output {
                    Ok(out) if out.status.success() => {
                        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                            if let Some(link) = json["data"]["link"].as_str() {
                                let _ = Command::new("wl-copy").arg(link).status(); // Consider making this async or handling errors better
                                format!("Uploaded! Link copied: {}", link)
                            } else {
                                "Upload failed: Could not parse response".to_string()
                            }
                        } else {
                            "Upload failed: Could not parse response".to_string()
                        }
                    }
                    Ok(out) => {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        format!("Upload failed: {}", stderr)
                    },
                    Err(e) => format!("Upload failed: {}", e),
                };

                sender.send(message).expect("Failed to send message to main thread");
            });
        }
    });
} 