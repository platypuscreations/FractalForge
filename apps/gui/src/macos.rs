// SPDX-License-Identifier: GPL-3.0-or-later

//! Receives files opened from Finder (double-click, "Open With", dropping on
//! the Dock icon). macOS delivers these to the application delegate as
//! `application:openURLs:`, not as command-line arguments. winit owns the
//! delegate and doesn't implement that method, so we add it to winit's
//! delegate class after the event loop is built and before it runs, which
//! also catches the file a launch was asked to open.
//!
//! This is the only module allowed to use `unsafe`: extending an
//! Objective-C class requires the Objective-C runtime.
#![allow(unsafe_code)]

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use eframe::egui;
use objc2::ffi::class_addMethod;
use objc2::runtime::{AnyClass, AnyObject, Imp, Sel};
use objc2::sel;
use objc2_foundation::{NSArray, NSURL};

static OPENED: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
static CTX: OnceLock<egui::Context> = OnceLock::new();

unsafe extern "C-unwind" fn open_urls(
    _this: *mut AnyObject,
    _cmd: Sel,
    _app: *mut AnyObject,
    urls: *mut NSArray<NSURL>,
) {
    // SAFETY: AppKit passes a valid NSArray of NSURL for this selector.
    let Some(urls) = (unsafe { urls.as_ref() }) else {
        return;
    };
    let mut opened = OPENED.lock().unwrap();
    for url in urls {
        if let Some(path) = url.path() {
            opened.push(PathBuf::from(path.to_string()));
        }
    }
    if let Some(ctx) = CTX.get() {
        ctx.request_repaint();
    }
}

/// Adds `application:openURLs:` to winit's application delegate. Call after
/// the event loop is built and before it runs.
pub fn install_open_handler() {
    let Some(class) = AnyClass::get(c"WinitApplicationDelegate") else {
        eprintln!("warning: can't find winit's application delegate; Finder file opening disabled");
        return;
    };
    // SAFETY: the function matches the selector's signature
    // (void, self, _cmd, NSApplication *, NSArray<NSURL *> *), described by
    // the type encoding "v@:@@". Adding a method the class lacks is allowed;
    // if it already exists, class_addMethod leaves it unchanged.
    unsafe {
        let imp: Imp = std::mem::transmute(
            open_urls
                as unsafe extern "C-unwind" fn(
                    *mut AnyObject,
                    Sel,
                    *mut AnyObject,
                    *mut NSArray<NSURL>,
                ),
        );
        let class = class as *const AnyClass as *mut AnyClass;
        class_addMethod(class, sel!(application:openURLs:), imp, c"v@:@@".as_ptr());
    }
}

/// Lets the handler wake the UI when files arrive.
pub fn set_context(ctx: &egui::Context) {
    let _ = CTX.set(ctx.clone());
}

/// Files opened from Finder since the last call.
pub fn take_opened() -> Vec<PathBuf> {
    std::mem::take(&mut *OPENED.lock().unwrap())
}
