<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Deep zoom: XFractint's arbitrary-precision mode

Status: **Phase 5 draft, derived from black-box analysis only.** Every rule
below comes from bytes and pixels of files XFractint saved, from files we
edited and loaded back into it, and from parameter files it wrote. No
Fractint, XFractint or Iterated Dynamics source code, headers, help text or
documentation was consulted (see [CLEANROOM.md](../CLEANROOM.md)). All names
are ours.

This spec builds on [gif-fractint-blocks.md](gif-fractint-blocks.md) (block
framing, `fractint001` fields, pixel ↔ coordinate mapping §2.2, the first
description of `fractint005` in §2.4/§5.5) and refines it; §11 lists the
corrections. Colour conventions are those of types.md §1.2 and
gif-fractint-blocks §9; resume data is in [fractint-resume.md](fractint-resume.md);
PAR syntax in [par.md](par.md).

FractalForge does its own deep zoom (perturbation). What it needs from
XFractint is file compatibility: reading and writing `fractint005`, knowing
what XFractint does with such files, and reference pixels.

Labels: **Confirmed** = shown directly by files or load tests.
**Hypothesis** = fits all evidence, not proven. **Unknown** = not determined.

Evidence: `corpus/xfractint/deep-zoom/` (folders `switch/`, `nsize/`,
`types/`, `encoding/`, `pixels/`, `base/`, `crafted/`, `loaded/`,
`interrupt/`, `par/`; command lines in its `cases.txt`, block summaries in
`results.txt`), plus `corpus/xfractint/complete/*deep*`. Scripts:
`tools/blackbox/deep-zoom/`. Unless noted, renders are 400×300
(`viewwindows=2`), palette `uniq.map` (distinct colours, so pixel values are
colour indices). XFractint: Debian `xfractint 20.4.10-5`, amd64 under qemu;
the PAR writer tests used the arm64 build of the same package (par.md).
`C` below is the centre
`-0.743643887037158704752191506114774/0.131825904205311970493132056385139`.

---

## 0. Summary for implementers

- Only **mandel (type 4) and julia (type 6)** ever use arbitrary precision.
  Every other type tried stays in doubles (§1.1).
- In deep mode a save carries a `fractint005` block of **22 slots of N+6
  bytes**, N = `fractint001`[414]. A slot is a little-endian signed fixed-point
  mantissa of N+4 bytes (one byte of integer/sign, N+3 bytes of fraction)
  followed by an i16 base-256 exponent (§3).
- Slots 0–5 are the corners (xmin, xmax, ymin, ymax, x3, y3), slots 6–15 the
  ten params, slots 16–21 a copy of 0–5 (§3.1).
- **What XFractint reads** (load tests, §5): `fractint005` slots 0–5 and the
  params in slots 6/7, interpreted with N from [414]. [414] must match the
  block length. [412], the f64 corners, the f64 params and slots 16–21 are
  ignored when `fractint005` is present. Any N we tried (8, 12, 16, 20) is
  accepted; XFractint picks its own N again and also drops deep mode if the
  view no longer needs it.
- **Writer recipe** (§5.3): [412] = 1, [414] = N (rule in §2), the nearest
  doubles in the f64 fields, `fractint005` with all 22 slots, everything
  else as in a normal save. Verified by loading files built that way from a
  non-deep save (`k10`, `k11`, `k19`, `k21`, `k23`).
- XFractint's deep pixels follow the normal colour rule; an exact
  simulation matches every pixel at mag 1e15 (mandel) and 1e20 (julia), all
  but one at 1e20 (mandel) and 98.8 % at 1e40, where XFractint's own
  limited precision shows (§8).
- Deep renders interrupt and resume like normal ones (`fractint002` +
  `fractint005`); the `r` load resumes them pixel-exactly (§9).
- PAR output writes the deep centre with about log10(mag) + 5 digits and
  the magnification with 7 significant digits (§7).

---

## 1. When XFractint uses arbitrary precision

### 1.1 Types (confirmed)

`center-mag=-0.1/0.65/1e15` with each type (`types/ty-*`), `1e30`
(`types/ty30-*`):

| Result | Types |
|---|---|
| `fractint005` written, [412] = 1 | `mandel` (4), `julia` (6); also `mandel float=yes` |
| no `fractint005`, [412] = 0 | `mandel4`, `julia4`, `lambda`, `mandellambda`, `manowar`, `manowarj`, `barnsleym1`, `barnsleyj1`, `spider`, `magnet1m`, `magnet1j`, `newton`, `newtbasin`, `phoenix`, `mandphoenix`, `fn*fn`, `mandelfn`, `julfn+zsqrd`, `popcorn`, `complexnewton`, `sierpinski`, `marksmandel`, `tetrate`, `formula` |

For the double-only types XFractint changes the stored view instead
(observed, **hypothesis** as to the rule): at mag 1e15 the stored corners
span almost exactly **2×** the requested width and height (factor 1.998
for every type above); at mag 1e30 the corners collapse to neighbouring
doubles (width and height of one or two ulps, width even negative). Pixels
of such views are blocky: `types/tyv-formula` (formula `ffquad` = z² + c at
the `base-v1` view, mag 1e15) has only 27 distinct rows and 117 distinct
columns, the same view as mandel (`types/tyv-mandel-float`) has 300 and
397. At mag 1e12 (no deep mode for anyone) formula and mandel agree except
for 4 pixels (`types/tyv-*-1e12`).

### 1.2 The switch (partly confirmed, rule partly unknown)

Deep mode is used when either of two conditions holds.

**(A) Small pixel spacing.** With centre 0/0 (where doubles are exact to
far below the pixel size) deep mode starts between mag 9.9e12 (no) and
9.95e12 (yes) at 400×300 (`switch/c0-*`, `c0f-*`). The same view at 800×600
switches earlier (C at 8e12 and 9e12: yes, `switch/full-*`; at 400×300: no).
Changing `xmag` (x spacing only) does not move it: `center-mag=0/0/6e12/2`
(x spacing like mag 1.2e13) stays double, `0/0/1.5e13/0.5` is deep
(`c0x*`). **Hypothesis:** deep mode is used when the row spacing
Δy = (ymax − ymin)/(H − 1) is below a constant between 6.72e-16 and
6.76e-16. Independent of `mathtolerance=` (`tol0.001-c0-1e12` double,
`tol0.5-c0-1e13` deep).

**(B) Doubles too coarse for the view, controlled by the 2nd math
tolerance [482].** Centre 300.5/−2.25: deep from mag 5e10 (3e10: no),
far below (A) (`switch/c300-*`). With `mathtolerance=0.001/0.001` it is
deep at 1e10 already, with `0.5/0.5` not even at 1e11. Only the **second**
value matters: `mathtolerance=0.001/0.05` leaves C at 9e12 double,
`0.05/0.001` makes it deep; `0.5/0.05` keeps C at 8e12 deep, `0.05/0.5`
makes it double (`tol*`). A load with [482] = 0 switches a plain
default-view file to deep mode; [474] = 0 does nothing (`loaded/k17`,
`k18`, §5). For centre 300.5, "the largest error of the f64 corners, in
units of the column spacing (W − 1 = 399), exceeds [482]" separates every
case (0.032 → double, 0.053 → deep; 0.032 > 0.001; 0.372 < 0.5), but the
same centre at 800×600 switches at the same mags (3e10 no, 5e10 yes,
`switch/full-c300-*`) although its columns are twice as close. For
centre C it does **not** (C at 8e12 and 9.5e12 are deep, 8.5e12, 9e12 and
9.9e12 are not, with no monotonic measure we could find; `switch/th-*`,
`tools/blackbox/deep-zoom/meas.py`). The measure behind (B) is **unknown**.

FractalForge does not need to reproduce the switch: XFractint decides again
whenever it loads a file (§5), so a writer may emit `fractint005` for any
view it likes, and a reader only has to look for the block.

### 1.3 On load (confirmed)

The decision is re-made when a file is loaded and recalculated:
`fractint005` holding the default view (−2.5…1.5) with [412] = 1 is
rendered in doubles and saved without `fractint005`, pixel-identical to a
normal render (`loaded/k16`). A plain default-view file with [482] = 0 is
rendered in deep mode with **N = 4** (`loaded/k18`; 290 pixels differ from
the double render).

---

## 2. Choice of N (confirmed for 400×300 and 800×600)

N = [414] depends on the pixel spacing, not on the centre (centres 0,
1.2e−5, 0.74 and 300.5 give the same N at the same mag: `nsize/nc*`), and
it changes with the image size (`switch/full-5e22`: N = 16 at 800×600,
N = 12 at 400×300). Observed for centre C at 400×300 (`nsize/n-*`, `nd-*`,
`nb-*`, `nf-*`) and centre 300.5 (`switch/c300-*`):

| N | mag range seen (400×300) |
|---|---|
| 4 | only when forced by [482] = 0 on a shallow view (§1.3) |
| 8 | 5e10 … 5e12 (centre 300.5) |
| 12 | 8e12 … 6.6e22 |
| 16 | 6.75e22 … 6.6e32 |
| 20 | 6.75e32 … 1e41 |
| 24 | 1e42 … 1e51 |
| 28 | 1e52 … 1e61 |
| 32 | 1e62 … 1e70 |
| 36 | 1e71 … 1e80 |
| 40 | 1e81 … 1e89 |
| 44 | 1e90 … 1e99 |
| 48, 68, 88, 108, 128 | 1e100, 1e150, 1e200, 1e250, 1e300 |

Fitted rule (**hypothesis**, reproduces N of all 259 deep 400×300 and
800×600 saves in the corpus, `tools/blackbox/deep-zoom/nrule.py`):

    Δ = min((xmax − xmin)/(W − 1), (ymax − ymin)/(H − 1))
    D = ceil(−log10 Δ)                       (decimal digits needed)
    N = the smallest multiple of 4 with  8·N·log10(2) − K ≥ D,
        K between 3.34 and 3.42 (we use 3.38)

The sharp boundaries (12→16 between mag 6.6e22 and 6.75e22, 16→20 between
6.6e32 and 6.75e32) sit exactly where −log10 Δ crosses 25 and 35, which is
why D is an integer. `xmag=2` (columns twice as close) raises N
(`nsize/nx2-*`), so x spacing counts; whether y spacing counts on its own is
**unknown** (the `min` is a guess). The 190×143 images that
`viewwindows=no` produced (`nsize/big-*`) get the N of a 400-wide image
(2 mismatches in nrule.py): the rule's W/H there is **unknown**.

Larger or smaller N in a file is fine for XFractint (§5), so writers can
use this rule or any N that holds their precision.

---

## 3. `fractint005` encoding (confirmed)

### 3.1 Layout

Payload = 22 slots × (N + 6) bytes, N = `fractint001`[414] (396, 484, 572,
660, 748, … bytes for N = 12, 16, 20, 24, 28; 220 for N = 4, 308 for N = 8).

| Slot | Value | Read on load? |
|---|---|---|
| 0, 1 | xmin, xmax | yes |
| 2, 3 | ymin, ymax | yes |
| 4, 5 | x3, y3 (3rd corner; = xmin, ymin unless rotated/skewed) | yes (written) |
| 6 … 15 | params 1–10 (6/7 = julia c) | 6/7 yes (`loaded/k12`); others not tested |
| 16 … 21 | copy of slots 0–5 | **no** (`loaded/k03`); XFractint rewrites them as a copy |

Corner meaning as in par.md §5.2 (top-left = (xmin, ymax), bottom-right =
(xmax, ymin), bottom-left = (x3, y3)); a rotated view (`par/wdrot.gif`) has
x3, y3 ≠ xmin, ymin. Every XFractint save had slots 16–21 = slots 0–5.

### 3.2 Slot format

With M = N + 4:

    bytes 0 … M−1   m: signed little-endian two's-complement integer
    bytes M, M+1    e: i16 little endian
    value = m / 2^(8(M−1)) × 256^(e+1)

So the mantissa is a fixed-point number in [−1, 1) with M − 1 = N + 3
fraction bytes. **Normal form:** `e` is the smallest integer with
−1 ≤ value / 256^(e+1) < 1. Zero is all bytes 0 (e = 0). 6,159 of the
6,160 slots in XFractint's saves in the corpus are in this form
(`normchk.py`). Examples (`encoding/enc-one`, `enc-negone`, `enc-256`):

| value | e | top mantissa bytes (most significant first) |
|---|---|---|
| 1 | 0 | `00 01 00 …` (1/256 × 256) |
| 0.00390625 = 1/256 | −1 | `00 01 00 …` |
| −1 (a corner) | −1 | `ff 00 00 …` (mantissa −1.0) |
| −1/256 | −2 | `ff 00 00 …` |
| 0.5 | −1 | `00 80 00 …` |
| 256 | 1 | `00 01 00 …` |
| 300.5 (computed from the rule) | 1 | `00 01 2c 80 00 …` |

Negative values with |v| an exact power of 256 therefore use one exponent
less than the positive value. The one exception: the julia parameter −1
(`params=0/-1`, slot 7 of `base/base-j2`) is stored with e = 0 and
mantissa −1/256 (`ff ff 00 …`), i.e. normalised like +1. So both forms
occur for negative powers of 256, and XFractint read our smallest-e form
of −1 in slot 7 correctly (`loaded/k12`). A reader decodes with the formula
and need not care about normalisation. A writer should produce the normal
form (`slot_encode` in `tools/blackbox/deep-zoom/dz.py`); other
non-normalised slots (extra leading zero bytes) were not load-tested.

Precision: 8(N + 3) bits below 256^(e+1). For values in [1/256, 1) that is
about 2.41·(N + 3) decimal places.

### 3.3 Where the values come from (confirmed)

- **Corners from `corners=`**: XFractint's decimal conversion is **not**
  correctly rounded. Compared with the exact decimal value
  (`tools/blackbox/deep-zoom/encchk.py`), stored slots were off by 0 to 306
  units of the last mantissa byte (`encoding/enc-*`). Exact binary values
  (1, 0.5, 0.25) come out exact; `0.1` and `0.2` are truncated to N + 2
  fraction bytes (last byte `00`); `-0.743643887037151` in
  `complete/mandel-deep-corners` is 0.7 units low. **Hypothesis:** the
  conversion works with one byte less than the slot (about 8(N + 2)
  fraction bits); errors stayed below 1.2 units of the second-lowest byte.
  A 60-digit and a 39-digit centre gave identical slots at N = 12
  (`encoding/dig60`, `dig60c`): digits beyond that precision are dropped.
- **Corners from `center-mag=`**: the centre keeps the precision above;
  the half-height 1/mag and half-width (4/3)/(mag·xmag) are only about
  double-accurate (relative error up to ~1e−16 of the view size; e.g.
  `encoding/enc-cmag3` matches mag = the double nearest 3e40). The view
  is therefore exact to far below one pixel; only its size is "double".
- **Params (slots 6–15)** are the **f64 params**, converted exactly:
  `params=-0.12345678901234567890123456789/…` stores
  −0.12345678901234567737…, the double (`encoding/enc-jparams`,
  `complete/julia-deep`). A julia constant typed on the command line has
  only double precision even in deep mode. A slot value loaded from a file
  is used as stored (§5).

### 3.4 `fractint001` in deep mode (confirmed)

| Offset | In XFractint's own deep saves |
|---|---|
| 412 (i16) | 1 (0 in every non-deep save) |
| 414 (i16) | N |
| 416 (2 bytes) | 0 |
| 12–43, 184–199 (f64 corners) | the **nearest doubles** of slots 0–5, so at high mag xmin = xmax etc. 2,064 of 2,088 checked values are the nearest double; the other 24 differ by a few ulps, all for coordinates very close to 0 (1e−20 … 1e−40, `px-m20`, `px-m40`, `px-j20`, `*-i3`), where the double is presumably computed from the centre and mag instead (`f64chk.py`) |
| 44, 52, 270, … (f64 params) | the params; slots 6–15 equal them exactly |
| 474 / 482 (f64) | unchanged: 0.05 / 0.05, or the `mathtolerance=` values |
| everything else | as in a non-deep save |

After loading a file whose f64 corners disagree with its slots, XFractint
renders from the slots but saves the **f64 corners and params unchanged**
from the input (`loaded/k01`, `k04`, `k09`, `k10`, `k12`): the f64 fields
of a deep file are not reliable. Readers must use `fractint005`.

---

## 4. Block order (confirmed)

`fractint005` comes after the image, before `fractint001` (gif spec §1.2).
With resume data the order is `fractint002`, `fractint005`, `fractint001`
(`interrupt/int-i1`, `int-i3`). `fractint001`[204] (total extension bytes)
counts the `fractint005` framing too (3 + 11 + sub-blocks; 934 for
`complete/julia-deep`); our crafted files kept it consistent.

---

## 5. How XFractint reads deep files

### 5.1 Load tests (confirmed)

Each crafted file (`crafted/kNN`, made by `craft.py` from `base/base-v1`
(mandel, centre i + small offset, mag 1e15, maxiter 60, `passes=1`,
`periodicity=0`), `base/base-d` (the default mandel view, no deep mode) or
`base/base-j` (julia c = i, same view)) was loaded with `filename=` and
Enter, and XFractint's save compared with batch renders (`cmp.py`). V2 =
the same view mirrored to centre −i (reference `base/base-v2`).

| File | Edit | XFractint rendered | Saved |
|---|---|---|---|
| `k00` | none | V1, pixel-identical | as input |
| `k01` | slots 0–5 and 16–21 → V2, f64 = V1 | **V2**, identical to `base-v2` | slots V2, f64 still V1 |
| `k02` | slots 0–5 → V2 only | **V2** | slots 16–21 rewritten = 0–5 |
| `k03` | slots 16–21 → V2 only | V1 | slots 16–21 back to V1 |
| `k04` | f64 corners → V2, slots V1 | V1 | f64 V2 kept |
| `k05` | `fractint005` removed, [412] = 1 | V1 **from the f64 corners** (15 of 300 rows equal; the doubles are off by a fraction of a pixel) | deep, slots = the doubles |
| `k06` | slots V2, [412] = 0 | V2 | [412] = 1 |
| `k07`, `k08` | slots V2 re-encoded with N = 20 / N = 8 ([414] set) | V2 | N = 12 |
| `k09` | slots V2, f64 corners all 0 | V2 | f64 0 kept |
| `k10` | `base-d` + `fractint005` V2, [412] = 1, [414] = 12; f64 = default view | V2 | deep, N = 12 |
| `k11` | as k10 + f64 = nearest doubles of V2 | V2 | |
| `k12` | `base-j`: slots 6/7 → (0, −1), f64 params (0, 1) | julia c = **−i** (= `base-j2`) | f64 params still (0, 1) |
| `k13` | `base-j`: f64 params → (0, −1), slots (0, 1) | julia c = i (= `base-j`) | |
| `k15` | slots V2, [414] = 16 but block for N = 12 | garbage view (y0 = 39.2, not deep) | |
| `k16` | slots = default view −2.5/1.5/−1.5/1.5, [412] = 1 | default view in doubles, = `base-d` | no `fractint005`, [412] = 0 |
| `k17` | `base-d` with [474] = 0 | double, = `base-d` | |
| `k18` | `base-d` with [482] = 0 | deep, N = 4 (290 pixels differ from `base-d`) | |
| `k19` | `base-d` + `fractint005` V2 with N = 16, f64 V2 | V2 | N = 12 |
| `k20` | `base-d`, [412] = 1, [414] = 12, f64 V2, no `fractint005` | V2 from the doubles (15 rows equal) | |
| `k21` | `base-d` + `fractint005` V2 with N = 8, f64 V2 | V2 | N = 12 |
| `k22` | `base-d` + `fractint005` V2, [412] = 0, [414] = 0 | garbage (not deep) | |
| `k23` | `base-d` + `fractint005` V2, [412] = 0, [414] = 12, f64 default | **V2** | deep |

### 5.2 What is mandatory (confirmed)

- `fractint005` itself, with **[414] = N matching its length** (22·(N+6)).
  [414] wrong or 0 makes XFractint misread the block (`k15`, `k22`).
- Slots 0–5 (corners) and, for julia, slots 6/7 (c). Slots 16–21 are not
  read.
- Not needed: [412] (`k06`, `k23`), the f64 corners (`k09`, `k10`), the
  f64 params when slots 6/7 are present (`k12`).
- With [412] = 1 but no `fractint005`, XFractint takes the f64 corners
  (`k05`, `k20`).
- N need not follow §2: 8, 12, 16, 20 all load, XFractint re-picks N, and
  the precision of a coarser N (8 at mag 1e15, ~2e−17) was still enough
  for identical pixels (`k08`, `k21`).
- What XFractint does with slots 8–15 (params 3–10) was not tested; mandel
  and julia use only params 1/2.

### 5.3 Writer recipe

1. Start from a normal save of the same view (gif spec §5.2), type 4 or 6.
2. f64 corners and params: the nearest doubles of the exact values (so
   tools that ignore `fractint005` still get the view approximately).
3. [412] = 1, [414] = N (§2, or any multiple of 4 that holds the
   precision; ≥ 8 recommended), [416] = 0, [474]/[482] = 0.05.
4. `fractint005`: slots 0–5 = xmin, xmax, ymin, ymax, x3, y3; slots 6–15
   = params 1–10 (exact values; for julia, c with full precision is read
   back); slots 16–21 = copy of 0–5. Each slot in the normal form of §3.2,
   rounded to nearest.
5. Place it before `fractint001`; keep [204] correct.

Reader: if `fractint005` is present, N = [414], check the length, decode
slots 0–5 (and 6/7 for julia) exactly and ignore the f64 corners;
otherwise use the doubles. A FractalForge-written file following this
recipe is equivalent to `k11`/`k19`.

---

## 6. Command-line and `.par` input (confirmed)

- A single command-line argument longer than **140 characters** crashes
  XFractint (`*** buffer overflow detected ***`, no image): `center-mag=`
  tokens of 140 characters worked, 142–150 crashed (`encoding/len-*`, the
  failed `enc-long`, `enc-third` cases are listed in `results.txt`). This
  limits command-line deep views to about 60 digits per coordinate with
  `center-mag=`.
- In a `.par` entry the same key with 251 characters, and one split over
  two lines with `\`, loaded fine (`encoding/pl-*`, file
  `tools/blackbox/deep-zoom/dz.par`).
- Outside deep mode, corner values were clamped to ±32767.99
  (`corners=65535.75/…/-65536.125/…` was stored as 32767.99 / −32767.99,
  not deep; `encoding/enc-65536`). Whether deep views can lie beyond
  ±32768 is **unknown**.

---

## 7. PAR output of deep views (confirmed unless noted)

Written by XFractint's "save parameters" screen (`b`; `par/wd*.out.par`,
native arm64 build, which gives the same blocks as amd64 apart from
`fractint001`[122–123], par.md). The view key is `center-mag=` when given
as `center-mag`, `corners=` when given as `corners` (as par.md §5.2):

| Case | mag | Written |
|---|---|---|
| `wd12` (not deep) | 1e12 | `center-mag=-0.74364388703715867/+0.13182590420531198/9.999944e+11` |
| `wd13` | 1e13 | `-0.743643887037158705/0.13182590420531197/1e+13` |
| `wd9e14` | 9e14 | `-0.7436438870371587048/0.1318259042053119705/9e+14` |
| `wd15`, `wd3e15` | 1e15, 3e15 | `-0.74364388703715870475/0.13182590420531197049/…` |
| `wdrot` | 1e20, xmag 1.5, rot 30, skew 10 | `-1.234567890123456789012346/0.9876543210987654321098765/1e+20/1.5/30.0000000000000497/10.0000000000001421` |
| `wd5e25` | 5e25 | `-0.743643887037158704752191506115/0.131825904205311970493132056385/5e+25` |
| `wdsm` | 1e30 | `1.234567890123456789012345678901e-5/-2.345678912345678912345678912345e-5/1e+30` |
| `wd40`, `wd100` | 1e40, 1e100 | the 34-digit input centre, unchanged |
| `wd300`, `wdc0` | 1e30, 1e20 | `300.5/-2.25/1e+30`, `0.0/0.0/1e+20` |
| `wdmag` | 1.23456789123e20 | mag written `1.234568e+20` |
| `wdcorn` | (corners, height 3e−15) | `corners=-0.7436438870371587048/-0.743643887037151/0.1318259042053119705/0.131825904205315` |

- The centre is rounded to L + 5 digit positions, L = floor(log10 mag)
  (`wd13`: 18, `wd9e14`: 19, `wd15`/`wd3e15`: 20, `wdrot`: 25,
  `wd5e25`: 30), counted from the first integer digit when |v| ≥ 1
  (`-1.234…346`: 25 digits) and from the decimal point when |v| < 1
  (`1.2…e-5`: 35 decimal places, printed in exponent form). Trailing zeros
  are dropped, so short inputs come back unchanged (`wd40`, `wd300`).
  **Hypothesis** (the rule fits all 11 cases; image size made no
  difference, `wdfull` at 800×600 = `wdmag` digits).
- `corners=` values get the same treatment with L from the view height
  (`wdcorn`: 19 digits at mag ≈ 6.6e14).
- The magnification has **7 significant digits** (as par.md §5.2), so a
  mag like 1.23456789123e20 comes back as 1.234568e20: a slightly
  different zoom. Rotation and skew are printed with 18 digits.
- x3/y3 are not written for unrotated `corners` views. A non-deep view
  prints its y centre with a `+` sign (`wd12`).
- Round trip (`par/rt-*`, loaded on amd64 with `@name.par/name`): all
  eight files (`wd13`, `wd15`, `wd40`, `wdcorn`, `wdj20`, `wdrot`, `wdsm`,
  `wd5e25`) gave pixel-identical images and the same N; the corners moved by
  at most 0.055 pixel (`wdrot`), usually < 0.003 pixel.

**For FractalForge's PAR writer** (hypothesis-free advice): write the
centre with at least log10(mag) + 5 significant decimals (or as many as
the view needs), and the mag with enough digits to be exact — XFractint
reads long values; keep every token ≤ 140 characters if the file may also be
used as a command line (§6).

---

## 8. Pixels at deep zoom

### 8.1 Colour convention (confirmed)

The normal rule holds in deep mode: with z₀ = 0 (mandel, so z₁ = c) or the
pixel (julia), k = the index of the first iterate with |z_k|² > 4 (z₀ is
not tested), colour = max(k − 1, 1), inside (k − 1 ≥ maxiter, or no
escape) = 0. Pixel ↔ coordinate mapping as gif spec §2.2 with the corners
from slots 0–3.

Evidence (`pixels/`, all `maxiter=200 passes=1 periodicity=0`, centres at
i + small offset, where the Mandelbrot set has structure at every scale;
`sim-results.txt`):

| File | View | N | exact simulation (600 bits) | XFractint-like model (§8.2) |
|---|---|---|---|---|
| `px-m15` | mandel, mag 1e15 | 12 | 120000/120000 | 120000/120000 |
| `px-m15g` | same, default `passes=g` | 12 | 120000/120000 | 120000/120000 |
| `px-m20` | mandel, 1e20 | 12 | 119999 (1 pixel: |z₅₃|² = 3.99998) | 120000 |
| `px-j20` | julia c = i, 1e20 | 12 | 120000 | 120000 |
| `px-m40` | mandel, 1e40 | 20 | 118523 (98.77 %) | 119996 |

Also `base/base-v1`, `base-v2`, `base-j`, `base-j2` and `interrupt/ref-i1`,
`ref-i3` (julia, N = 16) were spot-checked with the exact simulation (rows
0, 75, 150, 225, 299: 0 mismatches).

### 8.2 XFractint's own precision (hypothesis, strongly supported)

The mismatches at 1e40 are XFractint's, not ours. A model with
**8(N − 1) fraction bits** (88 for N = 12, 152 for N = 20) in fixed point,
products truncated toward zero, and pixel coordinates built as
`corner + index × step` with the corner and the step
`(xmax − xmin)/(W − 1)` (and the y equivalents) **reduced to that
precision** (rounded down) reduces 1477 mismatches to 4
(`SIMVAR=tz sim.py … bits=152 step_coords=1`). The step truncation is the
main effect: at the last column the coordinate drifts by up to
(W − 1)·2^(−8(N−1)), about 0.1 pixel at mag 1e40 with N = 20. Exact
coordinates with 152-bit arithmetic still gave 21 of 4000 mismatches on
sample rows; truncated steps with exact arithmetic were not separated
further.

Consequence for FractalForge: a perturbation renderer with exact pixel
coordinates should match XFractint's deep pixels except near band edges
where XFractint's coarse steps matter (≈ 1 % at 1e40, ~0 at 1e15–1e20).
Compare by colour index with a distinct palette (types.md §1.3), and
expect that fraction of mismatches.

---

## 9. Interrupt and resume (confirmed)

- An interrupted deep render (`s` while calculating) saves status 2,
  `fractint002`, `fractint005` and `fractint001`, in that order
  (`interrupt/int-i1`: mandel 1e15 `passes=1`, item `[2,1,0,399,44,299,44,16,0,99]`;
  `int-i3`: julia 1e30, N = 16, `[2,1,0,399,131,299,131,48,0,174]`).
  `fractint002` is the normal work list (fractint-resume.md §1). Pixels
  before (`row`, `col`) equal the finished reference; the rest are 0 except
  pixel (`row`, `col`) itself in `int-i3`, which was already final.
- The julia save has symmetry code **48** (0x30), a value not listed in
  fractint-resume.md §2.1. Its view (centre i, mag 1e30) contains neither
  axis; the item resumed correctly, so 48 means no mirroring here
  (**hypothesis** for other cases).
- `int-i2` (guessing) finished within the 40 s, so no guessing work list
  was captured.
- Loading with the in-program `r` command resumes: `r-int-i1`, `r-int-i3`
  finish with status 4 and are pixel-identical to the batch references
  (`ref-i1`, `ref-i3`). With row 10 repainted colour 7 first
  (`int-i*-mark`) the marker row survives and every other row matches:
  a real resume, not a recalculation.
- `filename=` + Enter recalculates (fractint-resume.md §5.1); not
  repeated here.

A FractalForge writer of interrupted deep files: `fractint002` as in
fractint-resume.md §6 plus `fractint005` as in §5.3 (not load-tested in the
combination with crafted work lists).

---

## 10. Unknowns

- The measure behind switch condition (B) and the exact constant of (A)
  (§1.2).
- Whether y spacing alone influences N; the rule for 190×143 images
  (`viewwindows=no`, whose meaning is not understood either) (§2).
- Behaviour with slots not in normal form, or with N not a multiple of 4.
- Params 3–10 in slots 8–15 on load; deep views beyond ±32768.
- Exact arithmetic details behind the last 4 mismatches at 1e40 (§8.2).
- Whether XFractint ever writes [412] = 1 without `fractint005`.

---

## 11. Corrections and additions to gif-fractint-blocks.md

- §2.4 normalisation: the rule "1/256 ≤ |value|/256^(e+1) < 1" is wrong
  for negative powers of 256 in corner slots: −1 is stored with e = −1 and
  mantissa −1.0. Use "−1 ≤ value/256^(e+1) < 1, smallest e" (§3.2; one
  param slot followed the old rule).
- §2.4 "slots 16–21 = copy": true, and they are ignored on load (§5).
- §2.1 [414]: N is 4, 8, 12, … by the rule of §2; "12 (mag 1e13–1e20)" is
  12 up to 6.6e22 at 400×300 and depends on image size.
- §2.1 [474] "If it's 0, a load switches to deep-zoom mode": it is
  **[482]** that does this; [474] = 0 alone has no effect (`loaded/k17`,
  `k18`). The zero-template files had both zero.
- §2.4 "not at 1e12, at 1e13": the switch depends on the view and the
  image size (§1.2); 300.5 switches at 5e10.
- §5.5 deep-zoom writer: now load-tested; the f64 corners are optional,
  [414] is mandatory, params belong in slots 6–15 (§5.3).

---

## 12. Evidence index

| Topic | Files | Tools |
|---|---|---|
| types | `types/ty-*`, `ty30-*`, `tyv-*` | `tab.py` |
| switch | `switch/th-*`, `c0-*`, `c0f-*`, `c0x*`, `c300-*`, `csm-*`, `tol*`, `full-*` | `tab.py`, `meas.py` |
| N | `nsize/*`, `switch/full-*`, `switch/c300-*` | `nrule.py` |
| encoding | `encoding/enc-*`, `dig*`, `len-*`, `pl-*`, `complete/mandel-deep-*`, `julia-deep` | `dz.py`, `encchk.py`, `normchk.py`, `f64chk.py` |
| load tests | `base/*`, `crafted/k*`, `loaded/k*` | `craft.py`, `craft-all.sh`, `load-all.sh`, `cmp.py` |
| pixels | `pixels/px-*`, `pixels/sim-results.txt` | `sim.py` |
| resume | `interrupt/*` | `keys.sh`, `cases-intr.txt`, `cmp.py` |
| PAR | `par/wd*.out.par`, `par/wd*.gif`, `par/rt-*` | `cases-par*.txt`, `cases-e12.txt` |
