<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# XFractint 3D transform evidence

Black-box evidence for [docs/specs/3d-transform.md](../../../docs/specs/3d-transform.md).

- `cases.txt`: one line per case, `NAME INPUT ARGS...`. `out/NAME.gif` is what
  `xfractint batch=yes ARGS savename=ff.gif` saved, run in a fresh directory
  with `in/INPUT` copied to `in.gif` (Debian xfractint 20.4.10-5 in the
  `fractalforge-blackbox` container).
- `in/`: input GIFs. All except `mandel.gif` (an XFractint render, see
  `cases.txt`) are synthetic images written by
  `tools/blackbox/3d-transform/mkin.py`; `tools/blackbox/3d-transform/gen-inputs.sh`
  regenerates them byte-for-byte. Unless a case says otherwise, their palette
  has 256 distinct colours and no black entry.
- `tools/blackbox/3d-transform/runcases.sh` reruns the cases;
  `tools/blackbox/3d-transform/sim3d.py cases.txt in out` compares our
  simulator with every output.
- `SHA256SUMS`: `shasum -a 256` of every file, run from this folder.
