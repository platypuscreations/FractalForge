<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# FractalForge native GIF block

Status: **identifier fixed, payload to be specified in Phase 1.**

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

## Payload (Phase 1)

Planned content: a payload version, then a compressed body that holds:

- the full parameter set, including values Fractint can't express
- the exact `RenderState` checkpoint
- the full-precision iteration buffer (the GIF image holds only 8-bit indices)
- the palette

When a file has this block, FractalForge resumes from it instead of from the
Fractint-format blocks.
