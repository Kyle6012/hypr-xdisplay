use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};
use crate::settings::Settings;
use crate::ui::main_window::{build_ui, AppState};

mod display_manager;
mod network_display;
mod settings;
mod ui;

#[tokio::main]
async fn main() {
    // Setup logging
    let cache_dir = dirs::cache_dir().map(|p| p.join("hypr-xdisplay")).unwrap();
    let file_appender = tracing_appender::rolling::daily(cache_dir, "app.log");
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking_writer)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    info!("Application starting...");

    let application = adw::Application::new(
        Some("com.github.Kyle6012.hypr-xdisplay"),
        Default::default(),
    );

    // Create shared state
    let app_state: AppState = Arc::new(Mutex::new(Vec::new()));
    let settings = Arc::new(Settings::new());

    application.connect_activate(move |app| {
        build_ui(app, app_state.clone(), settings.clone());
    });

    application.run();
}