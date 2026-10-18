<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# XFractint evidence: continuous and 3D orbit types

Evidence for [docs/specs/attractors-3d.md](../../../docs/specs/attractors-3d.md):
lorenz (63), lorenz3d (84), rossler3d (86), pickover (89), lorenz3d1/3/4
(124–126), kamtorus3d (42) and icons3d (160).

- `cases.txt` gives the XFractint command line of every file. Batch files were
  made with `tools/blackbox/attractors-3d/gen.sh`, the `-int` / `-rload` files
  with `tools/blackbox/attractors-3d/intr.sh`.
- `tools/blackbox/attractors-3d/sim.py FILE.gif …` re-renders files from their
  stored parameters and reports the match (pickover needs Docker).
- XFractint: Debian `xfractint 20.4.10-5` (amd64) in the `fractalforge-blackbox`
  container. Repeated renders differ only in the seed and timer bytes of
  `fractint001` (gif-fractint-blocks.md §3).
- `SHA256SUMS`: `shasum -a 256` of the files, run in this folder.
