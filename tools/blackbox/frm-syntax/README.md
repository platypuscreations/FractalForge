<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Black-box tools for the formula-file syntax spec

Produces the evidence in `corpus/xfractint/frm-syntax/` for
[docs/specs/frm-syntax.md](../../../docs/specs/frm-syntax.md). Read
[docs/CLEANROOM.md](../../../docs/CLEANROOM.md) first. Every `.frm` and `.par`
file under `inputs/` was written by us for these tests; none comes from
XFractint. `inputs/uniq.map` is a copy of `tools/blackbox/coloring/uniq.map`
(256 distinct colours, so saved colour indices can be read back exactly).
The file-structure inputs (`s*.frm`, `t*.frm`, `r6.frm`, `p*.par`) carry no
licence comment on purpose, because a comment line would change what they
test.

- `run-case.sh OUTDIR NAME ARGS...`: one batch run (amd64 reference image
  `fractalforge-blackbox`) in a fresh `mktemp` directory holding `inputs/*`,
  with `type=formula passes=1 periodicity=0 viewwindows=2 map=uniq.map`
  prepended and `savename=ff.gif` appended. Moves `ff.gif`, `stopmsg.txt`
  and the console log to `OUTDIR/NAME.*`. A watchdog removes the container
  after `CASE_TIMEOUT` seconds (default 120). `IMG=native` uses the
  native-architecture image of `tools/blackbox/par-map/docker` instead.
- `run-cases.sh CASES OUTDIR`: all lines `<name> <args...>` of a cases file,
  `JOBS` at a time.
- `gen-probe.py LIST PREFIX`: turns `probe-*.txt` (one expression per line)
  into `inputs/PREFIX.frm` and `cases-PREFIX.txt`. Each entry sets `k` in the
  initialisation section and counts `z` up to `k`, so the saved colour index
  is floor(k) + 1 (for 0 ≤ k < 254; calibrated by `cases-cal.txt`).
- `gen-struct.py`, `gen-struct2.py`, `gen-sym.py`, `gen-r3.py`: write the
  file-structure, symmetry and round-3 inputs and case files. `cal.frm`,
  `a.frm`, `r4.frm`, `r5.frm`, `r6.frm`, `p2.par` and their case files were
  written by hand (r4 and r6 with short inline scripts whose output is the
  file itself).
- `gen-all.sh OUTDIR`: regenerates everything and runs all rounds (about
  15 minutes with `JOBS=10` under qemu). `collect.sh OUTDIR` copies the
  results into the corpus folder and writes `results.txt`, `cases.txt` and
  `SHA256SUMS`.
- `summarise.py DIR`: one line per case (pixel hash, most common colour
  indices, type, symmetry setting, `fractint003` name and flags, stop
  message, console output). `symcheck.py DIR REF`: mirror counts for the
  symmetry cases. Both use our GIF reader `tools/blackbox/resume/gif.py`.
