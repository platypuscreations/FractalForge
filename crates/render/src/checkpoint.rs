// SPDX-License-Identifier: GPL-3.0-or-later

//! Serializable render progress.
//!
//! A [`RenderState`] plus the iteration buffer is everything needed to resume
//! a render. Together with the parameters and image size it determines the
//! remaining work exactly, so a resumed render is pixel-identical to one that
//! was never interrupted.

/// How pixels are visited.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassMode {
    /// Every pixel, top to bottom.
    One,
    /// A half-resolution preview pass, then the remaining pixels.
    Two,
    /// Successive refinement from coarse blocks; blocks whose surroundings
    /// all have the same value are filled without being computed.
    Guess,
}

impl PassMode {
    pub fn tag(self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Guess => 3,
        }
    }

    pub fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            1 => Some(Self::One),
            2 => Some(Self::Two),
            3 => Some(Self::Guess),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderState {
    pub pass_mode: PassMode,
    /// Step of the coarsest pass, a power of two: 1 for one-pass, 2 for
    /// two-pass, larger for guessing.
    pub coarsest_step: u32,
    /// Index of the current stage in the render plan.
    pub stage: u32,
    /// Index of the next row to process within the current stage.
    pub row: u32,
    /// Calculation time so far, in microseconds.
    pub elapsed_us: u64,
}

impl RenderState {
    pub fn new(pass_mode: PassMode, width: u32, height: u32) -> Self {
        let coarsest_step = match pass_mode {
            PassMode::One => 1,
            PassMode::Two => 2,
            PassMode::Guess => default_guess_step(width, height),
        };
        Self {
            pass_mode,
            coarsest_step,
            stage: 0,
            row: 0,
            elapsed_us: 0,
        }
    }
}

fn default_guess_step(width: u32, height: u32) -> u32 {
    match width.min(height) {
        400.. => 16,
        100.. => 8,
        _ => 4,
    }
}
