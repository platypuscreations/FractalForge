<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Orbit-map evidence

XFractint (Debian `xfractint 20.4.10-5`, amd64, in `fractalforge-blackbox`)
renders of the 2D orbit-plotting types, used by
[docs/specs/orbit-maps.md](../../../docs/specs/orbit-maps.md). Black-box
only; see [docs/CLEANROOM.md](../../../docs/CLEANROOM.md).

- `*.gif` here: 400×300 batch renders. `cases.txt` gives every command line;
  `tools/blackbox/orbit-maps/run-cases.sh OUTDIR` regenerates them (it adds
  `viewwindows=2 savename=ff.gif`; `map=uniq.map` is the distinct-colour
  palette from `tools/blackbox/coloring/`).
- `resume/`: interrupted saves, resumed loads and their batch references;
  `resume/cases.txt` gives the commands.
- `res800/`: three 800×600 renders (resolution check); `res800/cases.txt`.
- `SHA256SUMS`: `shasum -a 256` of every file, run from this folder.

Reproduce the match table of the spec with

    tools/blackbox/orbit-maps/libm-orbits.sh corpus/xfractint/orbit-maps /tmp/orbits
    python3 -I tools/blackbox/orbit-maps/verify.py corpus/xfractint/orbit-maps /tmp/orbits
