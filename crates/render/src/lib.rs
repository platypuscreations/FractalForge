// SPDX-License-Identifier: GPL-3.0-or-later

//! Tile-based parallel renderer producing an iteration buffer.
//!
//! Every pass mode keeps its progress as plain data so a render can be
//! checkpointed and resumed with pixel-identical results.

pub mod checkpoint;
