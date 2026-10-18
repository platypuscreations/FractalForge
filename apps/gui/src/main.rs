// SPDX-License-Identifier: GPL-3.0-or-later

//! FractalForge desktop app.

// No console window on Windows (debug builds keep it for messages).
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod effects;
mod fonts;
#[cfg(target_os = "macos")]
mod macos;
mod view;
mod worker;

use std::path::PathBuf;

/// The Wayland app ID and X11 class: the name of the Linux packages'
/// .desktop file, so desktops show the window with the app's icon.
const APP_ID: &str = "fractalforge";

fn main() -> eframe::Result {
    // Older macOS passes a process serial number ("-psn_…") to apps.
    let open = std::env::args_os()
        .skip(1)
        .find(|a| !a.to_string_lossy().starts_with("-psn"))
        .map(PathBuf::from);
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("FractalForge")
            .with_app_id(APP_ID)
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([480.0, 320.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    // We build the event loop ourselves so macOS file-open support can be
    // added between building and running it (see macos.rs).
    let event_loop = winit::event_loop::EventLoop::<eframe::UserEvent>::with_user_event()
        .build()
        .map_err(|e| eframe::Error::AppCreation(Box::new(e)))?;
    #[cfg(target_os = "macos")]
    macos::install_open_handler();

    let mut app = eframe::create_native(
        "FractalForge",
        options,
        Box::new(|cc| {
            fonts::install(&cc.egui_ctx)?;
            #[cfg(target_os = "macos")]
            macos::set_context(&cc.egui_ctx);
            Ok(Box::new(app::App::new(cc, open)))
        }),
        &event_loop,
    );
    event_loop
        .run_app(&mut app)
        .map_err(|e| eframe::Error::AppCreation(Box::new(e)))
}
