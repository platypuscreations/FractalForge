<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# XFractint reference corpus

These GIFs were saved by XFractint (Debian `xfractint 20.4.10-5`, amd64) running
as a black box in `tools/blackbox`. They are test fixtures for working out the
Fractint-format GIF blocks and for checking interop. They are not shipped in
releases.

- `complete/`: batch renders, one per line of `cases-complete.txt`. Each
  changes one parameter from `mandel-base`. `*.stopmsg.txt` is the message
  XFractint wrote during that render.
- `interrupted/`: interactive renders saved partway by pressing **s** after a
  delay, one per line of `cases-interrupted.txt`. They hold resume data.
  Timing under emulation isn't deterministic, so these files can't be
  reproduced exactly. Never regenerate them, only add new ones.
  `int-passb` (boundary tracing) didn't save and is missing.
- `SHA256SUMS`: checksums of the committed files.

Regenerate with `tools/blackbox/gen-corpus.sh`, which writes to
`tools/blackbox/out/`. Copy new files here deliberately and update `SHA256SUMS`.
