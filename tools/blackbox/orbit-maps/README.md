<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Orbit-map experiments

Scripts behind [docs/specs/orbit-maps.md](../../../docs/specs/orbit-maps.md)
and the evidence in [corpus/xfractint/orbit-maps](../../../corpus/xfractint/orbit-maps).
Black-box only. See [docs/CLEANROOM.md](../../../docs/CLEANROOM.md). Python
runs as `python3 -I` and needs only the standard library.

| File | Purpose |
|---|---|
| `render.sh OUT NAME ARGS…` | one batch render (adds `viewwindows=2`, copies `../coloring/uniq.map` into the work dir) |
| `run-cases.sh OUT [CASES] [JOBS]` | renders a cases file (default: the corpus `cases.txt`) |
| `interrupt.sh OUT NAME DELAY ARGS…` | interactive render saved with `s` after DELAY seconds |
| `rload.sh OUT NAME IN.gif WAIT` | loads IN.gif with the in-program `r` command (resumes); `ENTER=1` uses `filename=` + Enter |
| `info.py FILE…` | prints the `fractint001` fields used here and a colour histogram |
| `orbitsim.py` | GIF loading, the pixel mapping (§2.3) and the time colour (§3.3) |
| `xmath.py` | sin/cos/tan rounded to 64-bit mantissa, then to double (§2.4) |
| `libm-orbit.pl`, `libm-orbits.sh`, `typeparams.py` | compute orbits inside the container with its glibc libm |
| `libm-popcorn.pl` | popcorn renderer with glibc libm (comparison only) |
| `verify.py CORPUS [ORBITS]` | renders every corpus file with the spec's algorithms and prints the match |
