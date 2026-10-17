<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Fractint-format GIF extension blocks

Status: **Phase 1 draft, derived from black-box analysis only.** Every field
below is justified by bytes of files that XFractint saved, or by how XFractint
behaved when it loaded files we made. No Fractint, XFractint or Iterated
Dynamics source code, headers or documentation was consulted. See
[CLEANROOM.md](../CLEANROOM.md). Field names are our own.

Conventions:

- Offsets are decimal and count from the first byte of a block's **payload**,
  after the sub-blocks are joined (see §1.3). Hex values are written `0x..`.
  Byte strings are written as hex pairs, `96 00`.
- `i16`, `u16`, `i32` and `u32` are little-endian integers. `f64` and `f32` are
  little-endian IEEE-754 doubles and singles.
- **Confirmed** means a one-parameter diff, or a load test, shows the field
  directly. **Hypothesis** means it fits the evidence but isn't proven.
- Evidence names refer to files under `corpus/xfractint/` (`complete/`,
  `interrupted/`, `verify/`). Their command lines are in `cases-complete.txt`
  and `cases-interrupted.txt`. Unless a case says otherwise, it is
  `mandel-base` (`type=mandel maxiter=150`) plus the one parameter it names.

## Sources

- `corpus/xfractint/complete/`: completed batch renders. Each one changes one
  parameter from `mandel-base` (see `cases-complete.txt`). The Phase 1 additions
  start at the comment "Phase 1 spec evidence".
- `corpus/xfractint/interrupted/`: renders saved mid-calculation, which carry
  resume data (see `cases-interrupted.txt`).
- `corpus/xfractint/verify/`: files our scripts wrote or patched, and what
  XFractint saved after loading them (see §8).
- `tools/blackbox/inputs/`: formula, L-system, IFS and palette files we wrote
  ourselves, for the cases that need them.
- Generator: `tools/blackbox/gen-corpus.sh`. XFractint version: Debian
  `xfractint 20.4.10-5` (amd64).

## Observations (black box)

- Saves are GIF89a. Batch mode renders 800×600 by default. `viewwindows=2`
  gives a 400×300 image (`mandel-viewwindows-2`). `-geometry` had no effect.
- Two renders of the same case differ in **3 bytes at most**: the 16-bit random
  seed (payload offsets 98–99) and the calculation time (214–217) in the
  `fractint001` block. Pixels and everything else are identical. See §3.
- In batch mode, `passes=b` with default coloring writes `stopmsg.txt` saying
  that boundary tracing can't be combined with inside=0 or outside=0. It still
  saves an image.
- `float=yes` and `float=no` produce byte-identical blocks and pixels
  (`mandel-float`, `mandel-float-no`). This build doesn't record a float flag
  that we could find. See §5.6.
- A batch run that loads a GIF (`filename=in.gif`) re-saves the loaded image
  as-is. It does not recalculate, and parameters given after the file name
  are ignored. Interactive mode resumes an interrupted image when **Enter** is
  pressed after loading. That is how §8 checks the writer's guide.
- `makepar=` segfaults under qemu, so it couldn't be used.

---

## 1. GIF-level structure

### 1.1 Block order

Every XFractint save we have has this layout:

| # | Item | Bytes | Notes |
|---|------|-------|-------|
| 1 | Header | 6 | `GIF89a` |
| 2 | Logical screen descriptor | 7 | width, height (u16 each), flags `0xD7`, background `0x00`, aspect byte `0x31` (49) |
| 3 | Global color table | 768 | 256 RGB entries (flag `0xD7` = table present, colour resolution 6 bits, not sorted, size 2^(7+1)) |
| 4 | Image descriptor | 10 | `2C`, left 0, top 0, width, height, flags `0x00` (no local table, **not interlaced**) |
| 5 | Image data | | LZW minimum code size `08`, then data sub-blocks, then `00` |
| 6 | Application extensions | | **after** the image data (§1.2) |
| 7 | Trailer | 1 | `3B` |

- There are no graphic control, comment or plain-text extensions, and nothing
  comes before the image.
- XFractint writes the image's LZW data in sub-blocks of 254 bytes (the last
  one is shorter). Readers must accept any length from 1 to 255.
- Logical screen and image sizes are always equal (`mandel-viewwindows-2`:
  both are 400×300).
- Header bytes of `mandel-base.gif`:
  `47 49 46 38 39 61 20 03 58 02 d7 00 31`, then the table at offset 13. The
  image descriptor at offset 781 is `2c 00 00 00 00 20 03 58 02 00 08`.

### 1.2 Application extensions

Each one is framed as a standard GIF89a application extension:
`21 FF 0B`, an 8-byte identifier and a 3-byte authentication code (11 bytes in
all), then data sub-blocks, then `00`. XFractint packs the payload into
sub-blocks of 255 bytes, with the last one shorter.

| Identifier + auth (11 ASCII bytes) | Payload bytes | Sub-blocks | When present | Section |
|---|---|---|---|---|
| `fractint001` | 504 | 255, 249 | always | §2 |
| `fractint002` | 392 | 255, 137 | image saved before the calculation finished | §4 |
| `fractint003` | 66 | 66 | type uses a named entry from an external file (formula, L-system, IFS) | §2.3 |
| `fractint005` | 22 × (N+6) | as needed | arbitrary-precision coordinates in use (deep zoom) | §2.4 |
| `fractint007` | 200 | 200 | `passes=o` (orbits drawing mode) | §2.5 |

The 11 bytes are read as an 8-byte identifier `fractint` plus a 3-digit
authentication code `001`, `002`, and so on. That is the standard split. The
digits pick the block kind.

Order (confirmed): all the other blocks come first, and `fractint001` is
always the last block before the trailer. Seen: `002, 001`; `003, 001`;
`005, 001`; `007, 001`. We never saw two optional blocks in one file, so their
order relative to each other is unknown.

### 1.3 Joining sub-blocks

The payload is all the sub-block data joined, without the length bytes.
Offsets in this spec are payload offsets. For `fractint001` that means
offsets 0–254 are in the first sub-block and 255–503 are in the second.

### 1.4 Palette

The global color table holds 6-bit-per-channel values expanded to 8 bits by
bit replication: `out = (v6 << 2) | (v6 >> 4)`. Loading our own map file
(`map=ff.map`, entries `i, 7i mod 256, 255-i`) produced exactly
`((x >> 2) << 2) | (x >> 6)` for every 8-bit input `x`. A reader can recover
the 6-bit value as `x >> 2`. The default palette starts
`00 00 00, 00 00 aa, 00 aa 00, 00 aa aa, aa 00 00, …` (`mandel-base`). Palette
changes affect only the color table, never the blocks.

---

## 2. Parameter blocks

### 2.1 `fractint001`: main parameter block (504 bytes)

The full hex dump from `mandel-base.gif` is in Appendix A. "Base" below is
the value in `mandel-base.gif`.

| Off | Width | Type | Meaning (our name) | Base | Observed values / notes | Evidence |
|---|---|---|---|---|---|---|
| 0 | 8 | ASCII | signature | `Fractal\0` | constant in every file | all |
| 8 | 2 | i16 | max iterations, clamped | 150 | `min(maxiter, 32767)`: 1000, 255, 2, 32767 for maxiter 32767 to 100000. **Rewritten from offset 408 on load** | `mandel-maxiter-*`, `verify/edit-maxiter-*` |
| 10 | 2 | i16 | fractal type number | 4 | see §6 | type cases |
| 12 | 8 | f64 | xmin (left edge) | -2.5 | | `mandel-corners`, `mandel-corners-distinct` |
| 20 | 8 | f64 | xmax (right edge) | 1.5 | | same |
| 28 | 8 | f64 | ymin (bottom edge) | -1.5 | | same |
| 36 | 8 | f64 | ymax (top edge) | 1.5 | | same |
| 44 | 8 | f64 | param 1 (real part of c for julia) | 0 | julia default 0.3, `params=-0.8/0.156` → -0.8 | `julia-*`, `mandel-params-*` |
| 52 | 8 | f64 | param 2 | 0 | julia 0.6 / 0.156 | same |
| 60 | 8 | — | **unknown** | `00`×8 | always 0 | |
| 68 | 2 | i16 | **unknown** | 19 (`13 00`) | constant. XFractint fills in 19 when the field version (76) is 0 | `verify/zero-template-*` |
| 70 | 2 | i16 | screen width (hypothesis) | 800 | stays 800 even for a 400×300 image | `mandel-viewwindows-2` |
| 72 | 2 | i16 | screen height (hypothesis) | 600 | stays 600 for 400×300 | same |
| 74 | 1 | u8 | **unknown** | 0 | | |
| 75 | 1 | u8 | **unknown** | 1 | refilled with 1 when the version is 0 | `verify/zero-template-v0-resaved` |
| 76 | 2 | i16 | **block field-version** | 17 | 17 in every save. **If it's 0, XFractint ignores inside, outside, passes, bailout and other later fields** (§5.3) | `verify/zero-template-*` |
| 78 | 8 | — | **unknown** | 0 | always 0 | |
| 86 | 4 | f32 | potential: max colour | 0 | `potential=255/820/200` → 255.0; `100/512/50` → 100.0 | `mandel-potential*` |
| 90 | 4 | f32 | potential: slope (encoding unclear) | 0 | 820 → 3.203125 (=820/256), 512 → 2.0 (=512/256), but 100 → 100.0 | `mandel-potential`, `-2`, `-3` |
| 94 | 4 | f32 | potential: bailout | 0 | 200 → 200.0, 50 → 50.0 | same |
| 98 | 2 | u16 | random seed | varies | `rseed=4660` → `34 12`, `rseed=0` → 0, `rseed=-2` → 65534. If no rseed is given, it's the **low 16 bits of the Unix time** at calculation start | `mandel-rseed-*`; mtimes in §3 |
| 100 | 1 | u8 | seed given explicitly | 0 | 1 when `rseed=` was given | `mandel-rseed-*` |
| 101 | 1 | — | unknown | 0 | | |
| 102 | 2 | i16 | biomorph colour | -1 | -1 = off. `biomorph=0` → 0, `=5` → 5 | `mandel-biomorph*` |
| 104 | 2 | i16 | inside colouring | 0 | n ≥ 0 = solid colour n (0, 1, 5, 255). -1 = maxiter, -59 = zmag, -60 = bof60, -61 = bof61, -100 = epscross, -101 = startrail, -102 = period, -103 = fmod, -104 = atan. `inside=-5` is stored as -5 | `mandel-inside-*` |
| 106 | 2 | i16 | log palette mapping, low 16 bits | 0 | yes → 1, 7 → 7, -3 → -3, 100000 → `a0 86` (truncated) | `mandel-logmap-*` |
| 108 | 4 | f32 | invert value 1 (encoding unclear) | 0 | `invert=1/…` → 1/256, `invert=3/…` → 3.0 | `mandel-invert`, `-2` |
| 112 | 4 | f32 | invert value 2 (encoding unclear) | 0 | 0.5 → 0.5/256, 0.125 → 0.125/256 | same |
| 116 | 4 | f32 | invert value 3 (encoding unclear) | 0 | 0.25 → 0.25/256, -0.5 → -0.5/256 | same |
| 120 | 2 | u16 | decomposition colours | 0 | `decomp=2` → 2, 256 → 256 | `mandel-decomp-*` |
| 122 | 2 | i16 | **unknown** | 400 | refilled with 400 when the version is 0 | |
| 124 | 2 | i16 | symmetry setting | 999 | 999 = automatic (default), `symmetry=none` → 0 | `mandel-symmetry-none` |
| 126 | 2 | i16 | 3D: sphere | 0 | `sphere=yes` → 1 | `mandel-3d-sphere` |
| 128 | 2 | i16 | 3D: rotation x | 60 | `rotation=11/22/33` → 11 | `mandel-3d-rotation` |
| 130 | 2 | i16 | 3D: rotation y | 30 | → 22 | same |
| 132 | 2 | i16 | 3D: rotation z | 0 | → 33 | same |
| 134 | 2 | i16 | 3D: scale x | 90 | `scalexyz=41/42/43` → 41 | `mandel-3d-scalexyz` |
| 136 | 2 | i16 | 3D: scale y | 90 | → 42 | same |
| 138 | 2 | i16 | 3D: roughness (= scale z) | 30 | `scalexyz` 3rd value → 43; `roughness=44` → 44 | `-scalexyz`, `-roughness` |
| 140 | 2 | i16 | 3D: waterline | 0 | 5 | `mandel-3d-waterline` |
| 142 | 2 | i16 | 3D: fill type | 0 | 2 | `mandel-3d-filltype` |
| 144 | 2 | i16 | 3D: perspective distance | 0 | 150 | `mandel-3d-perspective` |
| 146 | 2 | i16 | 3D: x shift | 0 | `xyshift=3/4` → 3 | `mandel-3d-xyshift` |
| 148 | 2 | i16 | 3D: y shift | 0 | → 4 | same |
| 150 | 2 | i16 | 3D: light x | 1 | `lightsource=1/2/3` → 1 | `mandel-3d-lightsource` |
| 152 | 2 | i16 | 3D: light y | -1 | → 2 | same |
| 154 | 2 | i16 | 3D: light z | 1 | → 3 | same |
| 156 | 2 | i16 | 3D: smoothing | 0 | 2 | `mandel-3d-smoothing` |
| 158 | 2 | i16 | **unknown** (3D section) | 20 | | |
| 160 | 4 | — | **unknown** | 0 | | |
| 164 | 2 | i16 | stereo: crop 1 | 4 | `crop=7/8/9/10` → 7 | `mandel-stereo` |
| 166 | 2 | i16 | stereo: crop 2 | 0 | → 8 | same |
| 168 | 2 | i16 | stereo: crop 3 | 0 | → 9 | same |
| 170 | 2 | i16 | stereo: crop 4 | 4 | → 10 | same |
| 172 | 2 | i16 | stereo: brightness 1 | 80 | `bright=11/12` → 11 | same |
| 174 | 2 | i16 | stereo: brightness 2 | 100 | → 12 | same |
| 176 | 2 | i16 | stereo: convergence | 0 | `converge=6` → 6 | same |
| 178 | 2 | i16 | stereo: eye separation | 0 | `interocular=5` → 5 | same |
| 180 | 2 | i16 | stereo mode | 0 | `stereo=1` → 1, `=2` → 2 | `mandel-stereo*` |
| 182 | 2 | i16 | outside colouring | -1 | -1 = iter (default), -2 = real, -3 = imag, -4 = mult, -5 = summ, -6 = atan, -7 = fmod, -8 = tdis; n ≥ 0 = solid colour (`outside=7` → 7) | `mandel-outside-*` |
| 184 | 8 | f64 | x of 3rd corner (bottom-left) | -2.5 | equals xmin unless 6-value corners are given: `corners=…/-2.0625/-1.1875` → -2.0625 | `mandel-corners-6`, `mandel-centermag` |
| 192 | 8 | f64 | y of 3rd corner (bottom-left) | -1.5 | equals ymin unless given: → -1.1875 | same |
| 200 | 1 | ASCII | pass mode | `g` | `1`, `2`, `3`, `g`, `b`, `t`, `s`, `d`, `o`. `potential=…/16bit` and `truecolor=yes` force `1` | `mandel-pass*` |
| 201 | 1 | — | unknown | 0 | | |
| 202 | 2 | i16 | calculation status (hypothesis) | 4 | 4 = calculation finished; 2 = interrupted (resume block present); -1 = image re-saved by a batch load; 0 seen when a load forced deep-zoom mode | `interrupted/*`, `verify/*-resaved` |
| 204 | 2 | u16 | **total length of all application extensions** in the file, framing included | 521 | 521 = `fractint001` alone (2+12+2+504+1). Add 409 for `002`, 82 for `003`, 216 for `007`, or the `005` size. Not validated on load (0 is accepted) | all; `verify/edit-extlen-zero*` |
| 206 | 2 | — | unknown (maybe the high half of 204) | 0 | always 0 | |
| 208 | 2 | i16 | distance estimator, low 16 bits | 0 | `distest=1` → 1, `3/2` → 3, 70000 → `70 11` | `mandel-distest-*` |
| 210 | 2 | i16 | **unknown** | 1 | refilled with 1 when the version is 0 | |
| 212 | 2 | i16 | bailout, 16-bit | 0 | 0 = type default. 4, 100, 1000 as given. 0 when the value doesn't fit (100000) | `mandel-bailout-*` |
| 214 | 4 | u32 | calculation time, 1/100 s | 16 | varies between runs. Interrupted: 545 (stopped after 6 s), 1949 (after 20 s). Kept on reload | §3 |
| 218 | 1 | u8 | function 1 index | 0 (sin) | see §6.2 | `fnfn-*`, `mandelfn-cosh` |
| 219 | 1 | u8 | function 2 index | 6 (sqr) | | same |
| 220 | 1 | u8 | function 3 index | 2 (sinh) | | `fnfn-fnset-*` |
| 221 | 1 | u8 | function 4 index | 3 (cosh) | | same |
| 222 | 2 | i16 | finite attractor | 0 | `finattract=yes` → 1 | `mandel-finattract` |
| 224 | 16 | — | **unknown** | 0 | always 0 | |
| 240 | 2 | i16 | periodicity check | 1 | 0, 2, -3 as given. `inside=atan` and `sierpinski` force 0 | `mandel-periodicity*` |
| 242 | 2 | i16 | 16-bit potential flag | 0 | `potential=…/16bit` → 1 | `mandel-potential-16bit` |
| 244 | 2 | — | unknown | 0 | | |
| 246 | 2 | u16 | **unknown** | 15168 (`40 3b`) | constant; refilled when the version is 0 | |
| 248 | 2 | u16 | **unknown** | 1 | constant. Cleared to 0 by a load with version 0 | `verify/zero-template-v0-resaved` |
| 250 | 2 | u16 | program version (hypothesis) | 2004 (`d4 07`) | constant; refilled when the version is 0 | |
| 252 | 2 | — | unknown | 0 | | |
| 254 | 2 | i16 | 3D: transparent range low | 0 | `transparent=13/14` → 13 | `mandel-3d-transparent` |
| 256 | 2 | i16 | 3D: transparent range high | 0 | → 14 | same |
| 258 | 2 | i16 | 3D: ambient | 20 | 10 | `mandel-3d-ambient` |
| 260 | 2 | i16 | 3D: haze | 0 | 12 | `mandel-3d-haze` |
| 262 | 2 | i16 | 3D: randomize | 0 | 5 | `mandel-3d-randomize` |
| 264 | 2 | i16 | colour-cycle range low | 1 | `cyclerange=10/200` → 10 | `mandel-cyclerange` |
| 266 | 2 | i16 | colour-cycle range high | 255 | → 200 | same |
| 268 | 2 | i16 | distance estimator width factor | 71 | `distest=3/2` → 2 | `mandel-distest-3-2` |
| 270 | 8 | f64 | param 3 | 0 | popcorn 3.0 (default) | `mandel-params-10`, `popcorn-base` |
| 278 | 8 | f64 | param 4 | 0 | lorenz 1.0 (default) | `mandel-params-10`, `lorenz-base` |
| 286 | 2 | i16 | fill colour | -1 | -1 = off, `fillcolor=5` → 5 | `mandel-fillcolor-5` |
| 288 | 8 | f64 | julibrot from/to 1 | -0.83 | `julibrotfromto=1.5/-1.5/0.75/-0.75` → 1.5 | `julibrot-fromto` |
| 296 | 8 | f64 | julibrot from/to 2 | -0.83 | → -1.5 | same |
| 304 | 8 | f64 | julibrot from/to 3 | 0.25 | → 0.75 | same |
| 312 | 8 | f64 | julibrot from/to 4 | -0.25 | → -0.75 | same |
| 320 | 2 | i16 | julibrot 3D value 1 (depth steps) | 128 | `julibrot3d=17/18/19/20/21/2.25` → 17 | `julibrot-3d` |
| 322 | 4 | f32 | julibrot 3D value 2 | 8.0 | → 18.0 | same |
| 326 | 4 | f32 | julibrot 3D value 3 | 8.0 | → 19.0 | same |
| 330 | 4 | f32 | julibrot 3D value 4 | 7.0 | → 20.0 | same |
| 334 | 4 | f32 | julibrot 3D value 5 | 10.0 | → 21.0 | same |
| 338 | 4 | f32 | julibrot 3D value 6 | 24.0 | → 2.25. A 7th or 8th value changed nothing | `julibrot-3d`, `-3d-8values` |
| 342 | 4 | f32 | julibrot eye separation | 2.5 | `julibroteyes=3.75` → 3.75 | `julibrot-eyes` |
| 346 | 2 | i16 | julibrot inner type (hypothesis) | 1 | 6 (= julia's type number) for every julibrot file | `julibrot-*` |
| 348 | 2 | i16 | **unknown** | 1 | constant | |
| 350 | 4 | — | unknown | 0 | | |
| 354 | 48 | 6×f64 | params 5–10 | 0 | `params=0.01/…/0.11`: 354 → 0.05, 362 → 0.06, …, 394 → 0.11 | `mandel-params-10` |
| 402 | 4 | i32 | bailout, 32-bit | 0 | 4, 100, 1000, 100000. The 2nd `decomp=` value also lands here (`decomp=2/70000` → 70000) | `mandel-bailout-*`, `mandel-decomp-2-70000` |
| 406 | 2 | i16 | bailout test | 0 | 0 = default (modulus), `bailoutest=real` → 1, `manh` → 5 | `mandel-bailoutest-*` |
| 408 | 4 | i32 | **max iterations (authoritative)** | 150 | 32768, 65535, 65536, 100000 stored in full. Wins over offset 8 on load | `mandel-maxiter-*`, `verify/edit-maxiter-*` |
| 412 | 2 | i16 | deep-zoom flag | 0 | 1 when `fractint005` is present | `mandel-deep-*` |
| 414 | 2 | i16 | deep-zoom size N | 0 | 12 (mag 1e13–1e20), 16 (1e30), 28 (1e60), 8 (a forced case). Sets the `fractint005` layout | same |
| 416 | 2 | — | unknown | 0 | | |
| 418 | 2 | i16 | old distance-estimator colours | 0 | `olddemmcolors=yes` → 1 | `mandel-olddemmcolors` |
| 420 | 4 | i32 | log palette mapping, full | 0 | -3 → `fd ff ff ff`, 100000 in full | `mandel-logmap-*` |
| 424 | 4 | i32 | distance estimator, full | 0 | 70000 in full | `mandel-distest-70000` |
| 428 | 8 | f64 | invert radius | 0 | `invert=1/0.5/0.25` → 1.0, `3/0.125/-0.5` → 3.0 | `mandel-invert*` |
| 436 | 8 | f64 | invert centre x | 0 | 0.5, 0.125 | same |
| 444 | 8 | f64 | invert centre y | 0 | 0.25, -0.5 | same |
| 452 | 6 | — | unknown | 0 | | |
| 458 | 8 | f64 | **unknown** | 0.01 | constant | |
| 466 | 2 | i16 | no-bof flag | 0 | `nobof=yes` → 1 | `mandel-nobof` |
| 468 | 2 | i16 | orbit interval | 1 | `orbitinterval=3` → 3 | `mandel-orbitinterval` |
| 470 | 2 | — | unknown | 0 | | |
| 472 | 2 | i16 | orbit delay | 0 | `orbitdelay=7` → 7 | `mandel-orbitdelay` |
| 474 | 8 | f64 | math tolerance 1 | 0.05 | `mathtolerance=0.02/0.03` → 0.02. **If it's 0, a load switches to deep-zoom mode** | `mandel-mathtolerance`, `verify/zero-template-v17*` |
| 482 | 8 | f64 | math tolerance 2 | 0.05 | → 0.03 | same |
| 490 | 14 | — | unknown | 0 | always 0 | |

Notes:

- Two maxiter fields: offset 8 holds `min(maxiter, 32767)` and 408 the full
  value. When loading, XFractint uses 408 and rewrites 8 from it
  (`verify/edit-maxiter-u32*`: 8=150, 408=1000 → re-saved 8=1000;
  `verify/edit-maxiter-i16*`: 8=1000, 408=150 → re-saved 8=150).
- Two bailout fields: 212 (16-bit, 0 when the value doesn't fit) and 402
  (32-bit). The same pattern holds for logmap (106/420) and distest (208/424).
  Writers should fill both.
- Fields marked *encoding unclear* (86–97 and 108–119) hold f32 bit patterns,
  but the scale isn't consistent across values. Prefer the f64 copies (invert
  at 428–451). For potential there is no other copy: write what XFractint
  writes for the same command line, or copy the baseline zeros if potential
  isn't used.
- Bytes we never saw change are listed as unknown with their constant value.
  Copy them from the baseline (Appendix A).

### 2.2 Pixel ↔ coordinate mapping (confirmed for unrotated views)

Column `c` (0…w−1) and row `r` (0…h−1) map to

    x = xmin + c · (xmax − xmin) / (w − 1)
    y = ymax − r · (ymax − ymin) / (h − 1)

Row 0 is the **top** (ymax). Evidence: we computed escape counts ourselves for
3000 random pixels of `mandel-pass1` and `mandel-corners` (the latter isn't
symmetric). This mapping matched far better than `/w` or pixel-centre
mappings (§9). With a 3rd corner that differs from (xmin, ymin), the view is
rotated or skewed. That case wasn't analysed beyond storage.

### 2.3 `fractint003`: entry name (66 bytes)

| Off | Width | Type | Meaning | Observed | Evidence |
|---|---|---|---|---|---|
| 0 | 48 | ASCII, NUL-padded | entry name inside the formula / L-system / IFS file | `ffquad`, `ffcube`, `ffkoch`, `fftri` | `formula-*`, `lsystem-ffkoch`, `ifs-fftri` |
| 48 | 2 | i16 | **unknown** (hypothesis: "is a formula") | 1 for both formulas, 0 for L-system and IFS | same |
| 50 | 16 | — | unknown | 0 | |

The file name itself (`ff.frm` and so on) is **not** stored anywhere. The
type number (§6) says which kind of file to look in. The 48-byte name width
is an upper bound: the longest name we tried was 6 characters.

### 2.4 `fractint005`: arbitrary-precision coordinates

Present when the view is too small for doubles. We saw it at magnification
1e13 and above, but not at 1e12 (`mandel-deep-mag1e12` vs `-mag1e13`). It
also appears on load when the math tolerance fields are 0. With
N = `fractint001`[414], the payload is **22 slots of N+6 bytes each**
(396 / 484 / 748 bytes for N = 12 / 16 / 28):

| Slot | Content | Evidence |
|---|---|---|
| 0–3 | xmin, xmax, ymin, ymax | `mandel-deep-*` |
| 4–5 | x3rd, y3rd | same |
| 6–15 | params 1–10 (6 = real of c, 7 = imag of c for julia) | `julia-deep` |
| 16–21 | a second copy of slots 0–5 (identical in every file) | all deep files |

Each slot is a **signed little-endian mantissa of M = N+4 bytes followed by an
i16 exponent `e`**. Its value is

    value = mantissa / 2^(8·(M−1)) × 256^(e+1)

and the exponent is chosen so that `1/256 ≤ |value| / 256^(e+1) < 1`.
Confirmed for e = −7 (±1e−15), −3, −2, −1, 0 (−1.75…) and 1 (300.5)
(`mandel-deep-*`). For `mandel-deep-corners` the 34-digit corner string
`-0.743643887037158704752191506114774` was recovered from slot 0 to all its
digits, so this block keeps more precision than the doubles at 12–43.
XFractint still fills the f64 corners with the nearest doubles.

### 2.5 `fractint007`: orbit window (200 bytes, only with `passes=o`)

| Off | Width | Type | Meaning | Base `mandel-passo` | Evidence |
|---|---|---|---|---|---|
| 0 | 48 | 6×f64 | orbit window xmin, xmax, ymin, ymax, x3rd, y3rd | same as the image corners | `mandel-passo-orbitcorners` (`orbitcorners=-1.125/0.875/-0.4375/0.3125` stored here) |
| 48 | 2 | i16 | orbit window set explicitly | 0 → 1 | same |
| 50 | 1 | ASCII | orbit draw mode | `r`; `orbitdrawmode=l` → `l` | `mandel-passo-drawmode-l` |
| 51 | 149 | — | unknown | 0 | |

---

## 3. Bytes that differ between two renders of the same case

Repeated runs of every case (`tools/blackbox/out/complete` vs `complete-1`)
differ only in `fractint001`:

| Offset | Field | Why it varies |
|---|---|---|
| 98–99 | random seed (u16) | Without `rseed=`, it's the low 16 bits of the Unix time when the calculation started. Example: `mandel-base` holds 0x9C83, and its file mtime was 1792253061 = 0x6AD39C85, 2 s later. Over a whole corpus run the value rose by about 1 per second. |
| 214–217 | calculation time, 1/100 s (u32) | Wall-clock noise (16 vs 15, 2 vs 1). |

Pixel data is byte-identical between repeats. Tests comparing our output with
the corpus should mask offsets 98–99 and 214–217 of `fractint001`, unless the
case sets `rseed=`.

---

## 4. Resume data: `fractint002` (392 bytes)

Present only when the image was saved before the calculation finished. Then
`fractint001`[202] = 2 and [204] counts this block too (930). All 392 bytes
are given as 4-byte little-endian slots. Every value we saw fits in 16 bits
and the upper 2 bytes are always 0, so we can't tell `u16 + pad` from `i32`.
Write `i32`.

| Off | Meaning (our name) | Observed | Status |
|---|---|---|---|
| 0 | **unknown** (format marker?) | 2 everywhere | copy |
| 4 | **unknown** | 1 everywhere | copy |
| 8 | region left column | 0 | hypothesis |
| 12 | region right column = width − 1 | 799; 399 for a 400-wide image | confirmed (`int-viewwindows-2`) |
| 16 | first row not yet final | pass1: same as 24; 2-pass, first pass: 0; second pass: same as 24; guessing: 0 | hypothesis |
| 20 | region bottom row = height − 1 | 599; 299 for 300-high | confirmed |
| 24 | current row (1, 2, 3-pass modes) | see table below | confirmed against pixels |
| 28 | **unknown** | 16 in every XFractint-started render; **0** after a resumed calculation was saved again | copy 16 |
| 32 | pass index (2-pass, guessing) or mode counter | see table | partly confirmed |
| 36 | current column in the current row (1, 2, 3-pass modes) | see table | confirmed against pixels |
| 40–391 | unknown | always 0 | copy zeros |

Values across the interrupted corpus (pixels checked by decoding the image
ourselves and finding the last row with data):

| File | passes | [16] | [24] | [32] | [36] | What the pixels show |
|---|---|---|---|---|---|---|
| `int-pass1-early` | 1 | 30 | 30 | 0 | 0 | rows 0–29 done, row 30 empty |
| `int-pass1-late` | 1 | 93 | 93 | 0 | 743 | rows 0–93 have data; row 93 is the partial one |
| `int-float` | 1 | 48 | 48 | 0 | 741 | rows 0–48 have data |
| `int-viewwindows-2` (400×300) | 1 | 88 | 88 | 0 | 350 | rows 0–88 have data |
| `int-pass2` | 2 | 0 | 180 | 0 | 350 | first (coarse) pass: rows 180–181 hold blocks up to column 349 |
| `int-pass2-late` | 2 | 3 | 3 | 1 | 759 | whole image coarse, second pass at row 3 |
| `int-pass3` | 3 | 0 | 148 | 0 | 760 | every row has data (earlier coarse pass) |
| `int-passg-early` | g | 0 | 132 | 2 | 0 | every row has data (guess blocks) |
| `int-passg` | g | 0 | 74 | 3 | 0 | every row has data |
| `int-passt` | t | 0 | 4139 | 8812 | 0 | values aren't rows. Tesseral-specific, opaque |
| `int-passd` | d | 0 | 0 | 15774 | 0 | [32] looks like a diffusion step counter. Opaque |

Confirmed:

- Pass modes 1, 2 and 3 resume at column [36] of row [24]. Every pixel before
  it has been computed. A 1-pass render picks up there when resumed
  (`verify/interactive-resume-int-pass1-early`: rows 0–29 → 0–91).
- Row 0 / column 0 with [16]=[24]=[32]=[36]=0 makes XFractint compute the
  whole image from scratch with the block's parameters (§8).

Hypothesis:

- [32] is the pass number within a multi-pass mode: 0 = first pass, 1 = second
  pass for passes=2. For guessing it's a pass or level counter: the save
  made after 5 s holds 2 and the one made after 10 s holds 3. These are
  separate runs, so the direction isn't proven.
- [16] is where the final (finest) pass begins.
- For boundary tracing (`b`), XFractint didn't save an interrupted file, so
  that mode has no data.

Whether the image is interrupted shows in four places: `fractint002` is
present, `fractint001`[202] = 2, [204] includes 409, and [214] holds the time
used so far.

---

## 5. Writer's guide

### 5.1 Minimal file XFractint accepts (verified)

1. `GIF89a`, logical screen = image size, flags `D7`, background `00`, aspect
   `31`, then a 256-entry global color table (6-bit values replicated as in
   §1.4).
2. One image: descriptor `2C 0000 0000 w h 00`, LZW minimum code size 8,
   non-interlaced, row 0 = top.
3. Optional blocks (`fractint002`/`003`/`005`/`007`), then `fractint001` last,
   then `3B`.

Verified with files whose LZW stream was written by our own encoder (literal
codes, clear code every 254 symbols). XFractint doesn't need its own
compression layout.

### 5.2 `fractint001` recipe

Start from the 504-byte baseline in Appendix A (`mandel-base.gif`) and
overwrite these fields:

| Parameter | Write |
|---|---|
| type | 10: i16 type number (§6). For formula/L-system/IFS, also add `fractint003` with the entry name |
| corners | 12/20/28/36: f64 xmin, xmax, ymin, ymax. 184/192: f64 x3rd, y3rd (= xmin, ymin for an unrotated view) |
| max iterations | 408: i32 value, **and** 8: i16 `min(value, 32767)` |
| params | 44, 52, 270, 278, 354, 362, …, 394: f64 params 1–10 |
| bailout | 402: i32 value, **and** 212: i16 value if ≤ 32767, else 0. 0 means the type's default |
| bailout test | 406: i16 (0 = default) |
| inside | 104: i16 (§2.1 codes) |
| outside | 182: i16 (§2.1 codes) |
| pass mode | 200: ASCII `1`, `2`, `3`, `g`, `b`, `t`, `d`, `o`, … |
| functions | 218–221: u8 indices (§6.2) |
| status | 202: i16 4 for a finished image, 2 when adding `fractint002` |
| extension length | 204: u16 total bytes of all application extensions (framing included). Not checked on load, but write it correctly |
| seed | 98: u16 seed, 100: 1 if it should count as user-given |
| time | 214: u32 (0 is fine) |
| screen size | 70/72: i16. XFractint writes its screen size here (800/600). We wrote 800/600 for 800×600 images |
| float | nothing to write: no float flag found (§5.6) |

Keep **76 = 17** and the math tolerance doubles at 474/482 (0.05 / 0.05)
from the baseline. Loads with these zeroed behave differently (§5.3). Keep
every other byte from the baseline.

### 5.3 What happens when baseline bytes are left out (verified)

- Unknown bytes zeroed, 76 = 0 (`verify/zero-template-v0-resaved`):
  XFractint loads type, corners, maxiter and params. It **resets** inside,
  outside, passes, bailout and biomorph to defaults and fills in defaults for
  many unknown fields (68, 75, 122, 210, 246, 250, 3D values, julibrot values,
  …).
- 76 = 17, rest zeroed (`verify/zero-template-v17-resaved`): those fields are
  honoured, but because the math tolerances are 0, the re-save switched to
  deep-zoom mode ([412] = 1, N = 8, `fractint005` added).
- 76 = 17 plus tolerances restored (`…-v17-tol-resaved`): the re-saved block
  equals the input except for [202].
- All unknown ranges copied from the baseline (`…-full-resaved`): same.

So the safe recipe is to **copy the whole baseline**.

### 5.4 Resume block to start a fresh calculation (verified)

To make XFractint calculate an image itself from the block's parameters (for
example, if the writer has no pixels), write status 2 and a `fractint002`
block made of these i32 values, followed by zeros up to 392 bytes:

    [0]=2 [4]=1 [8]=0 [12]=w-1 [16]=0 [20]=h-1 [24]=0 [28]=16 [32]=0 [36]=0

with pass mode `1` at offset 200. Loading it interactively and pressing Enter
computes the whole image.

To resume a partly finished 1-pass image, set [16] = [24] = the row and
[36] = the column where work should continue.

### 5.5 Deep zoom

If the writer needs more precision than doubles give, set [412] = 1, choose N
(≥ 12 seen), write the nearest doubles at 12–43 and 184–199, and add
`fractint005` (§2.4) with slots 0–5 and 16–21 set to the corners and slots
6–15 set to the params. This layout came from reading XFractint's own files.
**A file we wrote with this block was not load-tested.**

### 5.6 Things we could not encode

- **Float vs integer math**: `float=yes` and `float=no` produced identical
  bytes and pixels in batch mode, and interrupted `int-float` matches
  `int-pass1-*` apart from time and progress. Either this build always uses
  one kind of math or it records the choice somewhere we couldn't see. A
  writer can't express it. Copy the baseline.
- Potential and invert f32 fields (86–97, 108–119): their encoding is unclear.

---

## 6. Fractal type identification

### 6.1 Type numbers (i16 at `fractint001`[10])

The type is stored as a **number**, not a name. Observed (type name as given
on the command line → number):

| Type | Number | Default corners stored (xmin, xmax, ymin, ymax) | Evidence |
|---|---|---|---|
| newtbasin | 2 | -2, 2, -1.5, 1.5 | `newtbasin-params5` |
| mandel | 4 | -2.5, 1.5, -1.5, 1.5 | `mandel-base` |
| newton | 5 | -2, 2, -1.5, 1.5 | `newton-base` |
| julia | 6 | -2, 2, -1.5, 1.5 | `julia-base` |
| mandelfn | 8 | -8, 8, -6, 6 | `mandelfn-base` |
| manowar | 9 | -2.5, 1.5, -1.5, 1.5 | `manowar-base` |
| ifs | 26 | -8, 8, -1, 11 | `ifs-fftri` |
| fn*fn | 35 | -4, 4, -3, 3 | `fnfn-base` |
| popcorn | 61 | -3, 3, -2.25, 2.25 | `popcorn-base` |
| lorenz | 63 | -15, 15, 0, 30 | `lorenz-base` |
| formula | 73 | -2, 2, -1.5, 1.5 | `formula-*` |
| sierpinski | 74 | -4/3, 96/45, -0.9, 1.7 | `sierpinski-base` |
| lambda | 75 | -1.5, 2.5, -1.5, 1.5 | `lambda-base` |
| barnsleym1 | 76 | -2, 2, -1.5, 1.5 | `barnsleym1-base` |
| lsystem | 107 | -1, 1, -1, 1 | `lsystem-ffkoch` |
| julibrot | 143 | -2, 2, -1.5, 1.5 | `julibrot-*` |
| mandel4 | 155 | -2, 2, -1.5, 1.5 | `mandel4-base` |
| julia4 | 156 | -2, 2, -1.5, 1.5 | `julia4-base` |

`mandelfp`, `juliafp` and `manzpowr` were rejected as unknown type names by
this build. Numbers for other types have to be measured the same way: render
`type=<name>` and read offset 10.

Default params seen: julia 0.3/0.6, lambda 0.85/0.6, julia4 0.6/0.55,
newton p1 = 3, newtbasin p1 = 5 (`params=5`), popcorn p1 = 0.05 and p3 = 3,
lorenz p1 = 0.02, p2 = 5, p3 = 15 and p4 = 1, lsystem p1 = 2.

### 6.2 Function indices (u8 at 218–221)

From `fnfn-fnset-a`…`-g`, `fnfn-sin-exp` and `mandelfn-cosh` (each set gives
four functions in order):

| Index | Function | Index | Function | Index | Function |
|---|---|---|---|---|---|
| 0 | sin | 11 | tanh | 22 | atanh |
| 1 | cosxx | 12 | cotan | 23 | cabs |
| 2 | sinh | 13 | cotanh | 24 | abs |
| 3 | cosh | 14 | flip | 25 | sqrt |
| 4 | exp | 15 | conj | 26 | floor |
| 5 | log | 16 | zero | 27 | ceil |
| 6 | sqr | 17 | asin | 28 | trunc |
| 7 | recip | 18 | asinh | 29 | round |
| 8 | ident | 19 | acos | 30 | one |
| 9 | cos | 20 | acosh | | |
| 10 | tan | 21 | atan | | |

The baseline slots are `00 06 02 03` (sin, sqr, sinh, cosh). XFractint writes
all four slots even for types that use no functions.

---

## 7. Interrupted vs completed: summary

| | Completed | Interrupted |
|---|---|---|
| blocks | `fractint001` (+ optional 003/005/007) | `fractint002`, `fractint001` |
| 001[202] status | 4 | 2 |
| 001[204] ext length | 521 (+ optional) | 930 |
| 001[214] time | full calculation time | time spent so far |
| everything else in 001 | — | identical to a finished render of the same case |

Interrupted renders that happened to finish before the key press
(`int-julia`) look exactly like completed ones.

---

## 8. Verification performed

All tests ran in `fractalforge-blackbox`. "Load interactively + Enter" means
XFractint was started with `filename=in.gif`. After 10 s we pressed Enter,
after another 45 s we pressed `s`, and we read the saved file.

| Test | Input (in `verify/`) | Result | Confirms |
|---|---|---|---|
| R1 | `crafted-r1.gif`: written from scratch. Zero pixels, baseline 001 with type 4, corners -0.75/-0.70/0.10/0.14, maxiter 1000 (8 and 408), bailout 100 (212 and 402), inside -60, outside -2, passes `1`, plus a resume-from-zero 002 | `crafted-r1-resumed.gif` was still in progress (row 393). **All 315,087 computed pixels equal** `complete/verify-ref-r1` (batch: `type=mandel corners=… maxiter=1000 bailout=100 inside=bof60 outside=real passes=1`). Re-saved 001 equals the reference except seed, time, status and extension length | type, corners, maxiter, bailout, inside, outside, passes, resume block |
| R2 | `crafted-r2.gif`: julia (6), corners -2/2/-1.5/1.5, params -0.8/0.156, maxiter 255 | **Pixel-identical** to `verify-ref-r2`. Block identical apart from seed and time | type 6, params 1–2 |
| R3 | `crafted-r3.gif`: fn*fn (35), corners -4/4/-3/3, functions cos/sinh (9, 2) | **Pixel-identical** to `verify-ref-r3` | function indices |
| R4 | `crafted-r4.gif`: lambda (75), params 1/0.4 | **Pixel-identical** to `verify-ref-r4` | type 75 |
| maxiter | `edit-maxiter-u32.gif` (8=150, 408=1000), `edit-maxiter-i16.gif` (8=1000, 408=150), batch reload | re-saved 8 follows 408 | 408 is authoritative |
| ext length | `edit-extlen-zero.gif` (julia-params with 204=0) | loaded normally. Re-saved with 521 | 204 isn't validated |
| templates | `zero-template-*-resaved.gif` | see §5.3 | 76 = version gate; 474/482 control deep zoom |
| batch reload | `batch-reload-int-pass1-early.gif` | batch load of a partial file saves the pixels unchanged, **drops 002** and sets 202 = -1 | batch mode doesn't resume |
| interactive resume | `interactive-resume-int-pass1-early.gif` | resumed from row 30, saved at row 92 with a new 002 ([28] became 0) | resume semantics |

Inferred but **not** load-tested: the deep-zoom write path (§5.5), the 003
block written by a third party, the 007 block, and the 3D, stereo and
julibrot fields. Those were only read from XFractint's own saves.

---

## 9. Appendix: iteration counts → colour indices (black-box inference)

Method: decode the GIF LZW ourselves. For 3000 random pixels, compute
z → z² + c from z = 0 with the §2.2 mapping. n is the number of iterations
completed before |z|² > 4 first holds (0 if the first iterate escapes). Then
compare n with the pixel's colour index (`mandel-pass1`, `mandel-corners`,
`mandel-maxiter-1000`).

- **Default outside (iter)**: colour = n mod 256, except **n = 0 is drawn as
  colour 1**. 2,110 of 2,618 escaped samples match n mod 256 exactly, and all
  503 others have n = 0 and colour 1 (`mandel-pass1`).
- **Default inside**: colour 0. All non-escaping samples are 0, and
  `inside=0` gives pixel-identical output to the default (`mandel-inside-0`).
- About 0.3% of escaped samples are unexplained (colour 16 lower than
  predicted, a few others). These may be periodicity-check effects or
  rounding near the boundary.
- `>` vs `≥` 4 made no difference on these samples.
- Whether n = 256k maps to 0 or 1 wasn't checked.

---

## Appendix A: baseline `fractint001` from `mandel-base.gif` (504 bytes)

Copy this as the template. Offsets 98–99 (seed 0x9C83) and 214–217 (time 16)
vary per render.

```
   0 000  46 72 61 63 74 61 6c 00 96 00 04 00 00 00 00 00  Fractal.........
  16 010  00 00 04 c0 00 00 00 00 00 00 f8 3f 00 00 00 00  ...........?....
  32 020  00 00 f8 bf 00 00 00 00 00 00 f8 3f 00 00 00 00  ...........?....
  48 030  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................
  64 040  00 00 00 00 13 00 20 03 58 02 00 01 11 00 00 00  ...... .X.......
  80 050  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................
  96 060  00 00 83 9c 00 00 ff ff 00 00 00 00 00 00 00 00  ................
 112 070  00 00 00 00 00 00 00 00 00 00 90 01 e7 03 00 00  ................
 128 080  3c 00 1e 00 00 00 5a 00 5a 00 1e 00 00 00 00 00  <.....Z.Z.......
 144 090  00 00 00 00 00 00 01 00 ff ff 01 00 00 00 14 00  ................
 160 0a0  00 00 00 00 04 00 00 00 00 00 04 00 50 00 64 00  ............P.d.
 176 0b0  00 00 00 00 00 00 ff ff 00 00 00 00 00 00 04 c0  ................
 192 0c0  00 00 00 00 00 00 f8 bf 67 00 04 00 09 02 00 00  ........g.......
 208 0d0  00 00 01 00 00 00 10 00 00 00 00 06 02 03 00 00  ................
 224 0e0  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................
 240 0f0  01 00 00 00 00 00 40 3b 01 00 d4 07 00 00 00 00  ......@;........
 256 100  00 00 14 00 00 00 00 00 01 00 ff 00 47 00 00 00  ............G...
 272 110  00 00 00 00 00 00 00 00 00 00 00 00 00 00 ff ff  ................
 288 120  8f c2 f5 28 5c 8f ea bf 8f c2 f5 28 5c 8f ea bf  ...(\......(\...
 304 130  00 00 00 00 00 00 d0 3f 00 00 00 00 00 00 d0 bf  .......?........
 320 140  80 00 00 00 00 41 00 00 00 41 00 00 e0 40 00 00  .....A...A...@..
 336 150  20 41 00 00 c0 41 00 00 20 40 01 00 00 00 00 00   A...A.. @......
 352 160  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................
 368 170  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................
 384 180  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................
 400 190  00 00 00 00 00 00 00 00 96 00 00 00 00 00 00 00  ................
 416 1a0  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................
 432 1b0  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................
 448 1c0  00 00 00 00 00 00 00 00 00 00 7b 14 ae 47 e1 7a  ..........{..G.z
 464 1d0  84 3f 00 00 01 00 00 00 00 00 9a 99 99 99 99 99  .?..............
 480 1e0  a9 3f 9a 99 99 99 99 99 a9 3f 00 00 00 00 00 00  .?.......?......
 496 1f0  00 00 00 00 00 00 00 00                          ........
```

Framing in the file: `21 ff 0b 66 72 61 63 74 69 6e 74 30 30 31`, then
`ff` + 255 bytes, `f9` + 249 bytes, `00`.

## Appendix B: other blocks

`fractint002` from `int-pass1-late.gif` (392 bytes; bytes 40–391 are all 0).
Framing: `21 ff 0b "fractint002" ff …255… 89 …137… 00`.

```
   0 000  02 00 00 00 01 00 00 00 00 00 00 00 1f 03 00 00
  16 010  5d 00 00 00 57 02 00 00 5d 00 00 00 10 00 00 00
  32 020  00 00 00 00 e7 02 00 00 00 00 00 00 00 00 00 00
  (zeros to offset 391)
```

`fractint003` from `formula-ffquad.gif` (66 bytes, one sub-block `42`):

```
   0 000  66 66 71 75 61 64 00 00 00 00 00 00 00 00 00 00  ffquad..........
  16 010  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
  32 020  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
  48 030  01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
  64 040  00 00
```

`fractint005` from `mandel-deep-corners.gif` (N = 12, slots of 18 bytes; first
108 bytes = slots 0–5; slots 6–15 are zero; slots 16–21 repeat slots 0–5):

```
   0 000  d6 cb 18 58 10 90 93 62 0d 1a 4e e1 8d a0 41 ff
  16 010  ff ff a9 3f dd a4 1b 12 2b 92 38 1c 4e e1 8d a0
  32 020  41 ff ff ff e4 01 20 55 d1 6f 2b 12 5b d3 53 ab
  48 030  57 bf 21 00 ff ff 9f f3 ca b1 2c 21 b5 5e 35 d4
  64 040  53 ab 57 bf 21 00 ff ff d6 cb 18 58 10 90 93 62
  80 050  0d 1a 4e e1 8d a0 41 ff ff ff e4 01 20 55 d1 6f
  96 060  2b 12 5b d3 53 ab 57 bf 21 00 ff ff 00 00 00 00
```

Slot 0 = mantissa `d6 cb … 41 ff` (16 bytes), exponent `ff ff` (−1) →
−0.743643887037158704752191506114774.

`fractint007` from `mandel-passo.gif` (200 bytes, one sub-block `c8`; bytes
51–199 are 0):

```
   0 000  00 00 00 00 00 00 04 c0 00 00 00 00 00 00 f8 3f
  16 010  00 00 00 00 00 00 f8 bf 00 00 00 00 00 00 f8 3f
  32 020  00 00 00 00 00 00 04 c0 00 00 00 00 00 00 f8 bf
  48 030  00 00 72 00 00 00 00 00 00 00 00 00 00 00 00 00
```

---

## 10. FractalForge implementation status

Added by the implementer, not part of the black-box analysis. Code:
`crates/formats/src/gif/fractint_blocks.rs`.

- **Read:** `fractint001` for types mandel (4) and julia (6): corners,
  params 1–4, maxiter (408, else 8), bailout (402, else 212, else 4),
  inside (n ≥ 0 or -1), outside -1 and pass modes `1`, `2`, `g`. Anything else
  that is set (third corner, params 5–10, other inside/outside modes,
  biomorph, decomposition, logmap, distance estimator, inversion, finite
  attractor, potential, stereo, deep zoom) is reported as a warning. Other
  type numbers are recognised but not rendered.
- **Write:** §5.2 on top of the Appendix A baseline, with the seed and time
  zeroed and 70/72 set to the image size. Finished renders get status 4 and
  their pass mode. Unfinished renders get the §5.4 "calculate from the start"
  `fractint002` block and pass mode `1`; resuming Fractint-style partial
  progress isn't implemented yet. Our `FRCTFRGE` block goes after the image,
  before the Fractint blocks.
- **Iteration and colour conventions** adopted to match §9:
  the start point isn't tested, escape counts n run 1..=maxiter, and colour
  = (n − 1) mod 256, drawn as 1 where that is 0. Inside is colour 0.

Interop results (`tools/blackbox/interop.sh`, `crates/formats/tests/fractint_interop.rs`):

- Our main block equals XFractint's for the same settings, except the seed,
  the time, and bailout (we write 4 where XFractint writes 0 for "default").
- XFractint batch-reloads a GIF we wrote with the right parameters and every
  colour unchanged. It tolerates our `FRCTFRGE` block (and drops it on
  re-save). When re-saving, XFractint merges palette entries that are equal
  at 6 bits, so palette *indices* can change while colours don't.
- XFractint calculating an unfinished GIF of ours from our blocks matches our
  own render in 99.47% of pixels.
- Rendering corpus parameters with FractalForge matches XFractint's pixels in
  99.4–99.8% of pixels. The rest are near the set boundary, mostly pixels
  where XFractint's count is 16 lower (or occasionally 1 higher) than ours.
  The cause is unknown and is a good next black-box question.
