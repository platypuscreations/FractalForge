<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# FractalForge native GIF block

Status: **payload version 3**. The block is written after the image data, before any Fractint-format blocks (implemented in `crates/formats/src/gif/native_block.rs`).

FractalForge stores a complete render checkpoint in its own GIF89a application
extension. Decoders that don't recognize an application extension skip it, so
the file stays a valid GIF in every viewer and in Fractint.

## Framing (fixed)

| Bytes | Value |
|-------|-------|
| 1 | `0x21` extension introducer |
| 1 | `0xFF` application extension label |
| 1 | `0x0B` block size (11) |
| 8 | application identifier `FRCTFRGE` (ASCII) |
| 3 | authentication code `1.0` (ASCII) |
| n | payload, split into data sub-blocks of 1–255 bytes, each prefixed by its length |
| 1 | `0x00` block terminator |

The identifier and authentication code are part of the file format. Never
change them. Payload versions are carried inside the payload, not in the
authentication code.

## Payload, version 3

The sub-blocks, concatenated, form the payload. All integers and floats are
little-endian; floats are IEEE 754 binary64.

| Bytes | Field |
|-------|-------|
| 2 | payload version, `3`. Version 2 (FractalForge 0.1.0) is still read: it has 4 type parameters and no fields marked *v3*. Version 1 (pre-release) is rejected |
| 1 | compression, `1` = raw DEFLATE (RFC 1951) |
| 8 | uncompressed body length in bytes |
| n | compressed body |

### Body

| Bytes | Field |
|-------|-------|
| 1 | length L of the fractal type name |
| L | type name, UTF-8 (e.g. `mandel`, `julia`) |
| 4×8 | window corners: x min, x max, y min, y max (centres of the edge pixels) |
| 10×8 | type parameters 1–10 (*v3*; v2 has 4) |
| 4 | maximum iterations |
| 8 | bailout: escape when \|z\|² exceeds this |
| 1+1 | inside colouring: mode, then palette index for mode 0. Modes: `0` fixed index, `1` maxiter, `2` zmag, `3` bof60, `4` bof61, `5` atan, `6` fmod, `7` epscross ([coloring.md](coloring.md) §6) |
| 1+1 | outside colouring: mode, then palette index for mode 1. Modes: `0` iter, `1` fixed index, `2` real, `3` imag, `4` mult, `5` summ, `6` atan, `7` fmod, `8` tdis (coloring.md §5) |
| 4 | decomposition sectors, `0` = off (coloring.md §7) |
| 8 | biomorph colour value as a signed integer, `-1` = off (coloring.md §8) |
| 1 | symmetry: `1` = copy mirrored pixels as Fractint does, `0` = compute every pixel |
| 1 | *v3* forced symmetry: `0` = the type's own, `1` x-axis, `2` y-axis, `3` xy, `4` origin, `5` none, `6` PI_SYM |
| 4 | *v3* fn1–fn4 bindings, as indices of Fractint's function table (gif-fractint-blocks.md §6.2) |
| 4 + n | *v3* formula entry name: length, then UTF-8 bytes (0 for non-formula types) |
| 4 + n | *v3* formula source: length, then the entry's full text, header to `}` (0 for non-formula types). FractalForge compiles it again on load, so formula GIFs don't need the `.frm` file |
| 4 | image width W |
| 4 | image height H |
| 1 | pass mode: `1` one pass, `2` two passes, `3` guessing |
| 4 | coarsest step (power of two: 1, 2, or the guessing block size) |
| 4 | current stage index |
| 4 | next row index within the stage |
| 8 | calculation time so far, microseconds |
| 768 | palette: 256 RGB triples |
| 4×W×H | value buffer, row-major: each pixel's colour value as defined per type in [types.md](types.md) and `crates/core/src/types.rs` (usually an escape count; values above 255 wrap through 1–255 when coloured), or `0xFFFFFFFE` = inside, `0xFFFFFFFF` = not computed |

A finished render has stage index equal to the number of stages and row 0.

### Resume semantics

The stage list is derived deterministically from the parameters, size, pass
mode and coarsest step (see `crates/render/src/plan.rs`): one stage per
power-of-two step from the coarsest down to 1, then a residue stage when
origin symmetry leaves pixels unmirrored. Resuming at (stage, row) with the
stored buffer gives exactly the pixels of an uninterrupted render. Any change
to the stage derivation or the iteration code that alters results requires a
new payload version.

When a file has this block, FractalForge resumes from it instead of from the
Fractint-format blocks.
