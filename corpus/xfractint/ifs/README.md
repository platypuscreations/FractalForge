<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# IFS reference renders

Evidence for [docs/specs/ifs.md](../../../docs/specs/ifs.md). XFractint
(Debian `xfractint 20.4.10-5`, amd64) saved every GIF here as a black box in
`tools/blackbox`. Every `.ifs` file the cases read is our own, from
`tools/blackbox/ifs/inputs/` (`ffcopy.ifs` is an exact copy of
`tools/blackbox/inputs/ff.ifs`). None of XFractint's bundled IFS files was
opened or used.

- `cases.txt`: the mode and command line of every file, grouped by spec
  section. `tools/blackbox/ifs/render.sh` ran them (it adds `savename=ff.gif`
  and works in an empty directory that holds the input files).
- `NAME.stopmsg.txt`: the message XFractint wrote for that case. Some cases
  (`sel-ifsname`, `name-19`, `const-mi1`) saved no image at all.
- Most files are 400×300 (`viewwindows=2`). Cases with the recorder corners
  `0/1/0.0009765625/1.1708984375` are 800×600 so that one row holds one point.
- `in20.gif`, the input of the `load-*` and `second-run` cases, is a copy of
  `int-20.gif`. `zz.ifs` in `sel-missingfile` is a copy of `ffcopy.ifs`.
- `int-*`, `load-*` and `second-run` depend on timing under emulation, so
  they can't be reproduced exactly. Never regenerate them, only add new ones.
  Batch cases come out with the same pixels every time. Only the seed and the
  calculation time in `fractint001` change between runs.
- `tools/blackbox/ifs/ifsmodel.py IMAGE.gif FILE.ifs ENTRY` predicts the
  pixels of a 2D or 3D case from the spec's model and counts differences.
- `SHA256SUMS`: run `shasum -a 256 -c SHA256SUMS` in this folder.
