<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Fractint-format GIF extension blocks

Status: **not yet written (Phase 1).** The reference corpus exists.

This spec must be derived **only** from bytes of files saved by XFractint. See
[CLEANROOM.md](../CLEANROOM.md).

## Sources

- `corpus/xfractint/complete/`: completed batch renders. Each one changes a
  single parameter from `mandel-base` (see `cases-complete.txt`).
- `corpus/xfractint/interrupted/`: renders saved mid-calculation, which carry
  resume data (see `cases-interrupted.txt`).
- Generator: `tools/blackbox/gen-corpus.sh`. XFractint version: Debian
  `xfractint 20.4.10-5` (amd64).

## Observations so far (black box)

- Saves are GIF89a, 800×600 by default in batch mode.
- Rendering the same case twice gives files that differ in only 2–3 bytes,
  probably a time-related field. The spec should identify it, and tests that
  compare our output with the corpus should ignore it.
- In batch mode, `passes=b` with default coloring writes `stopmsg.txt` saying
  that boundary tracing can't be combined with inside=0 or outside=0. It still
  saves an image.

## Method

1. List the application extensions in each file (identifier, authentication
   code, sub-block lengths).
2. Diff each case against `mandel-base` to find the bytes each parameter controls.
3. Compare interrupted files with each other to find the resume-state fields.
4. Write every field here: offset, width, byte order, meaning and evidence (the
   files that show it).
