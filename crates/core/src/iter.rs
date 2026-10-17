// SPDX-License-Identifier: GPL-3.0-or-later

//! The iteration buffer: one escape count per pixel, kept separate from
//! colouring so palette edits and colour cycling never re-render.

/// The point never escaped within the iteration limit.
pub const INSIDE: u32 = u32::MAX - 1;
/// The pixel has not been computed yet.
pub const UNKNOWN: u32 = u32::MAX;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IterBuffer {
    width: u32,
    height: u32,
    data: Vec<u32>,
}

impl IterBuffer {
    /// A buffer with every pixel [`UNKNOWN`].
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![UNKNOWN; width as usize * height as usize],
        }
    }

    pub fn from_data(width: u32, height: u32, data: Vec<u32>) -> Option<Self> {
        (data.len() == width as usize * height as usize).then_some(Self {
            width,
            height,
            data,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn data(&self) -> &[u32] {
        &self.data
    }

    pub fn get(&self, x: u32, y: u32) -> u32 {
        self.data[self.index(x, y)]
    }

    pub fn set(&mut self, x: u32, y: u32, v: u32) {
        let i = self.index(x, y);
        self.data[i] = v;
    }

    pub fn row(&self, y: u32) -> &[u32] {
        let start = y as usize * self.width as usize;
        &self.data[start..start + self.width as usize]
    }

    pub fn unknown_count(&self) -> usize {
        self.data.iter().filter(|&&v| v == UNKNOWN).count()
    }

    fn index(&self, x: u32, y: u32) -> usize {
        debug_assert!(x < self.width && y < self.height);
        y as usize * self.width as usize + x as usize
    }
}
