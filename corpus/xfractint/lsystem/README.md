<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Evidence for docs/specs/lsystem.md

Renders by Debian's prebuilt `xfractint 20.4.10-5` (amd64 under qemu, image
`fractalforge-blackbox`) of L-system files we wrote ourselves
(`tools/blackbox/lsystem/inputs/`). `cases.txt` lists every batch case with
its arguments; `tools/blackbox/lsystem/run-case.sh` runs
`xfractint batch=yes <arguments> savename=ff.gif` in a fresh directory that
holds the input files named by `only=` (all inputs when there is none).

One folder per round (`tools/blackbox/lsystem/cases-<round>.txt`):

| Folder | What it tests |
|---|---|
| `chars` | every printable character inside an axiom, with and without a digit after it |
| `args` | numeric arguments of `@`, `C`, `<`, `>`, `/`, `\`; push and pop |
| `fmt`, `fmt2`, `fmt3` | file format: names, braces, comments, keywords, values, rules, limits, file look-up, messages |
| `lhs` | which characters may start a rule line |
| `seg` | single segments at angles 3-50 and the L shape `F+F`, six image sizes |
| `rich` | multi-segment curves at several orders and sizes |
| `prec` | segment ends placed just above and below pixel ties |
| `sem` | order parameter, colour arithmetic, zero-size drawings, push/pop errors, `|`, `!`, `@` |
| `ext` | arguments during rewriting, branching and coloured plants, odd image sizes, default palette and size |
| `mv`, `deg`, `deg2` | the commands that use the degree heading (`D`, `M`, `/`, `\`) |
| `arr` | arrowhead-type curves (60-degree ties) |
| `misc` | `float=`, duplicate rules, `/` with `F`, view-window sizes, corners, extra params, unrelated options |
| `interrupt` | interrupted saves and reloads (`tools/blackbox/lsystem/run-interrupts.sh`); `*.keys.txt` give the keys and arguments, `results.txt` the summary |
| `bisect` | output of `bisect-prec.py` and `bisect-origin.py` |

Per case: `<case>.gif` (the saved image, renamed from `ff.gif`),
`<case>.stopmsg.txt` (XFractint's batch-mode message file, when written)
and `<case>.log` (console output, kept only when it says more than the qemu
platform warning and the closing banner: crashes and runs we stopped).
`results.txt` in each folder is `tools/blackbox/lsystem/summarise.py` run on
that folder; the spec cites its lines by case name. `verify.txt` is
`tools/blackbox/lsystem/verify.py` run on this folder: our model of the
renderer reproduces every image case it covers pixel for pixel.

Notes:

- `sem/s-kobig-16`, `-30`, `-100`, `-1000`, `-40000` were stopped by us
  after 300 s (very high orders); they have a log but no image.
- Several cases crash XFractint (segmentation fault or stack smashing); that
  is the observed behaviour, see spec §2.7 and §5.
- Pixel hashes in `results.txt` are for the amd64 build. The native arm64
  build of the same package saves different pixels where coordinates tie
  (spec §6.2), so it must not be used as a reference.
