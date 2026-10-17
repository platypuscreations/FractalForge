<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Resume-data experiments (`fractint002`)

Scripts behind [docs/specs/fractint-resume.md](../../../docs/specs/fractint-resume.md)
and the evidence in [corpus/xfractint/resume](../../../corpus/xfractint/resume).
Black-box only: they drive the `fractalforge-blackbox` image and read the
files it saves. See [docs/CLEANROOM.md](../../../docs/CLEANROOM.md).

The shell scripts write into `out/` and `work.*/` **next to themselves**, so
copy this folder to a scratch directory before running anything. Python runs
as `python3 -I` and needs only the standard library.

| File | Purpose |
|---|---|
| `views.sh` | the parameter sets (`MS`, `MN`, `J`, …) used everywhere |
| `queue.sh JOBS P` | runs a job list P at a time. Job kinds: `B` batch render, `I` interrupted render (`run-interrupt`), `L` command-line load + Enter (`run-load`; this **recalculates**), `R` load with XFractint's `r` command (this **resumes**) |
| `batch.sh`, `intr.sh`, `load.sh` | wrappers around the image's `run-batch`, `run-interrupt`, `run-load` |
| `run-keys.sh`, `keys.sh` | arbitrary key sequences inside the container (`sN` sleep, `kKEY` key, `tTEXT` type), used for the `r` load |
| `jobs-*.txt` | the job lists that were run (`jobs-ref`, `jobs-refg`: references; `jobs-int1`, `jobs-extra`, `jobs-f28`: interrupted saves and more; `jobs-r`: `r` loads; `jobs-res`, `jobs-craft`, `jobs-x`: early command-line loads, outputs not kept) |
| `gif.py` | GIF reader/writer (LZW decode, literal or real LZW encode); `python3 -I gif.py f.gif` dumps the blocks |
| `model.py INT REF` | the reader rule of spec §4: predicts every pixel of an interrupted save from the finished render; `FORCE_SYM=xy` for barnsleym1 |
| `craft.py REF OUT key=value…` | makes a resumable file from a finished render (item fields, pixel blanking/blocks, markers) |
| `craft-r.sh` | the crafted files of `corpus/xfractint/resume/crafted` (expects the corpus `ref/` and `int/` files in `out/`) |
| `craft-all.sh`, `craft-extra.sh`, `craft-x.sh` | early crafted files for command-line loads |
| `marker.py OUT IN REF`, `evalq.sh` | per-row report: rows equal to the reference, left as in the input (markers survived) or mixed |
| `final.py OUT REF` | is OUT finished and identical to REF |
