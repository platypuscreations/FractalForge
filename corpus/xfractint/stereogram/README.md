<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Random-dot stereogram evidence

Black-box evidence for [docs/specs/stereogram.md](../../../docs/specs/stereogram.md).
Produced with the `fractalforge-blackbox` container (Debian's prebuilt
XFractint 20.04.10) and the helpers in `tools/blackbox/stereogram/`.

- `depth/` — depth maps fed to XFractint. `rdsrows.gif` is XFractint's own
  render of the `rdsrows` formula in `tools/blackbox/stereogram/inputs/rds.frm`
  (row `y` has colour `y` for `y` < 256). The others were written by
  `tools/blackbox/stereogram/craft-depth.py` (pixels and palette replaced in a
  copy of `rdsrows.gif`, which keeps its `fractint001` block), except
  `des0`–`des6` (random rectangles picked by a throwaway search script to
  separate candidate calibration-bar windows), `big.gif` (XFractint's 800×600
  render of `rdsrows`) and `pat.gif` (50×40 image-map pattern, pixel
  `(5x + 7y) mod 256`).
- `out/` — the stereograms XFractint saved, one per line of `cases.txt`.
- `cases.txt` — for each output: depth map, width in inches, depth effect, bar
  mode, the glibc `srand` seed recovered from the pixels, the XFractint
  arguments and the key sequence.

Check everything with `python3 -I tools/blackbox/stereogram/verify.py`.
All cases match the model exactly except `l-gray*` (grayscale rounding not
pinned down) and two rows of `l-d500` (negative separation, not modelled).
