// SPDX-License-Identifier: GPL-3.0-or-later

//! Image effects on the current picture: Fractint-style 3D transforms and
//! random-dot stereograms, each with a settings window and a result window
//! that can be saved.

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use fractalforge_core::Palette;
use fractalforge_core::stereogram::{self, Bars};
use fractalforge_core::three_d::{self, Settings, Source};
use fractalforge_formats::gif::{self, GifImage};

/// The picture an effect starts from: palette indices and its Fractint
/// parameter block, if it has one.
pub struct Picture {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub main_block: Option<Vec<u8>>,
}

struct Result {
    title: &'static str,
    texture: TextureHandle,
    width: u32,
    height: u32,
    gif: Vec<u8>,
    rgb: Vec<u8>,
}

#[derive(Default)]
pub struct Effects {
    three_d: Option<Settings>,
    stereogram: Option<stereogram::Settings>,
    result: Option<Result>,
    /// Make the open effect at the next frame (screenshot test hook).
    make_requested: bool,
}

fn drag(ui: &mut egui::Ui, label: &str, v: &mut f64, range: std::ops::RangeInclusive<f64>) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(v).range(range).speed(1.0));
    });
}

impl Effects {
    pub fn open_three_d(&mut self) {
        self.three_d.get_or_insert_with(Settings::default);
    }

    pub fn open_stereogram(&mut self) {
        self.stereogram.get_or_insert_with(Default::default);
    }

    pub fn has_result(&self) -> bool {
        self.result.is_some()
    }

    /// Makes the open effect's picture at the next frame.
    pub fn request_make(&mut self) {
        self.make_requested = true;
    }

    /// Shows the open windows. `picture` gives the current picture (asked
    /// for only when an effect is made), shown with `palette`.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        picture: impl FnOnce() -> Option<Picture>,
        palette: &Palette,
        messages: &mut Vec<String>,
    ) {
        let mut make_3d = false;
        if let Some(s) = &mut self.three_d {
            let mut open = true;
            egui::Window::new("3D Transform")
                .open(&mut open)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("The current picture's colours become heights.");
                    ui.checkbox(&mut s.sphere, "Project onto a sphere");
                    if s.sphere {
                        let [mut a, mut b] = s.longitude();
                        drag(ui, "Longitude from", &mut a, -360.0..=360.0);
                        drag(ui, "Longitude to", &mut b, -360.0..=360.0);
                        s.rotation[0] = a;
                        s.rotation[1] = b;
                        let [mut a, mut b] = s.latitude();
                        drag(ui, "Latitude from", &mut a, -360.0..=360.0);
                        drag(ui, "Latitude to", &mut b, -360.0..=360.0);
                        (s.rotation[2], s.scale[0]) = (a, b);
                        drag(ui, "Radius %", &mut s.scale[1], 1.0..=1000.0);
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("Rotation x/y/z");
                            for r in &mut s.rotation {
                                ui.add(egui::DragValue::new(r).range(-360.0..=360.0).suffix("°"));
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Scale x/y %");
                            for v in &mut s.scale {
                                ui.add(egui::DragValue::new(v).range(-1000.0..=1000.0));
                            }
                        });
                        drag(ui, "Perspective %", &mut s.perspective, 0.0..=1000.0);
                        ui.horizontal(|ui| {
                            ui.label("Shift x/y %");
                            for v in &mut s.xyshift {
                                ui.add(egui::DragValue::new(v).range(-100.0..=100.0));
                            }
                        });
                    }
                    drag(ui, "Roughness %", &mut s.roughness, -1000.0..=1000.0);
                    ui.horizontal(|ui| {
                        ui.label("Waterline");
                        ui.add(egui::DragValue::new(&mut s.waterline));
                    });
                    egui::ComboBox::from_label("Fill")
                        .selected_text(fill_name(s.fill))
                        .show_ui(ui, |ui| {
                            for f in -1..=6 {
                                ui.selectable_value(&mut s.fill, f, fill_name(f));
                            }
                        });
                    if matches!(s.fill, 5 | 6) {
                        ui.horizontal(|ui| {
                            ui.label("Light x/y/z");
                            for v in &mut s.light {
                                ui.add(egui::DragValue::new(v).range(-100.0..=100.0));
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Ambient %");
                            ui.add(egui::DragValue::new(&mut s.ambient).range(0..=100));
                            ui.label("Smoothing");
                            ui.add(egui::DragValue::new(&mut s.smoothing).range(0..=20));
                        });
                    }
                    egui::ComboBox::from_label("Stereo")
                        .selected_text(
                            ["Off", "Red/cyan, alternate", "Red/cyan, superimposed"]
                                [s.stereo as usize],
                        )
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut s.stereo, 0, "Off");
                            ui.selectable_value(&mut s.stereo, 1, "Red/cyan, alternate");
                            ui.selectable_value(&mut s.stereo, 2, "Red/cyan, superimposed");
                        });
                    if s.stereo != 0 {
                        drag(ui, "Eye separation %", &mut s.interocular, -50.0..=50.0);
                        drag(ui, "Convergence %", &mut s.converge, -50.0..=50.0);
                    }
                    ui.horizontal(|ui| {
                        make_3d = ui.button("Make 3D Picture").clicked();
                        if ui.button("Defaults").clicked() {
                            *s = Settings::default();
                        }
                    });
                });
            if !open {
                self.three_d = None;
            }
        }
        let mut make_rds = false;
        if let Some(s) = &mut self.stereogram {
            let mut open = true;
            egui::Window::new("Stereogram")
                .open(&mut open)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("A random-dot stereogram of the current picture: relax your eyes until the two bars become three.");
                    ui.horizontal(|ui| {
                        ui.label("Depth effect");
                        ui.add(egui::DragValue::new(&mut s.depth).range(-500..=500));
                    });
                    drag(ui, "Picture width in inches", &mut s.width_inches, 1.0..=100.0);
                    ui.checkbox(&mut s.grayscale, "Depth from brightness");
                    egui::ComboBox::from_label("Calibration bars")
                        .selected_text(format!("{:?}", s.bars))
                        .show_ui(ui, |ui| {
                            for b in [Bars::Middle, Bars::Top, Bars::None] {
                                ui.selectable_value(&mut s.bars, b, format!("{b:?}"));
                            }
                        });
                    make_rds = ui.button("Make Stereogram").clicked();
                });
            if !open {
                self.stereogram = None;
            }
        }
        if std::mem::take(&mut self.make_requested) {
            make_3d = self.three_d.is_some();
            make_rds = !make_3d && self.stereogram.is_some();
        }
        if make_3d || make_rds {
            match picture() {
                None => messages.push("there is no picture yet".into()),
                Some(pic) => {
                    let made = if make_3d {
                        self.make_3d(ctx, &pic, palette)
                    } else {
                        self.make_stereogram(ctx, &pic, palette)
                    };
                    if let Err(e) = made {
                        messages.push(e);
                    }
                }
            }
        }
        self.show_result(ctx, messages);
    }

    fn make_3d(
        &mut self,
        ctx: &egui::Context,
        input: &Picture,
        palette: &Palette,
    ) -> std::result::Result<(), String> {
        let s = self.three_d.clone().unwrap_or_default();
        let out = three_d::render(
            &Source {
                pixels: &input.pixels,
                width: input.width,
                height: input.height,
                colours: 256,
            },
            &s,
        );
        let gif = gif::save_3d(&out, palette, &s, input.main_block.as_deref())
            .map_err(|e| e.to_string())?;
        let shown = GifImage::read(&gif).map_err(|e| e.to_string())?;
        self.set_result(ctx, "3D Picture", &shown, gif);
        Ok(())
    }

    fn make_stereogram(
        &mut self,
        ctx: &egui::Context,
        input: &Picture,
        palette: &Palette,
    ) -> std::result::Result<(), String> {
        let mut s = self.stereogram.clone().unwrap_or_default();
        s.seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(1, |d| d.as_secs() as u32);
        let pixels = stereogram::render(
            &input.pixels,
            input.width,
            input.height,
            &palette.0,
            &s,
            None,
        );
        let mut image = GifImage {
            width: input.width as u16,
            height: input.height as u16,
            palette: palette.0.to_vec(),
            background: 0,
            aspect: 0,
            pixels,
            extensions: Vec::new(),
        };
        if let Some(main) = &input.main_block {
            image.extensions.push(gif::container::AppExtension::new(
                *gif::fractint_blocks::IDENTIFIER,
                *gif::fractint_blocks::MAIN_AUTH,
                main,
                gif::container::Placement::AfterImage,
            ));
        }
        let bytes = image.write().map_err(|e| e.to_string())?;
        self.set_result(ctx, "Stereogram", &image, bytes);
        Ok(())
    }

    fn set_result(
        &mut self,
        ctx: &egui::Context,
        title: &'static str,
        img: &GifImage,
        gif: Vec<u8>,
    ) {
        let rgb: Vec<u8> = img
            .pixels
            .iter()
            .flat_map(|&i| img.palette.get(i as usize).copied().unwrap_or([0; 3]))
            .collect();
        let (w, h) = (img.width as u32, img.height as u32);
        let image = ColorImage::from_rgb([w as usize, h as usize], &rgb);
        self.result = Some(Result {
            title,
            texture: ctx.load_texture(title, image, TextureOptions::NEAREST),
            width: w,
            height: h,
            gif,
            rgb,
        });
    }

    fn show_result(&mut self, ctx: &egui::Context, messages: &mut Vec<String>) {
        let Some(r) = &self.result else { return };
        let mut open = true;
        let mut save = None;
        egui::Window::new(r.title)
            .open(&mut open)
            .default_width(r.width.min(800) as f32)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Save GIF…").clicked() {
                        save = Some("gif");
                    }
                    if ui.button("Save PNG…").clicked() {
                        save = Some("png");
                    }
                });
                let avail = ui.available_width().max(100.0);
                let scale = (avail / r.width as f32).min(1.0);
                ui.image((
                    r.texture.id(),
                    egui::vec2(r.width as f32 * scale, r.height as f32 * scale),
                ));
            });
        if let Some(ext) = save
            && let Some(path) = rfd::FileDialog::new()
                .add_filter(ext.to_uppercase(), &[ext])
                .set_file_name(format!("picture.{ext}"))
                .save_file()
        {
            let bytes = if ext == "gif" {
                Ok(r.gif.clone())
            } else {
                fractalforge_formats::png::encode_rgb(r.width, r.height, &r.rgb)
                    .map_err(|e| e.to_string())
            };
            match bytes.and_then(|b| std::fs::write(&path, b).map_err(|e| e.to_string())) {
                Ok(()) => messages.push(format!("saved {}", path.display())),
                Err(e) => messages.push(format!("{}: {e}", path.display())),
            }
        }
        if !open {
            self.result = None;
        }
    }
}

fn fill_name(f: i8) -> &'static str {
    match f {
        -1 => "Coarse grid",
        0 => "Points",
        1 => "Lines",
        2 => "Surface (interpolated)",
        3 => "Surface (flat)",
        4 => "Bars",
        5 => "Light source",
        _ => "Light source, turning with the surface",
    }
}
