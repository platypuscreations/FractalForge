<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Deep-zoom evidence

Evidence for [docs/specs/deep-zoom.md](../../../docs/specs/deep-zoom.md). Every
GIF was saved by XFractint (Debian `xfractint 20.4.10-5`) in the black-box
containers, except `crafted/`, which our own scripts edited. Command lines are
in [cases.txt](cases.txt); one-line block summaries of every GIF are in
[results.txt](results.txt). The scripts are in
[tools/blackbox/deep-zoom](../../../tools/blackbox/deep-zoom). No Fractint,
XFractint or Iterated Dynamics source code or documentation was used.

Batch renders are 400×300 (`viewwindows=2`) with the distinct palette
`uniq.map` unless noted, so pixel values are colour indices.

| Folder | Contents | Spec |
|---|---|---|
| `switch/` | when XFractint starts using arbitrary precision: magnification scans (`th-*`, `c0-*`, `c300-*`, `csm-*`, `c0f-*`, `c0x*-*`), `mathtolerance=` variants (`tol*`), 800×600 renders (`full-*`) | §1 |
| `nsize/` | the size N as a function of magnification, centre, image size and `xmag` (`n-*`, `nd-*`, `nb-*`, `nf-*`, `nc*-*`, `nx*-*`, `big-*` = 190×143) | §2 |
| `types/` | which fractal types switch (`ty-*` at mag 1e15, `ty30-*` at 1e30, `tyv-*` with a structured view) | §1.1 |
| `encoding/` | slot encoding from `corners=`/`center-mag=` decimal strings (`enc-*`), digits used (`dig*`), token length (`len-*`, `pl-*` read from a `.par`) | §3, §6 |
| `pixels/` | deep views for pixel comparison and `sim-results.txt` (our simulator against them) | §8 |
| `base/` | the renders the crafted files start from | §5 |
| `crafted/`, `loaded/` | hand-edited copies of saves, and what XFractint saved after loading each one with `filename=` and pressing Enter (`loaded/<k>` belongs to `crafted/<k>`) | §5 |
| `interrupt/` | interrupted deep saves (`int-*`), the same with a marker row (`*-mark`), the batch references (`ref-*`) and the saves after resuming with the `r` command (`r-*`) | §9 |
| `par/` | parameter files XFractint wrote for deep views (`wd*.out.par`, native arm64 build, "b" screen) with the GIF saved just before (`wd*.gif`), and the files loaded back (`rt-*`) | §7 |

Total size about 2 MB. `SHA256SUMS` covers every file (run `shasum -a 256 -c
SHA256SUMS` in this folder).
