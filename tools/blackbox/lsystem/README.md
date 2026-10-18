<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Black-box tools for the L-system spec

Produces the evidence in `corpus/xfractint/lsystem/` for
[docs/specs/lsystem.md](../../../docs/specs/lsystem.md). Read
[docs/CLEANROOM.md](../../../docs/CLEANROOM.md) first. Every `.l` file under
`inputs/` was written by us for these tests (`ff.l` is a copy of
`tools/blackbox/inputs/ff.l`); none comes from XFractint. `inputs/uniq.map`
is a copy of `tools/blackbox/coloring/uniq.map` (256 distinct colours, so a
saved colour index can be read back exactly). The file-format inputs
(`fm*.l`, `fn*.l`, `fp*.l`, `lk*.l`, `gl*.l`, `lh.l`) carry no licence
comment on purpose, because a comment line would change what they test.
XFractint can only open `.l` files whose names fit DOS 8.3, hence the short
names. `geo.l`, `dup.l`, `sl.l` and `rich.l` were written by hand; every
other `.l` file is written by one of the generators below.

- `run-case.sh OUTDIR NAME [only=F1,F2,...] ARGS...`: one batch run
  (`xfractint batch=yes ARGS savename=ff.gif`, amd64 reference image
  `fractalforge-blackbox`) in a fresh `mktemp` directory holding `inputs/*`,
  or only the listed input files. Isolation matters: when an entry or a
  file is missing, XFractint searches the other `.l` files of the directory
  (spec §2.1). Moves `ff.gif`, `stopmsg.txt` and the console log to
  `OUTDIR/NAME.*`; a watchdog stops runs after `CASE_TIMEOUT` seconds.
  `IMG=native` uses the native-architecture image of
  `tools/blackbox/par-map/docker`; it was used for exploration only, because
  its pixels differ from the amd64 build where coordinates tie (spec §6.2).
- `run-cases.sh CASES OUTDIR`: all lines `<name> <args...>` of a cases file,
  `JOBS` at a time.
- Generators (each writes its inputs and `cases-<round>.txt`):
  `gen-chars.py` (every printable character as a turtle command),
  `gen-args.py` (numeric arguments, push/pop), `gen-fmt.py`, `gen-fmt2.py`,
  `gen-fmt3.py` (file format, look-up, limits, messages), `gen-lhs.py`
  (which characters may be rule symbols), `gen-seg.py` (single segments at
  angles 3-50 and six image sizes), `gen-rich.py` (multi-segment curves),
  `gen-prec.py` (rounding near pixel ties), `gen-sem.py` (order, colours,
  push/pop, `@`, `|`), `gen-ext.py` (protected arguments, classic curves,
  odd sizes, default palette), `gen-turtle.py` (move and degree-turn
  commands, arrowhead curves), `gen-misc.py` (float, duplicate rules,
  view-window sizes, corners, extra parameters).
- `gen-all.sh OUTDIR`: regenerates everything and runs all rounds (about
  25 minutes with `JOBS=10` under qemu, most of it the very high orders).
- `run-interrupts.sh OUTDIR`: interrupt, save and reload tests (interactive,
  uses `tools/blackbox/resume/run-keys.sh` and `mark.py`); about 7 minutes.
- `bisect-prec.py OUTDIR M...`: finds, by bisection, where XFractint's
  rounding of a segment end flips near column M + 1/2; shows the float32
  scale (spec §6.2). `bisect-origin.py`: the same for the turtle origin of
  a given curve (to ~1e-16 pixel); it found the extended-precision
  direction table.
- `collect.sh OUTDIR`: copies the outputs into the corpus folder, writes
  `results.txt` per round (`summarise.py`), `interrupt/results.txt`
  (`intsum.py`), `cases.txt`, `verify.txt` and `SHA256SUMS`.
- `summarise.py DIR`: one line per case (pixel hash, bounding box of drawn
  pixels, colour counts, type, param 1, corners, status, `fractint003`
  name, blocks, stop message, console output). `ascii.py FILE.gif`: drawn
  pixels as text. `intsum.py`: interrupt/reload summary. `mark.py`: paints
  marker pixels into a saved GIF.
- `lsysmodel.py`: our reference model of the renderer, written from the
  spec (extended-precision arithmetic emulated with Python fractions).
  `verify.py CORPUSDIR`: renders every image case with it and compares the
  pixels with XFractint's.
