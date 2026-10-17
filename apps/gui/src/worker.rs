// SPDX-License-Identifier: GPL-3.0-or-later

//! Background rendering: a thread drives a [`Render`] and publishes colour
//! index frames for display.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use fractalforge_core::{IterBuffer, Params, UNKNOWN, color_index};
use fractalforge_render::Render;
use rayon::prelude::*;

/// Palette index frame for display. [`TRANSPARENT`] marks pixels with
/// nothing to show yet.
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub indices: Vec<u16>,
    pub progress: f64,
    pub complete: bool,
    pub elapsed_us: u64,
}

pub const TRANSPARENT: u16 = u16::MAX;

/// How often a running render publishes a frame.
const PUBLISH_EVERY: Duration = Duration::from_millis(120);

pub struct Worker {
    cancel: Arc<AtomicBool>,
    handle: Option<JoinHandle<Render>>,
    frame: Arc<Mutex<Option<Frame>>>,
}

impl Worker {
    /// Runs `render` to completion (or until stopped), calling
    /// `on_frame` after each published frame.
    pub fn start(mut render: Render, on_frame: impl Fn() + Send + 'static) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let frame = Arc::new(Mutex::new(None));
        let (c, f) = (Arc::clone(&cancel), Arc::clone(&frame));
        let handle = std::thread::Builder::new()
            .name("render".into())
            .spawn(move || {
                let mut last = Instant::now();
                publish(&render, &f);
                on_frame();
                while !render.is_complete() && !c.load(Ordering::Relaxed) {
                    render.step_cancellable(&c);
                    if render.is_complete() || last.elapsed() >= PUBLISH_EVERY {
                        publish(&render, &f);
                        on_frame();
                        last = Instant::now();
                    }
                }
                render
            })
            .expect("can't start render thread");
        Self {
            cancel,
            handle: Some(handle),
            frame,
        }
    }

    /// The newest frame, if one was published since the last call.
    pub fn take_frame(&self) -> Option<Frame> {
        self.frame.lock().unwrap().take()
    }

    pub fn is_finished(&self) -> bool {
        self.handle.as_ref().is_none_or(|h| h.is_finished())
    }

    /// Stops at the next safe point and returns the render, ready to save
    /// or resume.
    pub fn stop(mut self) -> Render {
        self.cancel.store(true, Ordering::Relaxed);
        self.handle
            .take()
            .unwrap()
            .join()
            .expect("render thread panicked")
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

fn publish(render: &Render, slot: &Mutex<Option<Frame>>) {
    let frame = Frame {
        width: render.buffer().width() as usize,
        height: render.buffer().height() as usize,
        indices: preview_indices(render.buffer(), render.params()),
        progress: render.progress(),
        complete: render.is_complete(),
        elapsed_us: render.checkpoint().elapsed_us,
    };
    *slot.lock().unwrap() = Some(frame);
}

/// Colour indices for display. Pixels not computed yet borrow the value of
/// the nearest coarser-grid pixel above-left, so progressive passes show as
/// blocks instead of holes.
pub fn preview_indices(buf: &IterBuffer, params: &Params) -> Vec<u16> {
    let (w, h) = (buf.width() as usize, buf.height() as usize);
    let mut out = vec![TRANSPARENT; w * h];
    out.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        for (x, px) in row.iter_mut().enumerate() {
            let mut v = buf.get(x as u32, y as u32);
            let mut step = 2;
            while v == UNKNOWN && step <= 64 {
                v = buf.get((x - x % step) as u32, (y - y % step) as u32);
                step *= 2;
            }
            if v != UNKNOWN {
                *px = color_index(v, params) as u16;
            }
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use fractalforge_core::INSIDE;

    #[test]
    fn preview_fills_from_coarse_grid() {
        let p = Params::for_type("mandel").unwrap();
        let mut b = IterBuffer::new(5, 5);
        b.set(0, 0, 3);
        b.set(4, 4, INSIDE);
        let idx = preview_indices(&b, &p);
        assert_eq!(idx[0], color_index(3, &p) as u16);
        assert_eq!(idx[1 + 5], idx[0]); // (1,1) borrows (0,0)
        assert_eq!(idx[3 + 3 * 5], idx[0]); // (3,3) borrows (0,0) via step 4
        assert_eq!(idx[4 + 4 * 5], 0);
        assert_eq!(idx[4], idx[0]); // (4,0) borrows (0,0) via step 8
    }
}
