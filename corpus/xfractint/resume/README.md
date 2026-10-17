<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# XFractint resume-data evidence

Evidence for [docs/specs/fractint-resume.md](../../../docs/specs/fractint-resume.md).
All `.gif` files under `ref/`, `int/`, `resumed/` and `loaded/` were saved by
XFractint (Debian `xfractint 20.4.10-5`, `fractalforge-blackbox`); files under
`crafted/` were written by our scripts (`tools/blackbox/resume`). `cases.txt`
gives the command line or recipe for every file; `model-check.txt` is the
output of the reader-rule check (spec §4).

| Folder | Content |
|---|---|
| `ref/` | finished batch renders, `ref-<view>-<passes>` |
| `int/` | interrupted interactive renders, `i-<view>-<passes>-<seconds>`, plus `s3-`/`s4-` (smaller images) |
| `resumed/` | `int/` saves loaded with XFractint's `r` command and left to finish (`o-…`) |
| `crafted/` | our files: `C*`, `Y*` from `ref/` files, `K*` = XFractint saves with marker pixels |
| `loaded/` | XFractint's saves after loading `crafted/` files (`q-…` with `r`; `k1-*`, `k2-*` key experiments) |

View tags are listed in spec §10. Timing of interrupted saves is not
reproducible (qemu emulation, three runs in parallel); never regenerate a
committed file, add new ones instead. Checksums: `SHA256SUMS`
(`shasum -a 256` from this folder).
