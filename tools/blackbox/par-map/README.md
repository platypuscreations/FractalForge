<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Black-box tools for the `.par` / `.map` specs

Produces the evidence in `corpus/xfractint/par-map/` for
`docs/specs/par.md` and `docs/specs/map.md`. Read
[docs/CLEANROOM.md](../../../docs/CLEANROOM.md) first.

- `gen-all.sh OUTDIR`: builds both images and runs everything.
- `run-case.sh`, `run-cases.sh`: amd64 batch cases (`cases-*.txt`, one
  `<name> <xfractint args>` per line), in fresh directories holding `inputs/*`.
- `docker/`: the same Debian prebuilt `xfractint` package for the host's
  native architecture, plus xdotool/xwd, with `drive.sh` (a tiny step
  language: `sleep`, `key`, `type`, `shot`, `waitfile`). Used for the
  interactive screens: `run-native.sh`, `run-writer-cases.sh` +
  `docker/makepar-steps.sh` (the "save current parameters" screen, `b` key),
  `steps/*.txt` + `cases-special.txt`.
- `maps-dir-check.sh`: places one of our maps in the program's maps
  directory to show where `map=` inside an entry is looked up.
- `inputs/`: our own test files. The `.map` files have no licence header on
  purpose: every line of a map file is a palette entry (map.md §2.3), so a
  comment line would change the test. `ff.frm` and `ff.map` are copies of
  `tools/blackbox/inputs/`.

`makepar=` segfaults in this XFractint build (both amd64 and arm64), so
parameter files are written through the interactive screen instead.
Screenshots (`shot`) are for driving the UI only and are not kept as
evidence; note that some screens list the files bundled with XFractint, which
we must not use.
