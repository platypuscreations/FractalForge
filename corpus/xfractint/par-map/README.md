<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# `.par` / `.map` black-box evidence

Evidence for [docs/specs/par.md](../../../docs/specs/par.md) and
[docs/specs/map.md](../../../docs/specs/map.md). Everything here was produced
by XFractint (Debian `xfractint 20.4.10-5`) from input files we wrote
ourselves (`tools/blackbox/par-map/inputs/`). No bundled XFractint `.par` or
`.map` file was used. See [CLEANROOM.md](../../../docs/CLEANROOM.md).

Regenerate with `tools/blackbox/par-map/gen-all.sh OUTDIR` (about 10 minutes
on Apple Silicon) plus `tools/blackbox/par-map/maps-dir-check.sh
OUTDIR/special`, then copy the files listed below.

| Path | What |
|---|---|
| `cases.txt` | command line of every case (all `cases-*.txt` files) |
| `results-read.txt` | outcome of **every** amd64 batch case: image or not, XFractint's message, and the difference from the reference `s-ref` (`fractint001` offsets that differ, palette, pixel count). "SAME" = identical apart from seed and time. |
| `results-roundtrip.txt` | each file XFractint wrote, loaded back in batch mode, compared with the GIF saved in the writing session |
| `read/<case>.gif` | GIFs kept for the palette, `center-mag` and map cases (the rest are summarised in `results-read.txt` only) |
| `written/<name>.out.par` | all 42 parameter files XFractint wrote (interactive `b` screen, `cases-writer.txt`) |
| `written/<name>.gif` | GIF saved just before writing, for a few of them |
| `written/xf-mapedit.map` | a map file written by XFractint's palette editor (`sp-mapedit`) |
| `rt/rt-<name>.gif` | batch load of `written/<name>.out.par` |
| `special/sp-multi.out.par` | two entries with four comment lines, one replaced (`cases-special.txt`) |
| `special/sp-into.r1.par` | `inputs/r1.par` after XFractint added one entry and replaced another |
| `special/sp-unknown.console.txt` | console message for an unknown key at interactive start-up |
| `special/maps-dir-check.gif` | `map=` inside an entry finds a file placed in the program's maps directory |

Notes:

- `read/` and `rt/` come from the amd64 image under qemu. `written/` and
  `special/sp-*` come from the **arm64** build of the same Debian package
  (`tools/blackbox/par-map/docker`), run natively because `makepar=`
  segfaults and the interactive screens are slow under emulation. The two
  builds differ only in `fractint001` offsets 122–123 for the same command
  line, which is why `results-roundtrip.txt` lists those offsets.
- Pixel differences in palette cases come from XFractint reading the image
  back from the display: entries with the same colour collapse to the lowest
  index, and 728 further pixels change (0 → 16, 15 → 31) whenever any map is
  loaded (map.md §2.6). They are not caused by the file being read.
- Screenshots taken while driving the program are not kept.
