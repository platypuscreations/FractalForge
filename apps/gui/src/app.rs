// SPDX-License-Identifier: GPL-3.0-or-later

//! The FractalForge window: canvas, controls, files and colour cycling.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui::{
    self, Color32, ColorImage, Key, KeyboardShortcut, Modifiers, PointerButton, Pos2, Rect, Sense,
    Stroke, TextureHandle, TextureOptions, Vec2,
};
use fractalforge_core::bignum::BigFixed;
use fractalforge_core::deep::DeepView;
use fractalforge_core::{Inside, Outside, PRESETS, Palette, Params, Window, types};
use fractalforge_formats::gif::{self, Checkpoint, Content};
use fractalforge_formats::programs::Kind;
use fractalforge_formats::{map, par};
use fractalforge_render::{PassMode, Render};
use rayon::prelude::*;

use crate::view;
use crate::worker::{Frame, TRANSPARENT, Worker};

/// Delay after the last view change before re-rendering, so continuous
/// gestures don't restart the render every frame.
const SETTLE: Duration = Duration::from_millis(250);

const SIZES: &[(u32, u32)] = &[
    (640, 480),
    (800, 600),
    (1024, 768),
    (1920, 1080),
    (3840, 2160),
];

const SAVE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);
const SAVE_AS: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::S);
const OPEN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
const UNDO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Z);
const REDO: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::Z);

#[derive(Clone, Copy, PartialEq)]
enum SizeMode {
    /// Render at the canvas's physical pixel size.
    FitWindow,
    Fixed(u32, u32),
}

/// An image on screen and the view it was rendered for.
struct Shown {
    texture: TextureHandle,
    view: ViewPos,
}

/// A view: its window, and its precise centre for deep zooms.
#[derive(Clone, PartialEq)]
struct ViewPos {
    window: Window,
    deep: Option<DeepView>,
}

impl ViewPos {
    fn of(p: &Params) -> Self {
        Self {
            window: p.window,
            deep: p.deep.clone(),
        }
    }

    fn restore(&self, p: &mut Params) {
        p.window = self.window;
        p.deep = self.deep.clone();
    }

    fn centre(&self, frac: usize) -> (BigFixed, BigFixed) {
        match &self.deep {
            Some(d) => (d.re.with_frac(frac), d.im.with_frac(frac)),
            None => {
                let w = &self.window;
                (
                    BigFixed::from_f64((w.x_min + w.x_max) / 2.0, frac),
                    BigFixed::from_f64((w.y_min + w.y_max) / 2.0, frac),
                )
            }
        }
    }

    fn half_spans(&self) -> (f64, f64) {
        match &self.deep {
            Some(d) => (d.half_w, d.half_h),
            None => {
                let w = &self.window;
                ((w.x_max - w.x_min) / 2.0, (w.y_max - w.y_min) / 2.0)
            }
        }
    }

    /// This view's window in coordinates centred on `other`'s centre, so
    /// deep views can be compared in doubles.
    fn relative_to(&self, other: &ViewPos) -> Window {
        if self.deep.is_none() && other.deep.is_none() {
            return self.window;
        }
        let frac = [&self.deep, &other.deep]
            .iter()
            .filter_map(|d| d.as_ref().map(|d| d.re.frac_limbs()))
            .max()
            .unwrap_or(1);
        let (ax, ay) = self.centre(frac);
        let (bx, by) = other.centre(frac);
        let (dx, dy) = (ax.sub(&bx).to_f64(), ay.sub(&by).to_f64());
        let (hw, hh) = self.half_spans();
        Window::new(dx - hw, dx + hw, dy - hh, dy + hh)
    }

    /// `other`'s view in the coordinates of [`ViewPos::relative_to`].
    fn own_frame(&self) -> Window {
        if self.deep.is_none() {
            return self.window;
        }
        let (hw, hh) = self.half_spans();
        Window::new(-hw, hw, -hh, hh)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Status {
    Idle,
    Rendering,
    Stopped,
    Complete,
}

pub struct App {
    params: Params,
    pass_mode: PassMode,
    size_mode: SizeMode,
    /// Size of the current render.
    render_size: (u32, u32),

    worker: Option<Worker>,
    /// Render taken back from a stopped worker, kept for saving/resuming.
    stopped: Option<Render>,
    status: Status,
    progress: f64,
    elapsed_us: u64,
    restart_at: Option<Instant>,

    frame: Option<Frame>,
    image: Option<Shown>,
    /// The previous image, shown under the new render until it finishes.
    backdrop: Option<Shown>,
    /// The window the current render was started for.
    render_window: ViewPos,

    palette_keys: Vec<[u8; 3]>,
    palette: Palette,
    cycling: bool,
    cycle_speed: f32,
    cycle_offset: f32,
    recolor: bool,

    history: Vec<(Params, PassMode)>,
    history_pos: usize,

    file: Option<PathBuf>,
    autosave: bool,
    autosave_minutes: u32,
    last_autosave: Instant,
    messages: Vec<String>,
    title: String,

    drag: Option<Drag>,
    /// 3D transform and stereogram windows.
    effects: crate::effects::Effects,
    /// Draw the orbit of the point under the pointer over the image.
    show_orbit: bool,
    /// The pointer's normalised image position, while over the image.
    hover: Option<(f64, f64)>,
    /// Parameter-plane views left with the Julia toggle, most recent last,
    /// so toggling back returns to them.
    mandel_views: Vec<(&'static str, ViewPos)>,
    /// A .par file with several entries, waiting for the user to pick one.
    par_choice: Option<(PathBuf, Vec<String>)>,
    /// A Fractint formula GIF waiting for its .frm file: the entry name and
    /// the image's parameters (the formula's file isn't stored in GIFs).
    pending_formula: Option<(String, Params)>,
    /// "Save parameters" dialog: file, entry name, comment.
    par_save: Option<(PathBuf, String, String)>,
    /// Test hook: `FRACTALFORGE_SCREENSHOT=out.ppm` saves a screenshot of
    /// the window once the first render completes, then quits.
    screenshot: Option<(PathBuf, bool)>,
}

#[derive(Clone)]
enum Drag {
    Pan { start: Pos2, view: ViewPos },
    ZoomBox { start: Pos2 },
}

impl App {
    pub fn new(cc: &eframe::CreationContext, open: Option<PathBuf>) -> Self {
        let params = Params::for_type("mandel").unwrap();
        let keys = PRESETS[0].1.to_vec();
        let mut app = Self {
            render_window: ViewPos::of(&params),
            params,
            pass_mode: PassMode::Guess,
            size_mode: SizeMode::FitWindow,
            render_size: (0, 0),
            worker: None,
            stopped: None,
            status: Status::Idle,
            progress: 0.0,
            elapsed_us: 0,
            restart_at: Some(Instant::now()),
            frame: None,
            image: None,
            backdrop: None,
            palette: Palette::from_keys(&keys),
            palette_keys: keys,
            cycling: false,
            cycle_speed: 20.0,
            cycle_offset: 0.0,
            recolor: false,
            history: Vec::new(),
            history_pos: 0,
            file: None,
            autosave: false,
            autosave_minutes: 5,
            last_autosave: Instant::now(),
            messages: Vec::new(),
            title: String::new(),
            drag: None,
            effects: Default::default(),
            show_orbit: false,
            hover: None,
            mandel_views: Vec::new(),
            par_choice: None,
            pending_formula: None,
            par_save: None,
            screenshot: std::env::var_os("FRACTALFORGE_SCREENSHOT")
                .map(|p| (PathBuf::from(p), false)),
        };
        app.record_history();
        if let Some(path) = open {
            app.open(&path, &cc.egui_ctx);
        }
        app
    }

    // ---- rendering -----------------------------------------------------

    fn canvas_size(&self, canvas: Rect, ppp: f32) -> (u32, u32) {
        match self.size_mode {
            SizeMode::Fixed(w, h) => (w, h),
            SizeMode::FitWindow => {
                let px = canvas.size() * ppp;
                ((px.x.round() as u32).max(1), (px.y.round() as u32).max(1))
            }
        }
    }

    /// Discards the current render and starts a new one for the current
    /// parameters.
    fn restart(&mut self, ctx: &egui::Context) {
        self.restart_at = None;
        self.stop_worker();
        self.stopped = None;
        let (w, h) = self.render_size;
        if w == 0 || h == 0 {
            return;
        }
        match Render::new(self.params.clone(), w, h, self.pass_mode) {
            Ok(render) => self.start(render, ctx),
            Err(e) => self.messages.push(format!("can't render: {e}")),
        }
    }

    fn start(&mut self, render: Render, ctx: &egui::Context) {
        if let Some(img) = self.image.take() {
            self.backdrop = Some(img);
        }
        self.params = render.params().clone();
        self.render_window = ViewPos::of(&self.params);
        self.render_size = (render.buffer().width(), render.buffer().height());
        self.status = Status::Rendering;
        let ctx = ctx.clone();
        self.worker = Some(Worker::start(render, move || ctx.request_repaint()));
    }

    fn stop_worker(&mut self) -> Option<Render> {
        let render = self.worker.take().map(Worker::stop)?;
        self.status = if render.is_complete() {
            Status::Complete
        } else {
            Status::Stopped
        };
        Some(render)
    }

    /// Stops rendering, keeping the progress so it can be saved or resumed.
    fn pause(&mut self) {
        if let Some(r) = self.stop_worker() {
            self.stopped = Some(r);
        }
    }

    fn resume(&mut self, ctx: &egui::Context) {
        if let Some(r) = self.stopped.take() {
            self.start(r, ctx);
            self.backdrop = None;
        }
    }

    fn schedule_restart(&mut self) {
        self.restart_at = Some(Instant::now() + SETTLE);
    }

    fn poll_worker(&mut self, ctx: &egui::Context) {
        let Some(worker) = &self.worker else { return };
        if let Some(frame) = worker.take_frame() {
            self.progress = frame.progress;
            self.elapsed_us = frame.elapsed_us;
            if frame.complete {
                self.backdrop = None;
            }
            self.frame = Some(frame);
            self.recolor = true;
        }
        if worker.is_finished() {
            let render = self.stop_worker().unwrap();
            self.stopped = Some(render);
            ctx.request_repaint();
        }
    }

    fn update_texture(&mut self, ctx: &egui::Context) {
        if !self.recolor {
            return;
        }
        let Some(frame) = &self.frame else { return };
        self.recolor = false;
        let palette = self.palette.rotated(self.cycle_offset as i32);
        let mut rgba = vec![0u8; frame.indices.len() * 4];
        rgba.par_chunks_mut(4)
            .zip(&frame.indices)
            .for_each(|(px, &i)| {
                if i != TRANSPARENT {
                    let [r, g, b] = palette.0[i as usize];
                    px.copy_from_slice(&[r, g, b, 255]);
                }
            });
        let image = ColorImage::from_rgba_unmultiplied([frame.width, frame.height], &rgba);
        match &mut self.image {
            Some(shown) if shown.texture.size() == [frame.width, frame.height] => {
                shown.texture.set(image, TextureOptions::LINEAR);
                shown.view = self.render_window.clone();
            }
            _ => {
                self.image = Some(Shown {
                    texture: ctx.load_texture("fractal", image, TextureOptions::LINEAR),
                    view: self.render_window.clone(),
                })
            }
        }
    }

    // ---- history --------------------------------------------------------

    fn record_history(&mut self) {
        let entry = (self.params.clone(), self.pass_mode);
        if self.history.get(self.history_pos.wrapping_sub(1)) == Some(&entry) {
            return;
        }
        self.history.truncate(self.history_pos);
        self.history.push(entry);
        self.history_pos = self.history.len();
    }

    /// Records the new parameters and re-renders after the view settles.
    fn changed(&mut self) {
        self.record_history();
        self.schedule_restart();
    }

    fn undo(&mut self) {
        if self.history_pos > 1 {
            self.history_pos -= 1;
            (self.params, self.pass_mode) = self.history[self.history_pos - 1].clone();
            self.schedule_restart();
        }
    }

    fn redo(&mut self) {
        if self.history_pos < self.history.len() {
            self.history_pos += 1;
            (self.params, self.pass_mode) = self.history[self.history_pos - 1].clone();
            self.schedule_restart();
        }
    }

    // ---- files ----------------------------------------------------------

    fn open_dialog(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Fractal files", &["gif", "par", "frm", "ifs", "l", "map"])
            .pick_file()
        {
            self.open(&path, ctx);
        }
    }

    pub fn open(&mut self, path: &Path, ctx: &egui::Context) {
        let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase());
        match ext.as_deref() {
            Some("par" | "frm" | "ifs" | "l") => return self.open_par(path, ctx),
            Some("map") => return self.open_map(path),
            _ => {}
        }
        let loaded = std::fs::read(path)
            .map_err(|e| e.to_string())
            .and_then(|b| gif::load(&b).map_err(|e| e.to_string()));
        let loaded = match loaded {
            Ok(l) => l,
            Err(e) => {
                self.messages.push(format!("{}: {e}", path.display()));
                return;
            }
        };
        let size = (loaded.image.width as u32, loaded.image.height as u32);
        match loaded.content {
            Content::Native(c) => {
                let c = *c;
                self.palette = c.palette.clone();
                self.recolor = true;
                self.pass_mode = c.state.pass_mode;
                self.size_mode = SizeMode::Fixed(size.0, size.1);
                let params = c.params.clone();
                match Render::resume(c.params, c.state, c.buffer) {
                    Ok(render) => {
                        self.stop_worker();
                        self.restart_at = None;
                        self.image = None;
                        self.backdrop = None;
                        self.start(render, ctx);
                    }
                    // Plotted types (IFS) start again, as in Fractint.
                    Err(fractalforge_render::ResumeError::NotResumable(_))
                        if params.fractal.plot.is_some() =>
                    {
                        self.messages.push(format!(
                            "{}: {} images can't be resumed; calculating from the start",
                            path.display(),
                            params.fractal.name
                        ));
                        self.params = params;
                        self.render_size = size;
                        self.restart(ctx);
                    }
                    Err(e) => self.messages.push(format!("{}: {e}", path.display())),
                }
            }
            Content::Fractint(info) => {
                self.messages.extend(
                    info.warnings
                        .iter()
                        .map(|w| format!("{}: {w}", path.display())),
                );
                let Some(mut params) = info.params else {
                    self.messages.push(format!(
                        "{}: this Fractint type isn't supported yet",
                        path.display()
                    ));
                    return;
                };
                if params.fractal.needs_program {
                    // Formula and IFS GIFs name the entry but not its file:
                    // look next to the GIF, else ask for the file.
                    let name = info.formula_name.clone().unwrap_or_default();
                    let kind = Kind::of_type(params.fractal).unwrap();
                    let found = path
                        .parent()
                        .and_then(|d| kind.find_in_dir(d, &name))
                        .map(|(_, text)| kind.load(&text, &name));
                    for (c, p) in self.palette.0.iter_mut().zip(&loaded.image.palette) {
                        *c = *p;
                    }
                    self.recolor = true;
                    self.size_mode = SizeMode::Fixed(size.0, size.1);
                    self.render_size = size;
                    self.pass_mode = info.pass_mode;
                    match found {
                        Some(Ok((program, warnings))) => {
                            self.messages.extend(warnings);
                            params.program = Some(program);
                        }
                        Some(Err(e)) => {
                            self.messages.push(e);
                            return;
                        }
                        None => {
                            self.messages.push(format!(
                                "{}: needs the {} '{name}'. Open the .{} file that has it.",
                                path.display(),
                                kind.noun(),
                                kind.extension()
                            ));
                            self.pending_formula = Some((name, params));
                            self.file = Some(path.to_owned());
                            return;
                        }
                    }
                }
                let partial = if info.finished {
                    None
                } else {
                    gif::fractint_blocks::partial_buffer(&loaded.image, params.fractal)
                };
                if !info.finished && partial.is_none() {
                    self.messages.push(format!(
                        "{}: this file's resume data can't be used; calculating from the start",
                        path.display()
                    ));
                }
                self.params = params;
                self.pass_mode = info.pass_mode;
                for (c, p) in self.palette.0.iter_mut().zip(&loaded.image.palette) {
                    *c = *p;
                }
                self.recolor = true;
                self.size_mode = SizeMode::Fixed(size.0, size.1);
                self.render_size = size;
                match partial {
                    // Continue from Fractint's progress: its final pixels are
                    // kept and the rest computed in one pass.
                    Some(buf) => {
                        match Render::from_partial(self.params.clone(), buf, PassMode::One) {
                            Ok(render) => {
                                self.stop_worker();
                                self.restart_at = None;
                                self.image = None;
                                self.backdrop = None;
                                self.pass_mode = PassMode::One;
                                self.start(render, ctx);
                            }
                            Err(e) => self.messages.push(format!("{}: {e}", path.display())),
                        }
                    }
                    None => self.restart(ctx),
                }
            }
            Content::Image => {
                self.messages.push(format!(
                    "{}: no fractal parameters in this GIF",
                    path.display()
                ));
                return;
            }
        }
        self.file = Some(path.to_owned());
        self.record_history();
    }

    /// Opens a .par, .frm or .ifs file: its only entry directly, otherwise
    /// through the entry picker.
    fn open_par(&mut self, path: &Path, ctx: &egui::Context) {
        let text = match fractalforge_formula::read_text(path) {
            Ok(t) => t,
            Err(e) => return self.messages.push(format!("{}: {e}", path.display())),
        };
        let kind = entry_file_kind(path);
        if let (Some(kind), Some((name, params))) = (kind, &self.pending_formula)
            && Kind::of_type(params.fractal) == Some(kind)
        {
            // The entry a GIF was waiting for.
            let name = name.clone();
            if kind
                .entry_names(&text)
                .iter()
                .any(|n| n.eq_ignore_ascii_case(&name))
            {
                return self.load_par_entry(path, &name, ctx);
            }
        }
        let names: Vec<String> = if let Some(kind) = kind {
            kind.entry_names(&text)
        } else {
            par::entries(&text)
                .into_iter()
                .map(|e| e.name)
                .filter(|n| !fractalforge_formats::programs::is_embedded(n))
                .collect()
        };
        match names.as_slice() {
            [] => self
                .messages
                .push(format!("{}: no entries", path.display())),
            [only] => {
                let only = only.clone();
                self.load_par_entry(path, &only, ctx);
            }
            _ => self.par_choice = Some((path.to_owned(), names)),
        }
    }

    fn load_par_entry(&mut self, path: &Path, name: &str, ctx: &egui::Context) {
        let text = fractalforge_formula::read_text(path).unwrap_or_default();
        if let Some(kind) = entry_file_kind(path) {
            return self.load_formula(kind, &text, name, ctx);
        }
        let all = par::entries(&text);
        let Some(raw) = par::find(&all, name) else {
            return;
        };
        match par::load(raw, path.parent(), Some(&text)) {
            Ok(l) => {
                self.messages
                    .extend(l.warnings.iter().map(|w| format!("{name}: {w}")));
                self.params = l.params;
                if let Some(m) = l.pass_mode {
                    self.pass_mode = m;
                }
                if let Some(p) = l.palette {
                    self.palette = p;
                    self.recolor = true;
                }
                let aspect = self.aspect();
                self.params.reframe(|w| view::fit_aspect(w, aspect));
                self.record_history();
                self.restart(ctx);
            }
            Err(e) => self.messages.push(e.to_string()),
        }
    }

    /// Uses formula or IFS entry `name`: for a GIF waiting for it, with
    /// that image's parameters; otherwise as a new image.
    fn load_formula(&mut self, kind: Kind, text: &str, name: &str, ctx: &egui::Context) {
        let (program, warnings) = match kind.load(text, name) {
            Ok(r) => r,
            Err(e) => return self.messages.push(e),
        };
        self.messages.extend(warnings);
        match self.pending_formula.take() {
            Some((_, mut params)) if params.fractal == kind.fractal() => {
                params.program = Some(program);
                self.params = params;
            }
            _ => {
                let mut params = Params::new(kind.fractal());
                params.window = view::fit_aspect(&params.window, self.aspect());
                params.program = Some(program);
                self.params = params;
            }
        }
        self.record_history();
        self.restart(ctx);
    }

    fn open_map(&mut self, path: &Path) {
        match std::fs::read_to_string(path) {
            Ok(text) => {
                let (p, warnings) = map::read(&text);
                self.messages
                    .extend(warnings.iter().map(|w| format!("{}: {w}", path.display())));
                self.palette = p;
                self.cycle_offset = 0.0;
                self.recolor = true;
            }
            Err(e) => self.messages.push(format!("{}: {e}", path.display())),
        }
    }

    fn save_par_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Fractint parameter file", &["par"])
            .set_file_name("fractals.par")
            .save_file()
        {
            let stem: String = path
                .file_stem()
                .map(|s| s.to_string_lossy().chars().take(18).collect())
                .unwrap_or_default();
            self.par_save = Some((path, stem, String::new()));
        }
    }

    fn export_map(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Fractint palette", &["map"])
            .set_file_name("palette.map")
            .save_file()
        else {
            return;
        };
        let pal = self.palette.rotated(self.cycle_offset as i32);
        match std::fs::write(&path, map::write(&pal)) {
            Ok(()) => self.messages.push(format!("exported {}", path.display())),
            Err(e) => self.messages.push(format!("{}: {e}", path.display())),
        }
    }

    /// Entry picker and "save parameters" windows.
    fn dialogs(&mut self, ctx: &egui::Context) {
        if let Some((path, names)) = self.par_choice.clone() {
            let mut open = true;
            let mut chosen = None;
            egui::Window::new("Choose an entry")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            for n in &names {
                                if ui.button(n).clicked() {
                                    chosen = Some(n.clone());
                                }
                            }
                        });
                });
            if let Some(n) = chosen {
                self.par_choice = None;
                self.load_par_entry(&path, &n, ctx);
            } else if !open {
                self.par_choice = None;
            }
        }
        if let Some((path, mut name, mut comment)) = self.par_save.take() {
            let mut open = true;
            let mut save = false;
            egui::Window::new("Save parameters")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(path.display().to_string());
                    ui.horizontal(|ui| {
                        ui.label("Entry name");
                        ui.text_edit_singleline(&mut name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Comment");
                        ui.text_edit_singleline(&mut comment);
                    });
                    ui.label(
                        "An entry with the same name is replaced; the rest of the file is kept.",
                    );
                    save = ui.button("Save").clicked();
                });
            if save {
                let pal = self.palette.rotated(self.cycle_offset as i32);
                let existing = std::fs::read_to_string(&path).unwrap_or_default();
                let result = par::save_entry(
                    &existing,
                    &name,
                    Some(&comment),
                    &self.params,
                    self.pass_mode,
                    Some(&pal),
                )
                .map_err(|e| e.to_string())
                .and_then(|text| {
                    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
                });
                match result {
                    Ok(()) => self
                        .messages
                        .push(format!("saved {name} to {}", path.display())),
                    Err(e) => {
                        self.messages.push(e);
                        self.par_save = Some((path, name, comment));
                    }
                }
            } else if open {
                self.par_save = Some((path, name, comment));
            }
        }
    }

    /// Saves the current image (finished or not) as a GIF. With `stop`,
    /// the render stays stopped afterwards.
    fn save(&mut self, path: Option<PathBuf>, stop: bool, ctx: &egui::Context) {
        let path = match path.or_else(|| self.file.clone()) {
            Some(p) => p,
            None => match rfd::FileDialog::new()
                .add_filter("GIF image", &["gif"])
                .set_file_name("fractal.gif")
                .save_file()
            {
                Some(p) => p,
                None => return,
            },
        };
        let was_running = self.worker.is_some();
        self.pause();
        let Some(render) = self.stopped.take() else {
            self.messages.push("nothing to save yet".into());
            return;
        };
        let complete = render.is_complete();
        let (params, state, buffer) = render.into_parts();
        let checkpoint = Checkpoint {
            params,
            state,
            buffer,
            palette: self.palette.rotated(self.cycle_offset as i32),
        };
        match write_gif(&path, &checkpoint, complete) {
            Ok(notes) => {
                self.messages.extend(notes);
                self.file = Some(path);
            }
            Err(e) => self.messages.push(e),
        }
        let render = Render::resume(checkpoint.params, checkpoint.state, checkpoint.buffer)
            .expect("checkpoint taken from a running render");
        if was_running && !stop {
            self.start(render, ctx);
            self.backdrop = None;
        } else {
            self.stopped = Some(render);
        }
    }

    /// Writes the image as currently shown (palette and cycling included).
    fn export_png(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG image", &["png"])
            .set_file_name("fractal.png")
            .save_file()
        else {
            return;
        };
        let palette = self.palette.rotated(self.cycle_offset as i32);
        let result = match (&self.worker, &self.stopped) {
            (Some(_), _) => Err("wait for the render to finish, or stop it first".to_owned()),
            (None, Some(r)) => fractalforge_formats::png::encode(r.params(), r.buffer(), &palette)
                .map_err(|e| e.to_string())
                .and_then(|b| {
                    std::fs::write(&path, b).map_err(|e| format!("{}: {e}", path.display()))
                }),
            (None, None) => Err("nothing to export yet".to_owned()),
        };
        match result {
            Ok(()) => self.messages.push(format!("exported {}", path.display())),
            Err(e) => self.messages.push(e),
        }
    }

    fn autosave(&mut self, ctx: &egui::Context) {
        let interval = Duration::from_secs(self.autosave_minutes as u64 * 60);
        if !self.autosave || self.worker.is_none() || self.last_autosave.elapsed() < interval {
            return;
        }
        self.last_autosave = Instant::now();
        let path = autosave_path(self.file.as_deref());
        let keep = self.file.clone();
        self.save(Some(path.clone()), false, ctx);
        self.file = keep;
        self.messages
            .push(format!("autosaved to {}", path.display()));
    }

    // ---- UI -------------------------------------------------------------

    fn update_title(&mut self, ctx: &egui::Context) {
        let name = self
            .file
            .as_deref()
            .and_then(|p| p.file_name())
            .map_or("Untitled".into(), |n| n.to_string_lossy().into_owned());
        let state = match self.status {
            Status::Idle => String::new(),
            Status::Rendering => format!(" — rendering {:.0}%", self.progress * 100.0),
            Status::Stopped => format!(" — stopped at {:.0}%", self.progress * 100.0),
            Status::Complete => String::new(),
        };
        let title = format!("{name}{state} — FractalForge");
        if title != self.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|i| i.consume_shortcut(&SAVE_AS)) {
            self.save_as(false, ctx);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&SAVE)) {
            self.save(None, false, ctx);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&OPEN)) {
            self.open_dialog(ctx);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&REDO)) {
            self.redo();
        }
        if ctx.input_mut(|i| i.consume_shortcut(&UNDO)) {
            self.undo();
        }
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        ctx.input(|i| {
            if i.key_pressed(Key::C) {
                self.cycling = !self.cycling;
            }
            if i.key_pressed(Key::O) {
                self.show_orbit = !self.show_orbit;
            }
            if i.key_pressed(Key::Plus) || i.key_pressed(Key::Equals) {
                self.cycle_speed = (self.cycle_speed * 1.5).min(500.0);
            }
            if i.key_pressed(Key::Minus) {
                self.cycle_speed = (self.cycle_speed / 1.5).max(1.0);
            }
        });
        if ctx.input(|i| i.key_pressed(Key::Space)) {
            self.toggle_julia();
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.pause();
        }
        if ctx.input(|i| i.key_pressed(Key::Enter)) {
            self.resume(ctx);
        }
        if ctx.input(|i| i.key_pressed(Key::Home)) {
            self.params.deep = None;
            self.params.window =
                view::fit_aspect(&self.params.fractal.default_window, self.aspect());
            self.changed();
        }
        for file in ctx.input(|i| i.raw.dropped_files.clone()) {
            let path = file.path().to_owned();
            self.open(&path, ctx);
        }
        #[cfg(target_os = "macos")]
        for path in crate::macos::take_opened() {
            self.open(&path, ctx);
        }
    }

    fn save_as(&mut self, stop: bool, ctx: &egui::Context) {
        if let Some(p) = rfd::FileDialog::new()
            .add_filter("GIF image", &["gif"])
            .set_file_name("fractal.gif")
            .save_file()
        {
            self.save(Some(p), stop, ctx);
        }
    }

    fn aspect(&self) -> f64 {
        let (w, h) = self.render_size;
        if w == 0 || h == 0 {
            4.0 / 3.0
        } else {
            w as f64 / h as f64
        }
    }

    /// Switches between a parameter-plane type and its Julia sets: to the
    /// Julia set for the point under the pointer (or the view's centre), or
    /// back to the parameter-plane view it came from.
    fn toggle_julia(&mut self) {
        let w = self.params.window;
        if types::julia_of(self.params.fractal).is_some() {
            let (u, v) = self.hover.unwrap_or((0.5, 0.5));
            let (x, y) = view::point_at(&w, u, v);
            let Some(mut p) = self.params.to_julia(fractalforge_core::Complex::new(x, y)) else {
                return;
            };
            self.mandel_views
                .push((self.params.fractal.name, ViewPos::of(&self.params)));
            p.window = view::fit_aspect(&p.window, self.aspect());
            self.params = p;
        } else {
            let saved = self
                .mandel_views
                .pop()
                .filter(|(name, _)| {
                    types::mandel_of(self.params.fractal).is_some_and(|m| m.name == *name)
                })
                .map(|(_, v)| v);
            let Some(mut p) = self.params.to_mandel(saved.as_ref().map(|v| v.window)) else {
                return;
            };
            match saved {
                Some(v) => v.restore(&mut p),
                None => p.window = view::fit_aspect(&p.window, self.aspect()),
            }
            self.params = p;
        }
        self.changed();
    }

    fn julia_toggle_label(&self) -> Option<&'static str> {
        if types::julia_of(self.params.fractal).is_some() {
            Some("Julia Set at Pointer")
        } else if types::mandel_of(self.params.fractal).is_some() {
            Some("Back to Parameter Plane")
        } else {
            None
        }
    }

    fn menu(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open…").clicked() {
                    self.open_dialog(&ctx);
                }
                ui.separator();
                if ui.button("Save").clicked() {
                    self.save(None, false, &ctx);
                }
                if ui.button("Save As…").clicked() {
                    self.save_as(false, &ctx);
                }
                if ui.button("Save and Stop").clicked() {
                    self.save(None, true, &ctx);
                }
                if ui.button("Export PNG…").clicked() {
                    self.export_png();
                }
                ui.separator();
                if ui.button("Save Parameters (.par)…").clicked() {
                    self.save_par_dialog();
                }
                if ui.button("Export Palette (.map)…").clicked() {
                    self.export_map();
                }
                ui.separator();
                ui.checkbox(&mut self.autosave, "Autosave while rendering");
                ui.add_enabled(
                    self.autosave,
                    egui::Slider::new(&mut self.autosave_minutes, 1..=60).suffix(" min"),
                );
            });
            ui.menu_button("Edit", |ui| {
                if ui
                    .add_enabled(self.history_pos > 1, egui::Button::new("Undo"))
                    .clicked()
                {
                    self.undo();
                }
                if ui
                    .add_enabled(
                        self.history_pos < self.history.len(),
                        egui::Button::new("Redo"),
                    )
                    .clicked()
                {
                    self.redo();
                }
            });
            ui.menu_button("Fractal", |ui| {
                let label = self.julia_toggle_label();
                if ui
                    .add_enabled(
                        label.is_some(),
                        egui::Button::new(label.unwrap_or("Julia Set")).shortcut_text("Space"),
                    )
                    .on_hover_text("Uses the view's centre when chosen from this menu")
                    .clicked()
                {
                    self.hover = None;
                    self.toggle_julia();
                }
                ui.checkbox(&mut self.show_orbit, "Show Orbits (O)")
                    .on_hover_text("Draws the orbit of the point under the pointer");
            });
            ui.menu_button("Image", |ui| {
                if ui.button("3D Transform…").clicked() {
                    self.effects.open_three_d();
                }
                if ui.button("Stereogram…").clicked() {
                    self.effects.open_stereogram();
                }
            });
            ui.menu_button("Render", |ui| {
                if ui
                    .add_enabled(self.worker.is_some(), egui::Button::new("Stop"))
                    .clicked()
                {
                    self.pause();
                }
                let resumable = self.stopped.as_ref().is_some_and(|r| !r.is_complete());
                if ui
                    .add_enabled(resumable, egui::Button::new("Resume"))
                    .clicked()
                {
                    self.resume(&ctx);
                }
                if ui.button("Restart").clicked() {
                    self.restart(&ctx);
                }
            });
        });
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        let mut changed = false;
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Fractal");
            egui::ComboBox::from_label("Type")
                .selected_text(match &self.params.program {
                    Some(p) => format!("{}: {}", self.params.fractal.name, p.0.name()),
                    None => self.params.fractal.name.to_owned(),
                })
                .show_ui(ui, |ui| {
                    // Formula types need a loaded .frm entry, not a menu pick.
                    for t in types::all().iter().filter(|t| !t.needs_program) {
                        if ui
                            .selectable_label(self.params.fractal == *t, t.name)
                            .on_hover_text(t.description)
                            .clicked()
                            && self.params.fractal != *t
                        {
                            self.params = Params::new(t);
                            self.params.window = view::fit_aspect(&t.default_window, self.aspect());
                            changed = true;
                        }
                    }
                });
            // For formulas, only the p1–p5 pairs the formula uses.
            let program = self.params.program.clone();
            for (i, name) in self.params.fractal.param_names.iter().enumerate() {
                if let Some(prog) = &program
                    && self.params.fractal == &types::FORMULA
                    && !prog.0.mentions(&format!("p{}", i / 2 + 1))
                {
                    continue;
                }
                ui.label(*name);
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut self.params.params[i])
                            .speed(0.001)
                            .max_decimals(15),
                    )
                    .changed();
            }
            if self.params.uses_view3d() {
                let v = &mut self.params.view3d;
                ui.horizontal(|ui| {
                    ui.label("Rotation x/y/z");
                    for r in &mut v.rotation {
                        changed |= ui
                            .add(egui::DragValue::new(r).range(-360..=360).suffix("°"))
                            .changed();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Shift right/up");
                    for s in &mut v.xyshift {
                        changed |= ui
                            .add(egui::DragValue::new(s).range(-100..=100).suffix("%"))
                            .changed();
                    }
                });
            }
            ui.horizontal(|ui| {
                ui.label("Max iterations");
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut self.params.max_iter)
                            .range(1..=1_000_000_000)
                            .speed(10),
                    )
                    .changed();
            });
            if self.params.fractal.uses_bailout {
                ui.horizontal(|ui| {
                    ui.label("Bailout");
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut self.params.bailout)
                                .range(0.0001..=1e12)
                                .speed(0.1),
                        )
                        .changed();
                });
            }
            // Plotted types draw the image: no pixel options.
            if self.params.fractal.plot.is_none() {
                changed |= ui
                    .checkbox(&mut self.params.use_symmetry, "Use symmetry")
                    .on_hover_text(
                        "Copy mirrored pixels instead of computing them, as Fractint does",
                    )
                    .changed();
                changed |= self.coloring_controls(ui);
                egui::ComboBox::from_label("Passes")
                    .selected_text(pass_name(self.pass_mode))
                    .show_ui(ui, |ui| {
                        for m in [PassMode::One, PassMode::Two, PassMode::Guess] {
                            changed |= ui
                                .selectable_value(&mut self.pass_mode, m, pass_name(m))
                                .changed();
                        }
                    });
            }

            ui.separator();
            ui.heading("View");
            match &self.params.deep {
                Some(d) => {
                    let digits = d.digits();
                    let (re, im) = (d.re.to_decimal(digits), d.im.to_decimal(digits));
                    let im = if im.starts_with('-') {
                        im
                    } else {
                        format!("+{im}")
                    };
                    let short = |v: &str| {
                        if v.chars().count() > 20 {
                            format!("{}…", v.chars().take(19).collect::<String>())
                        } else {
                            v.to_owned()
                        }
                    };
                    ui.monospace(format!("centre {}\n       {}i", short(&re), short(&im)))
                        .on_hover_text(format!("{re}\n{im}i"));
                    ui.horizontal(|ui| {
                        ui.monospace(format!("magnification {:.4e}", 1.0 / d.half_h));
                        if ui
                            .small_button("Copy centre")
                            .on_hover_text(format!("{} digits, as X/Y", digits))
                            .clicked()
                        {
                            ui.ctx()
                                .copy_text(format!("{re}/{}", im.trim_start_matches('+')));
                        }
                    });
                }
                None => {
                    let w = &self.params.window;
                    let (cx, cy) = ((w.x_min + w.x_max) / 2.0, (w.y_min + w.y_max) / 2.0);
                    ui.monospace(format!("centre {cx:.15}\n       {cy:+.15}i"));
                    ui.monospace(format!("magnification {:.4e}", view::magnification(w)));
                }
            }
            if ui.button("Reset view (Home)").clicked() {
                self.params.deep = None;
                self.params.window =
                    view::fit_aspect(&self.params.fractal.default_window, self.aspect());
                changed = true;
            }
            let size_text = match self.size_mode {
                SizeMode::FitWindow => "Fit window".to_owned(),
                SizeMode::Fixed(w, h) => format!("{w}×{h}"),
            };
            egui::ComboBox::from_label("Image size")
                .selected_text(size_text)
                .show_ui(ui, |ui| {
                    changed |= ui
                        .selectable_value(&mut self.size_mode, SizeMode::FitWindow, "Fit window")
                        .changed();
                    for &(w, h) in SIZES {
                        changed |= ui
                            .selectable_value(
                                &mut self.size_mode,
                                SizeMode::Fixed(w, h),
                                format!("{w}×{h}"),
                            )
                            .changed();
                    }
                });

            ui.separator();
            ui.heading("Colours");
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.cycling, "Cycle (C)");
                ui.add(
                    egui::Slider::new(&mut self.cycle_speed, 1.0..=500.0)
                        .logarithmic(true)
                        .suffix("/s"),
                );
            });
            let mut palette_changed = false;
            egui::ComboBox::from_label("Preset")
                .selected_text("Choose…")
                .show_ui(ui, |ui| {
                    for (name, keys) in PRESETS {
                        if ui.selectable_label(false, *name).clicked() {
                            self.palette_keys = keys.to_vec();
                            palette_changed = true;
                        }
                    }
                });
            let mut remove = None;
            for (i, key) in self.palette_keys.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    palette_changed |= ui.color_edit_button_srgb(key).changed();
                    if ui.small_button("Remove").clicked() {
                        remove = Some(i);
                    }
                });
            }
            if let Some(i) = remove.filter(|_| self.palette_keys.len() > 2) {
                self.palette_keys.remove(i);
                palette_changed = true;
            }
            if ui.button("Add colour").clicked() {
                let last = *self.palette_keys.last().unwrap();
                self.palette_keys.push(last);
                palette_changed = true;
            }
            if palette_changed {
                self.palette = Palette::from_keys(&self.palette_keys);
                self.recolor = true;
            }

            ui.separator();
            ui.collapsing("Help", |ui| {
                ui.label(
                    "Scroll or pinch: zoom\nDrag: pan\nShift-drag or right-drag: zoom box\n\
                     Double-click: zoom in ×2\nRight-click: zoom out ×2\nHome: reset view\n\
                     C: colour cycling, + / -: speed\nEsc: stop, Enter: resume\n\
                     Cmd+S save, Cmd+Shift+S save as, Cmd+O open\n\
                     Cmd+Z undo, Cmd+Shift+Z redo\n(Ctrl instead of Cmd on Linux and Windows)",
                );
            });
        });
        if changed {
            self.changed();
        }
    }

    /// Inside/outside/decomposition/biomorph. Only the default colouring
    /// is offered for types without orbit colouring.
    fn coloring_controls(&mut self, ui: &mut egui::Ui) -> bool {
        let full = self.params.fractal.orbit_coloring;
        let p = &mut self.params;
        let mut changed = false;
        let insides: &[Inside] = if full {
            &Inside::ALL
        } else {
            &Inside::ALL[..2]
        };
        ui.horizontal(|ui| {
            egui::ComboBox::from_label("Inside")
                .selected_text(p.inside.name())
                .show_ui(ui, |ui| {
                    for &m in insides {
                        let same = std::mem::discriminant(&m) == std::mem::discriminant(&p.inside);
                        if ui.selectable_label(same, m.name()).clicked() && !same {
                            p.inside = m;
                            changed = true;
                        }
                    }
                });
            if let Inside::Color(c) = &mut p.inside {
                changed |= ui.add(egui::DragValue::new(c)).changed();
            }
        });
        ui.add_enabled_ui(full, |ui| {
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Outside")
                    .selected_text(p.outside.name())
                    .show_ui(ui, |ui| {
                        for m in Outside::ALL {
                            let same =
                                std::mem::discriminant(&m) == std::mem::discriminant(&p.outside);
                            if ui.selectable_label(same, m.name()).clicked() && !same {
                                p.outside = m;
                                changed = true;
                            }
                        }
                    });
                if let Outside::Color(c) = &mut p.outside {
                    changed |= ui.add(egui::DragValue::new(c)).changed();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Decomposition");
                changed |= ui
                    .add(egui::DragValue::new(&mut p.decomp).range(0..=65535))
                    .on_hover_text("Colour by angle in this many sectors; 0 = off")
                    .changed();
            });
            ui.horizontal(|ui| {
                let mut on = p.biomorph.is_some();
                if ui.checkbox(&mut on, "Biomorph").changed() {
                    // Biomorph's customary bailout is 100 (coloring.md §8).
                    let was_default = p.bailout == p.default_bailout();
                    p.biomorph = on.then_some(0);
                    if was_default {
                        p.bailout = p.default_bailout();
                    }
                    changed = true;
                }
                if let Some(b) = &mut p.biomorph {
                    changed |= ui.add(egui::DragValue::new(b).range(0..=32767)).changed();
                }
            });
        });
        changed
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let secs = self.elapsed_us as f64 / 1e6;
            let (w, h) = self.render_size;
            let text = match self.status {
                Status::Idle => "Ready".to_owned(),
                Status::Rendering => format!("Rendering {:.1}%  {secs:.1}s", self.progress * 100.0),
                Status::Stopped => format!(
                    "Stopped at {:.1}% (Enter resumes; saving keeps it resumable)",
                    self.progress * 100.0
                ),
                Status::Complete => format!("Complete in {secs:.2}s"),
            };
            ui.label(text);
            ui.separator();
            ui.label(format!("{w}×{h}"));
            if self.status == Status::Rendering {
                ui.add(egui::ProgressBar::new(self.progress as f32).desired_width(160.0));
            }
            if !self.messages.is_empty() {
                ui.separator();
                let last = self.messages.last().unwrap().clone();
                ui.label(egui::RichText::new(last).color(ui.visuals().warn_fg_color));
                if ui.small_button("Clear").clicked() {
                    self.messages.clear();
                }
            }
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let avail = ui.available_rect_before_wrap();
        let size = self.canvas_size(avail, ctx.pixels_per_point());
        // A new image size (window resized, or a different fixed size)
        // keeps the view's height and centre and re-renders.
        if size != self.render_size {
            self.render_size = size;
            let aspect = self.aspect();
            self.params.reframe(|w| view::fit_aspect(w, aspect));
            self.schedule_restart();
        }

        // The image keeps its aspect ratio inside the available area.
        let aspect = self.aspect() as f32;
        let mut rect = avail;
        if rect.width() / rect.height() > aspect {
            rect = Rect::from_center_size(
                avail.center(),
                Vec2::new(rect.height() * aspect, rect.height()),
            );
        } else {
            rect = Rect::from_center_size(
                avail.center(),
                Vec2::new(rect.width(), rect.width() / aspect),
            );
        }
        let response = ui.allocate_rect(avail, Sense::click_and_drag());
        let painter = ui.painter_at(avail);
        painter.rect_filled(avail, 0.0, Color32::from_gray(16));

        let norm = |p: Pos2| {
            (
                (p.x - rect.left()) as f64 / rect.width() as f64,
                (p.y - rect.top()) as f64 / rect.height() as f64,
            )
        };
        let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
        for shown in [&self.backdrop, &self.image].into_iter().flatten() {
            let now = ViewPos::of(&self.params);
            let (u0, v0, u1, v1) = view::placement(&shown.view.relative_to(&now), &now.own_frame());
            let r = Rect::from_min_max(
                rect.lerp_inside(Vec2::new(u0 as f32, v0 as f32)),
                rect.lerp_inside(Vec2::new(u1 as f32, v1 as f32)),
            );
            painter.image(shown.texture.id(), r, uv, Color32::WHITE);
        }

        self.hover = response.hover_pos().filter(|p| rect.contains(*p)).map(norm);
        if self.show_orbit
            && self.drag.is_none()
            && let Some((u, v)) = self.hover
        {
            self.draw_orbit(&painter, rect, u, v);
        }

        // Zoom with scroll wheel or pinch, about the pointer.
        if let Some(pos) = response.hover_pos() {
            let (zoom, scroll) = ctx.input(|i| (i.zoom_delta(), i.smooth_scroll_delta.y));
            let factor = zoom as f64 * (scroll as f64 * 0.003).exp();
            if (factor - 1.0).abs() > 1e-6 {
                let (u, v) = norm(pos);
                self.params.reframe(|w| view::zoom_about(w, u, v, factor));
                self.changed();
            }
        }

        let shift = ctx.input(|i| i.modifiers.shift);
        if response.drag_started() {
            let start = response.interact_pointer_pos().unwrap_or_default();
            self.drag = Some(if shift || response.dragged_by(PointerButton::Secondary) {
                Drag::ZoomBox { start }
            } else {
                Drag::Pan {
                    start,
                    view: ViewPos::of(&self.params),
                }
            });
        }
        if let (Some(drag), Some(pos)) = (self.drag.clone(), response.interact_pointer_pos()) {
            match drag {
                Drag::Pan { start, view: from } => {
                    let d = pos - start;
                    from.restore(&mut self.params);
                    let (du, dv) = ((d.x / rect.width()) as f64, (d.y / rect.height()) as f64);
                    self.params.reframe(|w| view::pan(w, du, dv));
                }
                Drag::ZoomBox { start } => {
                    let r = aspect_box(start, pos, aspect);
                    painter.rect_stroke(
                        r,
                        0.0,
                        Stroke::new(1.5, Color32::WHITE),
                        egui::StrokeKind::Middle,
                    );
                    if response.drag_stopped() && r.width() > 4.0 {
                        let (u0, v0) = norm(r.min);
                        let (u1, v1) = norm(r.max);
                        self.params.reframe(|w| view::sub_window(w, u0, v0, u1, v1));
                    }
                }
            }
        }
        if response.drag_stopped() {
            self.drag = None;
            self.changed();
        }
        if response.double_clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            let (u, v) = norm(pos);
            self.params.reframe(|w| view::zoom_about(w, u, v, 2.0));
            self.changed();
        } else if response.secondary_clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            let (u, v) = norm(pos);
            self.params.reframe(|w| view::zoom_about(w, u, v, 0.5));
            self.changed();
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.shortcuts(&ctx);
        self.poll_worker(&ctx);

        if self.cycling {
            let dt = ctx.input(|i| i.stable_dt).min(0.1);
            self.cycle_offset = (self.cycle_offset + self.cycle_speed * dt).rem_euclid(255.0);
            self.recolor = true;
            ctx.request_repaint();
        }
        self.update_texture(&ctx);

        egui::Panel::top("menu").show(ui, |ui| self.menu(ui));
        egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui));
        egui::Panel::left("controls")
            .resizable(true)
            .default_size(260.0)
            .show(ui, |ui| self.controls(ui));
        egui::CentralPanel::no_frame().show(ui, |ui| self.canvas(ui));
        self.dialogs(&ctx);
        let palette = self.palette.rotated(self.cycle_offset as i32);
        let mut effects = std::mem::take(&mut self.effects);
        let mut messages = Vec::new();
        effects.show(&ctx, || self.picture(), &palette, &mut messages);
        self.effects = effects;
        self.messages.extend(messages);

        if let Some(at) = self.restart_at {
            let now = Instant::now();
            if now >= at && self.drag.is_none() {
                self.restart(&ctx);
            } else {
                ctx.request_repaint_after(at.saturating_duration_since(now));
            }
        }
        self.autosave(&ctx);
        self.update_title(&ctx);
        self.screenshot_hook(&ctx);
    }
}

impl App {
    fn screenshot_hook(&mut self, ctx: &egui::Context) {
        let Some((path, requested)) = &mut self.screenshot else {
            return;
        };
        if *requested || self.status != Status::Complete || self.restart_at.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
            return;
        }
        // FRACTALFORGE_EFFECT=3d or stereogram: show that effect's result.
        if let Ok(effect) = std::env::var("FRACTALFORGE_EFFECT")
            && !self.effects.has_result()
        {
            if effect == "3d" {
                self.effects.open_three_d();
            } else {
                self.effects.open_stereogram();
            }
            self.effects.request_make();
            ctx.request_repaint();
            return;
        }
        *requested = true;
        let (path, ctx2) = (path.clone(), ctx.clone());
        ctx.request_screenshot(move |img| {
            let mut ppm = format!("P6 {} {} 255\n", img.size[0], img.size[1]).into_bytes();
            for c in &img.pixels {
                ppm.extend_from_slice(&[c.r(), c.g(), c.b()]);
            }
            if let Err(e) = std::fs::write(&path, ppm) {
                eprintln!("screenshot: {}: {e}", path.display());
            }
            ctx2.send_viewport_cmd(egui::ViewportCommand::Close);
        });
        ctx.request_repaint();
    }
}

/// The kind of entry file (.frm, .ifs, .l) at `path`, or `None` for
/// others (.par).
fn entry_file_kind(path: &Path) -> Option<Kind> {
    let ext = path.extension()?.to_str()?;
    Kind::ALL
        .into_iter()
        .find(|k| ext.eq_ignore_ascii_case(k.extension()))
}

impl App {
    /// The picture on screen as palette indices (pixels not computed yet
    /// as 0), with its Fractint parameter block.
    fn picture(&self) -> Option<crate::effects::Picture> {
        let f = self.frame.as_ref()?;
        let pixels = f
            .indices
            .iter()
            .map(|&i| if i == TRANSPARENT { 0 } else { i as u8 })
            .collect();
        let (w, h) = (f.width as u32, f.height as u32);
        let main = gif::fractint_blocks::write(&self.params, self.pass_mode, None, w, h, 0)
            .and_then(|(blocks, _)| blocks.last().map(|b| b.data()));
        Some(crate::effects::Picture {
            pixels,
            width: w,
            height: h,
            main_block: main,
        })
    }
}

/// Most iterates the orbit window draws.
const ORBIT_POINTS: u32 = 5000;

impl App {
    /// Draws the orbit of the point at normalised position `(u, v)`, as
    /// dots joined by faint lines, with its start and length in a label.
    fn draw_orbit(&self, painter: &egui::Painter, rect: Rect, u: f64, v: f64) {
        let w = self.params.window;
        let (x, y) = view::point_at(&w, u, v);
        let (width, height) = self.render_size;
        let at = fractalforge_core::PixelContext {
            pixel: fractalforge_core::Complex::new(x, y),
            col: (u * width.saturating_sub(1) as f64).round() as u32,
            row: (v * height.saturating_sub(1) as f64).round() as u32,
            width,
            height,
        };
        let Some(points) = self.params.orbit(&at, ORBIT_POINTS) else {
            return;
        };
        let to_screen = |z: fractalforge_core::Complex| {
            let pu = (z.re - w.x_min) / (w.x_max - w.x_min);
            let pv = (w.y_max - z.im) / (w.y_max - w.y_min);
            rect.lerp_inside(Vec2::new(pu as f32, pv as f32))
        };
        let finite =
            |p: &Pos2| p.x.is_finite() && p.y.is_finite() && p.x.abs() < 1e6 && p.y.abs() < 1e6;
        let screen: Vec<Pos2> = std::iter::once(at.pixel)
            .chain(points.iter().copied())
            .map(to_screen)
            .take_while(finite)
            .collect();
        let line = Stroke::new(1.0, Color32::from_white_alpha(70));
        for pair in screen.windows(2) {
            painter.line_segment([pair[0], pair[1]], line);
        }
        for p in &screen[1..] {
            painter.circle_filled(*p, 1.5, Color32::YELLOW);
        }
        painter.circle_stroke(screen[0], 3.0, Stroke::new(1.5, Color32::WHITE));
        // With the plain colouring, a value tells escaped from inside.
        let plain = Params {
            inside: Inside::Color(0),
            outside: Outside::Iter,
            decomp: 0,
            biomorph: None,
            ..self.params.clone()
        };
        let value = match &plain.program {
            Some(prog) => {
                let mut carry = prog.0.new_carry(&plain, width, height);
                prog.0.iterate(&at, &plain, &mut carry)
            }
            None => (plain.fractal.iterate)(at.pixel, &plain),
        };
        let escaped = value != fractalforge_core::INSIDE;
        let text = format!(
            "{x:.6} {y:+.6}i\n{} iterates{}",
            points.len(),
            if escaped { "" } else { " (no escape)" }
        );
        painter.text(
            rect.left_top() + Vec2::new(8.0, 8.0),
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::monospace(12.0),
            Color32::WHITE,
        );
    }
}

/// A rectangle from `a` towards `b` with the image's aspect ratio.
fn aspect_box(a: Pos2, b: Pos2, aspect: f32) -> Rect {
    let d = b - a;
    let w = d.x.abs().max(d.y.abs() * aspect);
    let h = w / aspect;
    Rect::from_two_pos(a, a + Vec2::new(w.copysign(d.x), h.copysign(d.y)))
}

fn pass_name(m: PassMode) -> &'static str {
    match m {
        PassMode::One => "1 pass",
        PassMode::Two => "2 passes",
        PassMode::Guess => "Guessing",
    }
}

fn write_gif(path: &Path, c: &Checkpoint, complete: bool) -> Result<Vec<String>, String> {
    let saved = gif::save(c, complete).map_err(|e| e.to_string())?;
    // Write to a temporary file first so a failed save never destroys the
    // previous one.
    let tmp = path.with_extension("gif.partial");
    std::fs::write(&tmp, &saved.bytes)
        .and_then(|()| std::fs::rename(&tmp, path))
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(saved.notes)
}

fn autosave_path(file: Option<&Path>) -> PathBuf {
    match file {
        Some(f) => f.with_extension("autosave.gif"),
        None => std::env::home_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("FractalForge autosave.gif"),
    }
}
