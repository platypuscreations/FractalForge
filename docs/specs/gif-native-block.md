<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# FractalForge native GIF block

Status: **payload version 1**. The block is written after the image data, before any Fractint-format blocks (implemented in `crates/formats/src/gif/native_block.rs`).

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

## Payload, version 1

The sub-blocks, concatenated, form the payload. All integers and floats are
little-endian; floats are IEEE 754 binary64.

| Bytes | Field |
|-------|-------|
| 2 | payload version, `1` |
| 1 | compression, `1` = raw DEFLATE (RFC 1951) |
| 8 | uncompressed body length in bytes |
| n | compressed body |

### Body

| Bytes | Field |
|-------|-------|
| 1 | length L of the fractal type name |
| L | type name, UTF-8 (e.g. `mandel`, `julia`) |
| 4×8 | window corners: x min, x max, y min, y max (centres of the edge pixels) |
| 4×8 | type parameters 1–4 |
| 4 | maximum iterations |
| 8 | bailout: escape when \|z\|² exceeds this |
| 1+1 | inside colouring: `0, index` = fixed palette index; `1, 0` = colour of the maximum iteration count |
| 1 | outside colouring: `0` = by iteration count |
| 4 | image width W |
| 4 | image height H |
| 1 | pass mode: `1` one pass, `2` two passes, `3` guessing |
| 4 | coarsest step (power of two: 1, 2, or the guessing block size) |
| 4 | current stage index |
| 4 | next row index within the stage |
| 8 | calculation time so far, microseconds |
| 768 | palette: 256 RGB triples |
| 4×W×H | iteration buffer, row-major: escape count n (the first iterate with \|z\|² > bailout, 1..=max iterations; the start point isn't tested), or `0xFFFFFFFE` = inside, `0xFFFFFFFF` = not computed |

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
