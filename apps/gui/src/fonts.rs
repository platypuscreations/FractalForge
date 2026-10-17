// SPDX-License-Identifier: GPL-3.0-or-later

//! Uses the operating system's UI fonts. egui's bundled fonts carry licences
//! (Ubuntu Font Licence, OFL) we don't want in a GPL binary, so none are
//! bundled.

use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

const PROPORTIONAL: &[&str] = &[
    // macOS
    "/System/Library/Fonts/SFNS.ttf",
    "/System/Library/Fonts/Helvetica.ttc",
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    // Windows
    "C:\\Windows\\Fonts\\segoeui.ttf",
    "C:\\Windows\\Fonts\\arial.ttf",
    // Linux
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    "/usr/share/fonts/noto/NotoSans-Regular.ttf",
];

const MONOSPACE: &[&str] = &[
    "/System/Library/Fonts/SFNSMono.ttf",
    "/System/Library/Fonts/Menlo.ttc",
    "C:\\Windows\\Fonts\\consola.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
    "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
];

fn first_readable(candidates: &[&str]) -> Option<Vec<u8>> {
    candidates.iter().find_map(|p| std::fs::read(p).ok())
}

/// Installs system fonts. `FRACTALFORGE_FONT` overrides the proportional
/// font. Returns an error when no usable font is found.
pub fn install(ctx: &egui::Context) -> Result<(), String> {
    let custom = std::env::var_os("FRACTALFORGE_FONT").and_then(|p| std::fs::read(p).ok());
    let proportional = custom
        .or_else(|| first_readable(PROPORTIONAL))
        .ok_or("no system font found; set FRACTALFORGE_FONT to a .ttf file")?;
    let monospace = first_readable(MONOSPACE).unwrap_or_else(|| proportional.clone());

    let mut fonts = FontDefinitions::empty();
    fonts
        .font_data
        .insert("ui".into(), Arc::new(FontData::from_owned(proportional)));
    fonts
        .font_data
        .insert("mono".into(), Arc::new(FontData::from_owned(monospace)));
    fonts
        .families
        .insert(FontFamily::Proportional, vec!["ui".into(), "mono".into()]);
    fonts
        .families
        .insert(FontFamily::Monospace, vec!["mono".into(), "ui".into()]);
    ctx.set_fonts(fonts);
    Ok(())
}
