// SPDX-License-Identifier: GPL-3.0-or-later

//! Parallel renderer producing an iteration buffer.
//!
//! Every pass mode keeps its progress as plain data ([`RenderState`]) so a
//! render can be checkpointed at any batch boundary and resumed with
//! pixel-identical results.

pub mod checkpoint;
mod plan;

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use fractalforge_core::{IterBuffer, Params};
use rayon::prelude::*;

pub use checkpoint::{PassMode, RenderState};
use plan::Plan;

/// Rows processed between checkpoint opportunities.
const BATCH_ROWS: u32 = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResumeError {
    SizeMismatch,
    BadState(String),
    BadParams(String),
    NotResumable(&'static str),
}

impl fmt::Display for ResumeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SizeMismatch => write!(f, "iteration buffer size does not match the image"),
            Self::BadState(m) => write!(f, "invalid render state: {m}"),
            Self::BadParams(m) => write!(f, "invalid parameters: {m}"),
            Self::NotResumable(t) => write!(f, "fractal type {t} cannot be resumed"),
        }
    }
}

impl std::error::Error for ResumeError {}

/// When [`Render::run`] should stop early.
#[derive(Default)]
pub struct Control<'a> {
    /// Set from another thread to stop at the next batch boundary.
    pub cancel: Option<&'a AtomicBool>,
    /// Stop once this fraction (0..=1) of the work is done.
    pub stop_after: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Complete,
    Stopped,
}

pub struct Render {
    params: Params,
    state: RenderState,
    buffer: IterBuffer,
    plan: Plan,
}

impl Render {
    pub fn new(
        params: Params,
        width: u32,
        height: u32,
        pass_mode: PassMode,
    ) -> Result<Self, ResumeError> {
        if width == 0 || height == 0 {
            return Err(ResumeError::SizeMismatch);
        }
        let state = RenderState::new(pass_mode, width, height);
        Self::resume(params, state, IterBuffer::new(width, height))
    }

    /// Continues from a checkpoint.
    pub fn resume(
        params: Params,
        state: RenderState,
        buffer: IterBuffer,
    ) -> Result<Self, ResumeError> {
        params.validate().map_err(ResumeError::BadParams)?;
        let (width, height) = (buffer.width(), buffer.height());
        if width == 0 || height == 0 {
            return Err(ResumeError::SizeMismatch);
        }
        let step_ok = state.coarsest_step.is_power_of_two()
            && match state.pass_mode {
                PassMode::One => state.coarsest_step == 1,
                PassMode::Two => state.coarsest_step == 2,
                PassMode::Guess => (2..=256).contains(&state.coarsest_step),
            };
        if !step_ok {
            return Err(ResumeError::BadState(format!(
                "coarsest step {} for {:?}",
                state.coarsest_step, state.pass_mode
            )));
        }
        let plan = Plan::new(&params, width, height, &state);
        let stage = state.stage as usize;
        let in_range = stage < plan.stages.len() && state.row < plan.stage_rows(stage)
            || stage == plan.stages.len() && state.row == 0;
        if !in_range {
            return Err(ResumeError::BadState(format!(
                "position stage {} row {} is outside the plan",
                state.stage, state.row
            )));
        }
        let fresh = state.stage == 0 && state.row == 0;
        if !fresh && !params.fractal.resumable {
            return Err(ResumeError::NotResumable(params.fractal.name));
        }
        Ok(Self {
            params,
            state,
            buffer,
            plan,
        })
    }

    pub fn params(&self) -> &Params {
        &self.params
    }

    pub fn buffer(&self) -> &IterBuffer {
        &self.buffer
    }

    /// The current progress, to save alongside [`Render::buffer`].
    pub fn checkpoint(&self) -> RenderState {
        self.state.clone()
    }

    pub fn into_parts(self) -> (Params, RenderState, IterBuffer) {
        (self.params, self.state, self.buffer)
    }

    pub fn is_complete(&self) -> bool {
        self.state.stage as usize == self.plan.stages.len()
    }

    /// Fraction of the work done, 0..=1.
    pub fn progress(&self) -> f64 {
        let total = self.plan.total_rows();
        if total == 0 {
            1.0
        } else {
            self.plan.rows_done(&self.state) as f64 / total as f64
        }
    }

    /// Processes up to [`BATCH_ROWS`] rows of the current stage.
    pub fn step(&mut self) {
        self.step_cancellable(&AtomicBool::new(false));
    }

    /// Like [`Render::step`], but gives up mid-batch once `cancel` is set,
    /// leaving the state as it was. Returns whether the batch was applied.
    pub fn step_cancellable(&mut self, cancel: &AtomicBool) -> bool {
        if self.is_complete() {
            return true;
        }
        let started = Instant::now();
        let stage = self.state.stage as usize;
        let rows = self.plan.stage_rows(stage);
        let end = (self.state.row + BATCH_ROWS).min(rows);

        let (plan, params, buffer) = (&self.plan, &self.params, &self.buffer);
        let results: Option<Vec<_>> = (self.state.row..end)
            .into_par_iter()
            .map(|i| {
                (!cancel.load(Ordering::Relaxed)).then(|| plan.row_work(stage, i, params, buffer))
            })
            .collect();
        let Some(results) = results.filter(|_| !cancel.load(Ordering::Relaxed)) else {
            return false;
        };
        for (y, values) in &results {
            self.plan.apply(&mut self.buffer, *y, values, &self.params);
        }

        self.state.row = end;
        if end == rows {
            self.state.stage += 1;
            self.state.row = 0;
        }
        self.state.elapsed_us += started.elapsed().as_micros() as u64;
        true
    }

    /// Renders until complete, cancelled, or past `stop_after`.
    pub fn run(&mut self, control: &Control) -> Outcome {
        while !self.is_complete() {
            if control.cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
                return Outcome::Stopped;
            }
            if control.stop_after.is_some_and(|f| self.progress() >= f) {
                return Outcome::Stopped;
            }
            if let Some(cancel) = control.cancel {
                self.step_cancellable(cancel);
            } else {
                self.step();
            }
        }
        Outcome::Complete
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fractalforge_core::{UNKNOWN, Window};

    fn mandel() -> Params {
        Params::for_type("mandel").unwrap()
    }

    fn julia() -> Params {
        let mut p = Params::for_type("julia").unwrap();
        p.params = [-0.8, 0.156, 0.0, 0.0];
        p
    }

    fn full(params: &Params, w: u32, h: u32, mode: PassMode) -> IterBuffer {
        let mut r = Render::new(params.clone(), w, h, mode).unwrap();
        assert_eq!(r.run(&Control::default()), Outcome::Complete);
        r.buffer().clone()
    }

    fn all_types() -> Vec<Params> {
        fractalforge_core::types::all()
            .iter()
            .map(|t| Params::new(t))
            .collect()
    }

    /// Off-centre views exercise every symmetry kind's partial copying.
    fn off_centre(mut p: Params) -> Params {
        let w = p.window;
        let (dx, dy) = ((w.x_max - w.x_min) * 0.17, (w.y_max - w.y_min) * 0.23);
        p.window = Window::new(w.x_min + dx, w.x_max + dx, w.y_min + dy, w.y_max + dy * 0.5);
        p
    }

    #[test]
    fn all_types_fill_every_pixel_and_resume_identically() {
        for base in all_types() {
            for params in [base.clone(), off_centre(base)] {
                for mode in [PassMode::One, PassMode::Guess] {
                    let expected = full(&params, 61, 47, mode);
                    assert_eq!(
                        expected.unknown_count(),
                        0,
                        "{} {mode:?}",
                        params.fractal.name
                    );
                    let mut r = Render::new(params.clone(), 61, 47, mode).unwrap();
                    r.run(&Control {
                        stop_after: Some(0.4),
                        ..Default::default()
                    });
                    let (p, s, b) = r.into_parts();
                    let mut r = Render::resume(p, s, b).unwrap();
                    r.run(&Control::default());
                    assert_eq!(r.buffer(), &expected, "{} {mode:?}", params.fractal.name);
                }
            }
        }
    }

    #[test]
    fn every_pixel_is_filled() {
        for params in [mandel(), julia()] {
            for mode in [PassMode::One, PassMode::Two, PassMode::Guess] {
                for (w, h) in [(1, 1), (2, 3), (37, 23), (64, 48), (101, 77)] {
                    let b = full(&params, w, h, mode);
                    assert_eq!(
                        b.unknown_count(),
                        0,
                        "{} {mode:?} {w}x{h}",
                        params.fractal.name
                    );
                }
            }
        }
    }

    /// Without symmetry or guessing, every pixel is exactly computed, so all
    /// modes that don't guess must agree with a direct evaluation.
    #[test]
    fn exact_modes_match_direct_evaluation() {
        let mut p = mandel();
        p.params[1] = 0.25; // breaks symmetry
        let (w, h) = (41, 29);
        let one = full(&p, w, h, PassMode::One);
        assert_eq!(one, full(&p, w, h, PassMode::Two));
        let dx = (p.window.x_max - p.window.x_min) / (w - 1) as f64;
        let dy = (p.window.y_max - p.window.y_min) / (h - 1) as f64;
        for y in 0..h {
            for x in 0..w {
                let c = fractalforge_core::Complex::new(
                    p.window.x_min + x as f64 * dx,
                    p.window.y_max - y as f64 * dy,
                );
                assert_eq!(one.get(x, y), (p.fractal.iterate)(c, &p));
            }
        }
    }

    #[test]
    fn symmetric_window_mirrors_exactly() {
        // y range symmetric about 0 with an odd row count: row 30 is im = 0.
        let mut p = mandel();
        p.window = Window::new(-2.0, 1.0, -1.2, 1.2);
        let b = full(&p, 40, 61, PassMode::One);
        for y in 0..61 {
            assert_eq!(b.row(y), b.row(60 - y));
        }
        let mut j = julia();
        j.window = Window::new(-1.5, 1.5, -1.0, 1.0);
        let b = full(&j, 41, 31, PassMode::One);
        for y in 0..31 {
            for x in 0..41 {
                assert_eq!(b.get(x, y), b.get(40 - x, 30 - y));
            }
        }
    }

    #[test]
    fn guessing_is_close_to_exact() {
        let p = mandel();
        let exact = full(&p, 160, 120, PassMode::One);
        let guessed = full(&p, 160, 120, PassMode::Guess);
        let diff = exact
            .data()
            .iter()
            .zip(guessed.data())
            .filter(|(a, b)| a != b)
            .count();
        assert!(diff < exact.data().len() / 100, "{diff} pixels differ");
    }

    /// Stop at many points, rebuild from the checkpoint, and finish: the
    /// result must match an uninterrupted render bit for bit.
    #[test]
    fn resume_is_pixel_identical() {
        for params in [mandel(), julia()] {
            for mode in [PassMode::One, PassMode::Two, PassMode::Guess] {
                let (w, h) = (97, 71);
                let expected = full(&params, w, h, mode);
                for stop in [0.0, 0.01, 0.2, 0.5, 0.77, 0.99] {
                    let mut r = Render::new(params.clone(), w, h, mode).unwrap();
                    let control = Control {
                        stop_after: Some(stop),
                        ..Default::default()
                    };
                    let outcome = r.run(&control);
                    if stop < 0.5 {
                        assert_eq!(outcome, Outcome::Stopped);
                    }
                    let (p, state, buf) = r.into_parts();
                    let mut r = Render::resume(p, state, buf).unwrap();
                    r.run(&Control::default());
                    assert_eq!(
                        r.buffer(),
                        &expected,
                        "{} {mode:?} stop {stop}",
                        params.fractal.name
                    );
                }
            }
        }
    }

    #[test]
    fn single_steps_resume_identically() {
        let p = julia();
        let expected = full(&p, 64, 64, PassMode::Guess);
        let mut r = Render::new(p, 64, 64, PassMode::Guess).unwrap();
        while !r.is_complete() {
            r.step();
            let (p, s, b) = r.into_parts();
            r = Render::resume(p, s, b).unwrap();
        }
        assert_eq!(r.buffer(), &expected);
    }

    #[test]
    fn cancelled_step_changes_nothing() {
        let flag = AtomicBool::new(true);
        let mut r = Render::new(mandel(), 50, 50, PassMode::Guess).unwrap();
        assert!(!r.step_cancellable(&flag));
        assert_eq!(r.checkpoint(), RenderState::new(PassMode::Guess, 50, 50));
        assert_eq!(r.buffer().unknown_count(), 2500);
    }

    #[test]
    fn cancel_stops_immediately() {
        let flag = AtomicBool::new(true);
        let mut r = Render::new(mandel(), 50, 50, PassMode::One).unwrap();
        let control = Control {
            cancel: Some(&flag),
            ..Default::default()
        };
        assert_eq!(r.run(&control), Outcome::Stopped);
        assert_eq!(r.buffer().unknown_count(), 2500);
        assert!(r.buffer().data().iter().all(|&v| v == UNKNOWN));
    }

    #[test]
    fn rejects_bad_checkpoints() {
        let p = mandel();
        let mut s = RenderState::new(PassMode::Guess, 64, 64);
        s.stage = 99;
        assert!(Render::resume(p.clone(), s, IterBuffer::new(64, 64)).is_err());
        let mut s = RenderState::new(PassMode::One, 64, 64);
        s.coarsest_step = 4;
        assert!(Render::resume(p, s, IterBuffer::new(64, 64)).is_err());
    }

    #[test]
    fn progress_is_monotonic() {
        let mut r = Render::new(julia(), 80, 60, PassMode::Guess).unwrap();
        let mut last = r.progress();
        assert_eq!(last, 0.0);
        while !r.is_complete() {
            r.step();
            assert!(r.progress() >= last);
            last = r.progress();
        }
        assert_eq!(last, 1.0);
    }
}
