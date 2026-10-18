<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Deep-zoom (arbitrary precision) experiments

Scripts behind [docs/specs/deep-zoom.md](../../../docs/specs/deep-zoom.md) and
the evidence in [corpus/xfractint/deep-zoom](../../../corpus/xfractint/deep-zoom).
Black-box only: they drive the `fractalforge-blackbox` image (and, for the PAR
writer, `ff-parmap-native` from `../par-map/docker`) and read the files
XFractint saves. See [docs/CLEANROOM.md](../../../docs/CLEANROOM.md).
Python runs as `python3 -I` and needs only the standard library.

| File | Purpose |
|---|---|
| `render.sh OUT NAME ARGS…` | one batch render in a fresh `mktemp` directory; adds `viewwindows=2 map=uniq.map savename=ff.gif` (`FFNOVW=1` drops `viewwindows=2`, `FFINPUTS` copies input files in) |
| `run-cases.sh OUT CASES [P]` | every `<name> <args>` line of a cases file through `render.sh`, P at a time |
| `keys.sh OUT NAME IN\|- "ACTIONS" ARGS…` | interactive run driven by `run-keys.sh` (`sN` sleep, `kKEY` key, `tTEXT` type); used for interrupts, `r` loads and command-line loads |
| `load-all.sh IN OUT [WAIT] [P]` | loads every crafted GIF with `filename=`, presses Enter, saves after WAIT s |
| `craft-all.sh BASE OUT` | makes the crafted files (`k00`…`k23`) from the `cases-e7.txt` base renders |
| `craft.py IN OUT ops…` | edits a save: write slots, change N, drop/add `fractint005`, overwrite f64 fields, repaint a row (see its header) |
| `dz.py` | slot decoder/encoder (`slot_decode`, `slot_encode`); `python3 -I dz.py f.gif` dumps the deep fields |
| `tab.py`, `cmp.py` | one-line summaries; compare a loaded save with reference renders |
| `encchk.py`, `normchk.py`, `f64chk.py` | slot vs exact decimal input; normal-form check; f64 corners vs slots |
| `meas.py`, `nrule.py` | f64 corner error per pixel (switch study); check of the fitted N rule |
| `sim.py` | exact / XFractint-like fixed-point escape-time simulator for mandel and julia saves |
| `cases-e1.txt` … `cases-e12.txt`, `cases-intr.txt`, `cases-par*.txt` | the cases that were run (E-numbers as in the spec) |
| `ff.frm`, `dz.par`, `uniq.map` | our own input files (copies of `../inputs/ff.frm`, `../coloring/uniq.map`; `dz.par` written for the token-length test) |

The cases files are run with, for example,
`./run-cases.sh "$(mktemp -d)" cases-e2.txt 6`; `cases-e3.txt` and
`cases-e7.txt` need `FFINPUTS=$PWD/ff.frm`, `cases-e6b.txt` needs `FFNOVW=1`,
`cases-e11.txt` needs `FFINPUTS=$PWD/dz.par`. `cases-par*.txt` go through
`../par-map/run-writer-cases.sh CASES OUT 6`.
