<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# IFS: iterated function systems (`type=ifs`, type number 26)

Status: **Phase 4 draft, derived from black-box analysis only.** Every rule
below comes from pixels and parameter blocks that XFractint (Debian
`xfractint 20.4.10-5`, amd64) saved for `.ifs` files we wrote ourselves, or
from the messages it printed. No Fractint, XFractint or Iterated Dynamics
source code, headers, help text, documentation or bundled `.ifs` files were
consulted (see [CLEANROOM.md](../CLEANROOM.md)). The pseudo-random generator
in §4 is a published C-library algorithm that we used as a hypothesis and
then confirmed against XFractint's pixels. Names are our own.

Labels: **Confirmed** = shown directly by the evidence, usually by
reproducing every pixel. **Hypothesis** = fits all evidence but other
explanations remain. **Unknown** = not determined.

Evidence is in `corpus/xfractint/ifs/`. A name like `leaf` means
`corpus/xfractint/ifs/leaf.gif`, and `cases.txt` there gives its command line.
The `.ifs` inputs are in `tools/blackbox/ifs/inputs/`.
`tools/blackbox/ifs/ifsmodel.py` is our own implementation of this spec. It
predicts every pixel of a case and counts the differences (§7). Block offsets
refer to [gif-fractint-blocks.md](gif-fractint-blocks.md) §2.1. Pixel colours
are compared through the palette as in [types.md](types.md) §1.3 (stored
index 16 shows up as 0, 31 as 15).

---

## 0. Summary

- An entry is `name {` … `}` in a text file. Its numbers are read as one free
  stream: 7 per transform for a 2D entry, 13 per transform when the name is
  followed by `(3D)`. Up to 63 transforms. Probabilities are not normalised
  (§1).
- The command line needs `type=ifs ifsfile=FILE ifs=ENTRY`. `maxiter` sets
  the number of points, 1024·maxiter + 1. Default corners are
  −8, 8, −1, 11 (§2).
- **2D**: 16.16 fixed-point chaos game from (0, 0). The transform is chosen
  with glibc `rand()` seeded with 1. Every point after the start is plotted
  by adding 1 (mod 16) to its pixel. A point whose |column| + |row| exceeds
  about 10,000 stops the calculation (§3, §4).
- **3D**: double-precision chaos game. The first 100 points only measure the
  attractor, which is then centred in the view. The rest are rotated by
  `rotation=` and plotted the same way (§6).
- With `params=1` (|param 1| ≥ 1), each pixel gets the number of the
  transform that last hit it instead of a hit count (§3.6).
- An interrupted image is saved with status 3 and no resume data. XFractint
  can't resume it, only recalculate it from the start (§5).
- `ifsmodel.py` reproduces every pixel of most cases. In the worst case
  54 of 120,000 pixels (0.4 % of the lit ones) differ (§7).

---

## 1. The `.ifs` file

### 1.1 Entries (confirmed)

- An entry starts on a line that begins (after optional blanks) with the
  entry name, followed by blanks or directly by `{`: `name {`, `name{` or
  `  name {`. The name is matched without regard to case (`fmt-mixed`,
  `fmt-lower`, `fmt-upper`, `hdr-nospace`, `hdr-indented`).
- **Everything after `{` on that line is ignored.** `c4 { 0 0 0 0 4 4 1`
  followed by `0 0 0 0 1 2 1` and `}` draws only the second transform
  (`fmt-firstline`). So a one-line entry `name { … }` fails: XFractint reads
  the next entry's lines as numbers and reports `invalid IFS definition`
  (`fmt-oneline`), or crashes when no numbers follow (cases.txt,
  `fmt-oneline-crash`).
- A `{` on the line after the name doesn't count. The entry isn't found
  (`hdr-nextline`).
- The entry ends at the first `}`. Numbers before the `}` on the same line
  are still read (`fmt-close`). An entry with no `}` before the end of the
  file is not found (`lim-last`). Text after `}` on the same line was not
  tested.
- With duplicate names, the first entry in the file is used (`fmt-dup`).
- `;` starts a comment that runs to the end of the line, both outside and
  inside entries and after numbers (`fmt-trail`; every input file begins
  with `;` lines).
- 3D: the name followed by `(3D)` or `(3d)`, with the parentheses, before the
  `{`. Any amount of blank space is allowed around it (`3d-mark1`, `3d-mark2`,
  `hdr-sp3d`). Without the parentheses (`mark3 3D {`) the entry is read as 2D
  (`3d-mark3`: 26 numbers, so `invalid IFS definition`). 13 numbers per line
  without the marker are also invalid (`3d-plain13`).

### 1.2 Numbers (confirmed)

- The numbers inside an entry form one stream. Line breaks don't matter: a
  transform split 5 + 2 over two lines works (`fmt-split`), and so do lines
  of 8 and 6 numbers (`fmt-p8x6`).
- Separators: spaces, tabs and commas (`fmt-tabs`).
- Accepted forms: `.5`, `+0.5`, `-0`, `1e0`, `5e-1`, `1.953125e-3`
  (`fmt-mixed`).
- The count must be a multiple of 7 (2D) or 13 (3D). Otherwise the message is
  `invalid IFS definition` and nothing is drawn: 12 numbers (`fmt-six`), 16
  (`fmt-eight`), 14 in a 3D entry (`3d-mark7`).
- 2D transform, in file order: `a b c d e f p`, with
  `x' = a·x + b·y + e` and `y' = c·x + d·y + f` (§3.2). `p` is the
  probability weight.
- 3D transform: `m0 … m8 tx ty tz p`, with
  `x' = m0·x + m1·y + m2·z + tx`, `y' = m3·x + m4·y + m5·z + ty` and
  `z' = m6·x + m7·y + m8·z + tz` (§6.1).

### 1.3 Limits (confirmed)

| Limit | Evidence |
|---|---|
| 63 transforms work, 64 give `IFS definition has too many lines` then `invalid IFS definition`. The same for 2D and 3D | `lim-d63`, `lim-d64`, `lim-g63`, `lim-g64` |
| An entry with no numbers: `Empty IFS definition` | `lim-empty` |
| `ifs=` on the command line takes at most 18 characters. With 19 the whole argument is rejected (`Oops. I couldn't understand the argument`) | `name-18`, `name-19` |
| `maxiter=1` is rejected the same way. 2 is the smallest value accepted | `const-mi1`, `const-mi2` |

### 1.4 Probabilities (confirmed)

The weights are **not normalised**. The selection (§4.3, §4.4) compares a
random number with the running sum of the weights, so:

- If the weights add up to more than 1, later transforms lose. With weights
  `1 1 … 1`, only the first transform is ever used (`prob-ones`). With
  `3, 1`, only the first (`prob-big`).
- If they add up to less than 1, the rest of the range goes to **the last
  transform**. `0.25, 0.25` uses transform 2 three times out of four
  (`prob-low`). `0, 0` always uses transform 2 (`prob-allzero`).
- A zero or negative weight just adds nothing (or subtracts) from the running
  sum. `0, 1` always uses transform 2. `-0.5, 1.5` does the same, because the
  running sum starts below every possible random value (`prob-zero`,
  `prob-neg`). In 2D a weight of 0 still wins when the random value is
  exactly 0 (§4.3, hypothesis: not seen).

### 1.5 Messages

Batch mode writes each message to `stopmsg.txt` (kept as
`NAME.stopmsg.txt`) and still saves an empty image (all pixels 0) with the
entry name in `fractint003`, except where noted.

| Message | Cause | Evidence |
|---|---|---|
| `invalid IFS definition` | number count not a multiple of 7 / 13, or the stream runs into text | `fmt-six`, `fmt-oneline`, `3d-plain13` |
| `IFS definition has too many lines` + `invalid IFS definition` | more than 63 transforms | `lim-d64`, `lim-g64` |
| `Empty IFS definition` | no numbers | `lim-empty` |
| `'NAME' file entry item not found` | no such entry, entry without `}`, `{` on the next line, or no file. With no `ifs=` the name is empty (`''`) | `sel-noentry`, `lim-last`, `hdr-nextline`, `sel-nofile`, `sel-fileonly`, `sel-none` |
| `Oops. I couldn't understand the argument: …` | unknown argument such as `ifsname=`, a name over 18 characters, `maxiter=1`. **No image is saved** | `sel-ifsname`, `name-19`, `const-mi1` |
| (none, crash) | one-line entry at the end of the file | `fmt-oneline-crash` |

---

## 2. Selecting an entry, parameters, saved blocks

### 2.1 Command line (confirmed)

- `type=ifs ifsfile=FILE ifs=ENTRY`. `FILE` is relative to the working
  directory. `ifsname=` is not an argument this build knows (`sel-ifsname`).
- Without `ifs=` the entry name is empty and not found (`sel-fileonly`).
- Without `ifsfile=` and with no `.ifs` file in the working directory,
  `fftri` was not found (`sel-nofile`). XFractint presumably looked in a
  default file of its own, which we did not open.
- With `ifsfile=missing.ifs` and only `zz.ifs` (which contains `fftri`) in
  the directory, XFractint drew `fftri` (`sel-missingfile`). **Hypothesis:**
  when the named file doesn't exist, it searches other `.ifs` files in the
  directory. Not explored further.

### 2.2 Parameters

| Parameter | Effect | Status |
|---|---|---|
| `maxiter` | number of points, 1024·maxiter + 1 (§3.3) | confirmed |
| `corners` | the view (§3.4). Default −8 / 8 / −1 / 11 for 2D and 3D | confirmed |
| `params` | param 1: colour mode (§3.6). Params 2–10 change nothing (`bytr-p2only`) | confirmed |
| `rotation`, `xyshift` | 3D only (§6.3, §6.4). No effect on 2D (`noeffect-3dparams`) | confirmed |
| `perspective`, `stereo` | 3D only, change the image, not modelled (§6.6) | unknown |
| `scalexyz`, `sphere` | no effect on 3D (`3d-scale50-*`, `3d-sphere-*`) | confirmed |
| `float=yes` | no effect (`march-1e-4-float`, `march-2e-5-float`) | confirmed |
| `rseed` | no effect on pixels, still stored at 98–100 (`rng-seed-*`) | confirmed |
| `inside`, `outside`, `symmetry`, `passes` | no effect (`noeffect-inout`, `noeffect-sym`) | confirmed |

### 2.3 Saved blocks (confirmed)

Every IFS save has `fractint003` and `fractint001`, in that order, after the
image. Extension length 001[204] = 603 (521 + 82).

- `fractint003`: entry name, NUL-padded in the first 48 bytes. The file name
  is not stored. [48] = 0 for both 2D and 3D. The name is stored even when it
  wasn't found (`sel-noentry`) and is empty without `ifs=` (`sel-none`).
- `fractint001`: type 26 at [10], corners at 12–43 (184–199 equal to xmin,
  ymin), maxiter at 8 and 408, params at 44…, the seed at 98 and the pass
  mode letter (default `g`, unused) as for other types. 3D viewing values
  are stored where every type stores them: rotation at 128/130/132,
  perspective at 144, xyshift at 146/148 (`3d-rot45_20_70-a100`,
  `3d-persp150-a400`, `3d-shift-m5-10`).
- **001[252] = −1 (`ff ff`) for a 3D entry that was drawn**, and 0 for every
  2D save and for 3D entries that failed to load. Seen in all 68 valid 3D
  saves against 214 others. Whether XFractint reads it back is unknown.
- 001[202] status: 4 = finished (also when the far-point rule stopped the
  calculation early, §3.5), 3 = interrupted (§5), −1 = re-saved by a batch
  load.

---

## 3. The 2D algorithm

### 3.1 Numbers to fixed point (confirmed)

Each number `v` of a 2D entry becomes the integer
`V = trunc(float32(v) · 65536)`, rounding toward zero (16.16 fixed point):

- Truncation, not rounding: `e = 0.0001` steps by 6/65536 (`march-1e-4`:
  every pixel is hit 438 = 2628 / 6 times, colour 6). `0.001` → 65
  (`march-1e-3`), `0.00002` → 1 (`march-2e-5`).
- Toward zero: `e = −0.0001` steps by −6 (`num-negstep`, colour 6 again).
- Single precision first: `a = 0.99999999` behaves as exactly 1.0, because
  the line marches. In double precision it would be 65535, and the point
  would stay put (`num-nearone`). **Hypothesis:** the weights `p` are
  converted the same way. No case tells float and double apart for them.

### 3.2 Iteration (confirmed)

The point starts at `(X, Y) = (0, 0)`. Each step chooses transform `k` (§4.3)
and computes

    X' = T(a·X) + T(b·Y) + E
    Y' = T(c·X) + T(d·Y) + F

`T(q) = q / 65536`, truncated toward zero, applied to each 64-bit product
separately. Evidence:

- Coefficient order: `asym` = `0.5 0.25 0 0.5 1 2 1` converges to (4, 4),
  which needs `x' = 0.5x + 0.25y + 1` and `y' = 0.5y + 2`. The other order
  would converge to (2, 5) (`coef-asym`).
- The fractional bits are dropped: `x' = 0.5x + 1/65536` stays at 1/65536
  instead of converging to 2/65536 (`prec-half`, `prec-3q`).
  `x' = 0.5x + 3/65536` visits 3, 4, 5 (`prec-half3`).
- Toward zero for negative products: `x' = 0.5x − 3/65536` visits −3, −4, −5
  and stays at −5. Rounding down would reach −6 (`num-negprod`).
- Separately per product: with real mixing coefficients (`leaf`,
  `leaf-wide`, `leaf-zoom`) per-product truncation gives 16 / 54 / 10
  mismatching pixels. Truncating the sum of both products gives
  207 / 1086 / 346, and rounding down gives 365 / 1480 / 454.

Overflow behaviour (|x| ≥ 32768) is unknown.

### 3.3 Number of points (confirmed)

Exactly `N = 1024 · maxiter + 1` steps. **Every point after the start is
plotted, beginning with the first step's result.** The start (0, 0) itself is
not plotted, and nothing is discarded.

- `count-start`: the march `x' = x + 1/65536` lights the pixel of 1/65536
  but not that of 0.
- The march's end gives N: 2049 for maxiter 2 (`count-mi2a`, `count-mi2b`),
  3073 for 3 (`count-mi3`), about 153,600 for 150 (`march-2e-5`).
- A constant map (`0 0 0 0 1 2 1`) hits one pixel N times. Its colour is 1
  (N ≡ 1 mod 16) for maxiter 2, 17 and 150 (`const-*`).
- Run time grows with maxiter (`march-2e-5-mi20000`: 63 s).

### 3.4 Point to pixel

With `x = X/65536`, `y = Y/65536` and an image of `W × H` pixels:

    col = floor((x − xmin) · (W − 1) / (xmax − xmin))
    row = floor((ymax − y) · (H − 1) / (ymax − ymin))

A point is plotted only if `0 ≤ col < W` and `0 ≤ row < H`. Other points are
skipped, and the iteration goes on (but see §3.5).

- Confirmed: the recorder cases (`rng-d*`) have one point per row and match
  this formula in all 599 rows. Rounding to nearest gets about half of them
  wrong. A point just left of xmin (column value in (−1, 0)) is **not**
  plotted in column 0 (`line-def-unitneg`). That differs from 3D (§6.5).
- Not exact: XFractint's own calculation differs very slightly from this
  formula in double precision. In the long lines `line-*` (one point per
  1/65536), a point that lies within about 10⁻⁴ pixel of a column or row
  boundary sometimes lands in the neighbouring pixel. That happened at 2 of
  the 200 column boundaries crossed in `line-def-unit`, and at about 40 of
  the 275 row boundaries of `line-def-unity` (y = 0…11). The deviation is
  not a constant offset or scale. No combination of float32 rounding,
  16.16 corners and 16.16 scale factors that we tried (analysis scripts not
  kept) explains it. **Unknown.** In practice
  0–0.05 % of the lit pixels differ (§7). The third corner (rotated or
  skewed views) was not tested.

### 3.5 Far points stop the calculation (confirmed rule, hypothesis threshold)

If a point's pixel position `(col, row)` (§3.4, before the range check) has
`|col| + |row| > 10000`, **XFractint stops the whole calculation** and saves
what it has as finished (status 4). `col` and `row` are the indices of §3.4,
so a point right of or below the image counts its full index, not its
distance from the edge (`far-right*`).

| Case pair (march toward the window) | Index sum of the first point | Result |
|---|---|---|
| `far-x0740` / `far-x0741` (row 149) | 9,989 / 10,003 | drawn / empty |
| `far-top0750` / `far-top0752` (row 1) | 9,974 / 10,001 | drawn / empty |
| `far-bot0728` / `far-bot0745` (row 297) | 9,978 / 10,204 | drawn / empty |
| `far-diag037` / `far-diag0385` (above left) | 9,884 / 10,284 | drawn / empty |
| `far-right0735` / `far-right0745` (point right of the image) | 9,922 / 10,055 | drawn / empty |
| `far-800-363` / `far-800-365` (800×600, row 299) | 9,967 / 10,020 | drawn / empty |

So the threshold lies between 9,989 (drawn) and 10,001 (stopped). It
doesn't depend on the image size or on where the window is. We assume
`> 10000`. A sum of exactly 10,000 wasn't tested, so `>` vs `≥` is unknown.
A practical effect is that a deep zoom
into an attractor comes out empty or almost empty, because the chaos game
soon throws a point far outside: `fftri-z100` (a 100× zoom of the triangle)
is empty, while `fftri-z10` is complete.

### 3.6 Colour (confirmed)

The image starts at colour 0. Then, per plotted point:

- **param 1 truncated to an integer is 0** (default, `0.5`, `0.99`): the
  pixel's value becomes `(value + 1) mod 16`. A pixel shows its hit count
  mod 16, and 16 hits look like background (`march-1e-3`: 40 and 41 hits →
  8 and 9; `march-2e-5`: 2628 hits → 4; `fftri`, `bytr-p05`, `bytr-p099`).
- **|param 1| ≥ 1** (`1`, `2`, `−1`): the pixel's value becomes `k + 1`,
  where `k` is the 0-based index of the transform that produced the point.
  The last point to land wins (`bytr-p1`, `bytr-p2`, `bytr-pm1`,
  `bytr-leaf`). This works up to 63 transforms, with no wrap at 16
  (`bytr-d63`: values 1…63, apart from the saver's palette merging).
- Whether "integer part ≠ 0" or "|param 1| ≥ 1" is the real test can't be
  told apart from these values (hypothesis).

### 3.7 Pseudo-code (2D)

    V(v)     = trunc(float32(v) * 65536)                 # toward zero
    T(q)     = q / 65536 truncated toward zero           # 64-bit q
    C[i]     = V(p0) + ... + V(pi)                       # running sums
    reset rand() to seed 1; image = 0
    X = Y = 0
    repeat 1024*maxiter + 1 times:
        r = rand() mod 32768
        k = first i with 2*r <= C[i], else the last transform
        X, Y = T(a X) + T(b Y) + E,  T(c X) + T(d Y) + F    # transform k
        col = floor((X/65536 - xmin)*(W-1)/(xmax-xmin))
        row = floor((ymax - Y/65536)*(H-1)/(ymax-ymin))
        if |col| + |row| > 10000: stop
        if inside the image: pixel = (pixel + 1) mod 16, or k + 1 (§3.6)

---

## 4. Random numbers (confirmed)

### 4.1 Generator

The choices follow glibc's `rand()` exactly. It is the published additive
feedback generator, written here in our own words:

1. `s[0] = seed` (seed 0 counts as 1). For `i = 1…30`:
   `s[i] = (16807 · s[i−1]) mod 2147483647`, computed exactly and kept
   positive.
2. `s[31…33] = s[0…2]`.
3. For `i ≥ 34`: `s[i] = (s[i−31] + s[i−3]) mod 2³²`.
4. Output number `n` (n = 0, 1, …) is `s[344 + n] >> 1`, which lies in
   0 … 2³¹ − 1.

With seed 1 the first outputs are 1804289383, 846930886, 1681692777,
1714636915, 1957747793, and their low 15 bits are 17767, 9158, 6249, 18547,
23633.

### 4.2 Seeding

- **Every IFS calculation starts the generator as if seeded with 1**, and
  uses output 0 for the first point. That holds in batch mode, for the first
  interactive render, and for a second calculation in the same session
  (`second-run`, which follows a finished render of 102,401 points, is again
  a prefix of the seed-1 sequence).
- `rseed=` and the time-based seed that XFractint stores at 001[98] don't
  matter. `rng-seed-1234a`, `-1234b`, `-1235`, `-0` and `-none` have
  identical pixels.
- A pass that consumes random numbers before the first point (3D, §6.2) uses
  the same stream.

### 4.3 Choosing a 2D transform

`r = rand() mod 32768` (the low 15 bits). With `C[i]` the running sums of
`trunc(float32(p) · 65536)` (§3.1), the transform is the first `i` with
`2·r ≤ C[i]`. If there is none, it's the last transform.

- Recorders: in `rng-d2` … `rng-d63` (K = 2…63 transforms
  `x' = x/K + i/K`, `y' = y + 1/512`, weights `1/K` written with 8 decimals),
  each row of the 800×600 image holds one point and its column shows the
  transform chosen. All 599 choices match in all 7 files, and in K = 4, 16,
  40 and 48 (renders not kept).
- The fixed-point sums and `≤` matter at the edges. With K = 33, weight
  0.03030303 becomes 1985, so `C[18] = 37715`, and `r = 18863` picks
  transform 19 (a real-valued `r·33/32768 = 18.997` would pick 18). With
  K = 32, `r = 15360` gives `2r = C[14] = 30720` and picks 14, so the
  comparison is `≤`. Scaling the weights by 1000, 10000, 32767, 32768 or
  65535, or rounding them, gave 2–1276 mismatches. 65536 with truncation and
  `≤` gave 0.
- How we found it: the K = 63 choices matched `(rand() & 0x7fff)·63 / 32768`
  from glibc seed 1 at offset 0. Searches over bit positions, moduli, three
  LCGs (seeds 0–65535) and strides found nothing else.

### 4.4 Choosing a 3D transform

`r = rand()` (all 31 bits). The transform is the first `i` with
`r ≤ (p0 + … + pi) · 2³¹` in double precision, else the last. Confirmed with
3D recorders (`3d-e3`, `3d-e10`, `3d-e33`, `3d-rec-*`) and the fallback case
`3d-lowp`. With these files, `≤` vs `<` and 2³¹ vs 2³¹ − 1 can't be told
apart (hypothesis). The first 3D match was found as bit 30 of `rand()` at
offset 100 (§6.2).

---

## 5. Interrupting, saving and reloading (confirmed)

- Pressing `s` during a calculation saves the image so far with **status 3**
  at 001[202] and **no `fractint002` block**. Everything else is as in a
  finished save, including the time so far at 214 (`int-20`, `int-35`).
- The saved pixels are exactly the first `n` points of the uninterrupted
  calculation: n = 6,571,499 (`int-20`) and 11,953,549 (`int-35`). We found
  n by replaying the model point by point until it matched (4 pixels differ
  through §3.4). n isn't a multiple of 1024, so XFractint checks for a key
  at least that often.
- If the calculation finishes before the key press, the save is a normal
  finished one (`int-done`, status 4).
- Reloading (input `in20.gif` = `int-20.gif`):

| How | Result | Evidence |
|---|---|---|
| batch `filename=` | re-saved unchanged with status −1 | `load-batch` |
| `filename=` on the command line, then Enter | **recalculated from the start**: a fresh prefix (n = 1,431,050) with the old pixels gone | `load-cmdline-enter` |
| in-program `r` command | image shown, nothing calculated, re-saved with status 3 and the same time | `load-r` |
| `r`, then Enter | recalculated from the start (n = 1,165,475) | `load-r-enter` |

So IFS images can't be resumed. A writer should save an unfinished
FractalForge IFS render with status 3 and no `fractint002`, or as finished.
Status 3 was only seen for IFS. How other types treat it is unknown.

---

## 6. 3D entries

3D entries (§1.1) take a different path. All of the following was confirmed
by reproducing every pixel (§7), except where marked.

### 6.1 Arithmetic

Double-precision floating point, starting from (0, 0, 0), with
`(x, y, z)' = M·(x, y, z) + t` (layout in §1.2). Single-precision
coefficients give the same pixels in our cases, so the parse precision is
unknown. Evidence for the layout: translation-only probes `3d-probe-w9`,
`-w10`, `-w11` move the point along x, y and z, while probes in positions
0–8 leave it at the origin, so both transforms land on one pixel
(`3d-probe-w0`, `-w5`). The digit recorders (`3d-e*`), which use `m0`, `m4`,
`tx` and `ty`, match.

### 6.2 Measuring pass and centring

The total is again `N = 1024·maxiter + 1` points:

- **Points 1–100 are not plotted.** They (with random outputs 0–99) are only
  used to measure the view. The plotted points are 101…N, and the chaos game
  carries on from point 100 without restarting (`3d-rec-start`: the first
  plotted y is 101/512; `3d-rec-end`, `-end-mi3`: the last is 2049/512 and
  3073/512).
- Each point is rotated to screen coordinates `(u, v)` (§6.3). The centre is
  `cu = (min u + max u)/2` and `cv = (min v + max v)/2` over points 1–100,
  the centre of their bounding box in screen space. Plotting uses
  `(u − cu, v − cv)` (`3d-centre-w3`: the box centre, not the weighted mean;
  `3d-centre-t3c`: the box of the rotated points, not the rotated 3D box).
- A one-point attractor therefore sits at screen (0, 0) whatever its
  coordinates (`3d-single`).

### 6.3 Rotation

With `rotation=a/b/c` in degrees (default 60/30/0):

    Rx(a) = [1 0 0; 0 cos a  sin a; 0 −sin a  cos a]
    Ry(b) = [cos b  0  −sin b; 0 1 0; sin b  0  cos b]
    Rz(c) = [cos c  sin c  0; −sin c  cos c  0; 0 0 1]
    R = Rz(c) · Ry(b) · Rx(a)       (x rotation applied first)
    u = R[0]·(x, y, z),  v = R[1]·(x, y, z)

`v` points up on the screen. The depth `R[2]` isn't used without
perspective. Evidence: unit-vector pairs for single-axis rotations of
30° and 90° (`3d-rot*`). The order x, y, z fits the four mixed rotations with
a total squared error of 0.015 (pixel rounding). The other five orders give
0.33 to 2.1. 0/0/0 gives `u = x`, `v = y` (`3d-pair-r000-*`).

### 6.4 `xyshift`

`xyshift=sx/sy` moves the picture by sx % of the view width to the right and
sy % of the view height up. It's applied as
`cu −= sx/100 · (xmax − xmin)`, `cv −= sy/100 · (ymax − ymin)`
(`3d-shift3_4-*`: +12 px, 12 px up in 400×300; `3d-shift-m5-10`: −20 px,
30 px up). Whether XFractint shifts in coordinates or whole pixels can't be
told apart here (hypothesis).

### 6.5 Pixel, colour and stop rule

    col = trunc((u − cu − xmin) · (W − 1) / (xmax − xmin))
    row = trunc((ymax − (v − cv)) · (H − 1) / (ymax − ymin))

**Truncation toward zero**, unlike 2D. A value in (−1, 0) lands in column or
row 0 (`3d-e10-mi150`: with `floor`, 18 pixels of row 0 come out wrong; with
truncation, none).
Colouring is as in §3.6: hit count mod 16, or transform number with param 1
(`3d-bytr`). A far point stops the calculation (`3d-far` is empty where
plotting would light 23 pixels). We assume the 2D rule `|col| + |row| > 10000`
(hypothesis, threshold not measured).

### 6.6 Not modelled

- `perspective=150` changes the image (`3d-persp150-a400`: one point stays,
  the other moves 87 px). Unknown.
- `stereo=1` draws both points of `a400` in colour 85, and `a111` comes out
  empty (`3d-stereo1-*`). Unknown.
- Default 0 for both was used everywhere else.
- Interrupting a 3D render and 3D far-point details weren't tested.

---

## 7. Verification

`tools/blackbox/ifs/ifsmodel.py` implements §1.1–§6.5 independently of the
analysis scripts. Pixels that differ after merging palette duplicates:

| Case | Lit pixels | Differ |
|---|---|---|
| `corpus/xfractint/complete/ifs-fftri` (800×600) | 751 | 0 |
| `fftri`, `fftri-unit`, `fftri-z10`, `fftri-z100` | 253, 15877, 3943, 0 | 2, 0, 0, 0 |
| `leaf`, `leaf-wide` (maxiter 400), `leaf-zoom` (300), `leaf-800` | 5479, 14589, 11653, 43797 | 16, 54, 10, 18 |
| `bytr-leaf`, `bytr-p1`, `bytr-pm1` | 15001, 270, 270 | 1, 0, 0 |
| `coef-asym`, `march-*`, `count-*`, `const-*`, `prec-*`, `num-*` | 1–201 | 0, except `march-2e-5-mi1000` and `-mi20000`: 3 |
| `line-*` (one point per 1/65536 for 1.1 M points) | 26–275 | 2–80 (§3.4) |
| `fmt-*`, `hdr-*`, `lim-*`, `name-18`, `sel-params`, `noeffect-*`, `bytr-*` | 0–15001 | 0–2 |
| `rng-d2…d63`, `prob-*` (800×600 recorders) | 599 each | 0 |
| `far-*` (all 12) | 0 or 148–800 | 0 |
| `3d-mark1`, `3d-basis*`, `3d-probe-*`, `3d-single`, `3d-pair-*`, `3d-rot*`, `3d-centre-*`, `3d-shift*` | 1–904 | 0, except `3d-pair-def-a040`: 2 (a point exactly on a row boundary, value 299.000…) |
| `3d-rec-start`, `3d-rec-end`, `3d-e3/e10/e33`, `3d-e10-mi150`, `3d-lowp`, `3d-bytr`, `3d-far`, `lim-g63` | up to 3876 | 0 |
| `3d-spiral`, `3d-spiral-rot`, `3d-spiral-shift` | 1225, 12132, 6899 | 2, 8, 0 |

In all, 163 of the 187 batch cases match exactly, including the error cases,
which stay empty. The remaining differences are 2D boundary cases (§3.4) and
a few 3D pixels on boundaries or with mixed rotations (unknown cause; float32
variants didn't remove them). The command to rerun one case is in
`tools/blackbox/ifs/README.md`.
Interrupted saves and recalculations (§5) are exact prefixes of the model's
point sequence, apart from 4 boundary pixels each.

A FractalForge test can:

1. render `ifs-fftri` (800×600) or `fftri-unit` and require identical
   pixels;
2. render `leaf`-type cases and allow ≤ 0.5 % of lit pixels to differ;
3. check the recorders `rng-d*`, `prob-*`, `3d-e*` exactly, since they test
   the generator, the selection rule and the point count in isolation.

---

## 8. Unknowns

- The exact rounding of XFractint's 2D point-to-pixel conversion (§3.4) and
  the third-corner (skewed) case.
- Precision of the probability parse (float32 assumed), `≤` / `<` and the
  scale (2³¹ or 2³¹ − 1) in 3D, and whether the colour-mode test is
  "integer part ≠ 0" or "|p1| ≥ 1".
- `>` or `≥` 10000 in the stop rule, and the rule's exact form in 3D.
- `perspective`, `stereo` and other 3D options; 3D interrupt behaviour.
- Overflow of 16.16 coordinates (|x| ≥ 32768).
- What makes `ifsfile=missing.ifs` still find an entry (§2.1).
- Whether XFractint reads 001[252] or status 3 on load in other contexts.

## 9. Evidence index

| Files | Section |
|---|---|
| `fmt-*`, `hdr-*`, `lim-*`, `name-*` | §1 |
| `sel-*` | §2.1 |
| `noeffect-*`, `bytr-p2only`, `march-*-float`, `rng-seed-*` | §2.2 |
| `coef-asym`, `march-*`, `count-*`, `const-*`, `prec-*`, `num-*` | §3.1–3.3 |
| `line-def-*`, `line-odd-*` | §3.4 |
| `far-*`, `fftri-z*` | §3.5 |
| `bytr-*` | §3.6 |
| `rng-*`, `prob-*` | §1.4, §4 |
| `int-*`, `load-*`, `second-run` | §4.2, §5 |
| `3d-*` | §6 |
| `fftri*`, `leaf*`, `3d-spiral*` | §7 |
| `par-*` (IFS), `lpar-*` (L-system, [lsystem-par.md](lsystem-par.md)) | §10 |

---

## 10. IFS images in `.par` files

Tested in batch mode with `@ip.par/ENTRY` (loading as in [par.md](par.md)
§2.7), in a directory that held only the named files. Inputs:
`tools/blackbox/ifs/par/` (`ip.par` holds the image entries and the
embedded bodies). Results are compared pixel for pixel with renders of the
same bodies from ordinary files (`par-ref-*`). The L-system results are in
[lsystem-par.md](lsystem-par.md).

### 10.1 Selecting the entry (confirmed)

- The keys are the command-line ones, `ifsfile=` and `ifs=`, inside the
  entry. `ifsname=` is not a key (§2.1).
- **`ifsfile=` inside an entry is not looked up in the working directory**,
  whether relative or absolute: `ifsfile=pf.ifs ifs=pf` (`par-arel`) and
  `ifsfile=/work/pf.ifs` (`par-aabs`) both give `'pf' file entry item not
  found`, although `pf.ifs` is right there. An entry with no `ifsfile=` fails
  the same way (`par-anone`). This is the behaviour par.md §5.3 found for
  `formulafile=`. **Hypothesis:** in entries, names are looked up only in
  XFractint's own directories (not tested; we didn't touch them).
- `ifsfile=` given on the command line **before** `@file/entry` works
  (`par-anone-cmd`, `par-rt-wifs-cmd`).

### 10.2 Embedded IFS entries (confirmed)

- An entry named `ifs:NAME` in the same `.par` file is used when an entry
  of that file says `ifs=NAME`, without `ifsfile=` (`par-enoname`,
  `par-enoname-only`) or with a (then irrelevant) `ifsfile=` (`par-eself`).
  It is pixel-identical to the same body read from an `.ifs` file
  (`par-ref-eq`). The body syntax is that of an `.ifs` entry (§1), with
  the numbers starting on the line after `{`. `(3D)` works there too:
  `ifs:pq3 (3D) {` (`par-e3d` = `par-ref-eq3`).
- The prefix is case-insensitive: `IFS:pu` (`par-eupper`). The embedded
  entry may come after the image entry (every case here).
- `ifs=ifs:pq` in the entry does **not** find it (`par-eprefix`), and a plain
  entry `pr` of the `.par` with an IFS body is not used (`par-pplain`),
  even with `ifsfile=ip.par` in the entry (`par-eselfplain`).
- An `ifsfile=` on the command line wins over the embedded entry:
  `ifsfile=pf.ifs @ip.par/enoname` draws `pf.ifs`'s `pq`
  (`par-enoname-cmd` = `par-ref-pfq`).
- `fractint003` stores the name without the prefix (`pq`).
- A `.par` given as `ifsfile=` on the command line is read as an `.ifs`
  file, where the entry names include the prefix: `ifs=ifs:pq` works
  (`par-cf-pfx`, stored as `ifs:pq`), `ifs=pq` doesn't (`par-cf-nopfx`),
  and the plain entry `pr` works (`par-cf-plain`).

### 10.3 What XFractint writes (confirmed)

From the `b` ("save parameters") screen in the native-architecture build of
the same package (`par-w-*.out.par`). Its pixels equal the amd64 build's:
`par-w-ifs.gif` = `par-ref-pf.gif`.

```
wifs               { ; our comment one
  reset=2004 type=ifs ifsfile=pf.ifs ifs=pf corners=0/1/0/1 params=0
  float=y inside=0 viewwindows=2/0.75/yes/0/0
  }
```

- Order: `reset`, `type=ifs`, `ifsfile=`, `ifs=`, the view, `params=`
  (always written, `params=0` by default, `params=1` with colour by
  transform, `par-w-ifsp1`), `float=y`, `inside=0`, 3D values, `viewwindows`.
- **The IFS body is never embedded.** For an image made from an embedded
  entry, XFractint writes `ifsfile=ip.par ifs=pq`, the `.par` file it came
  from and the name without prefix (`par-w-ifsemb`).
- 3D entries add `rotation=10/20/30 perspective=0 xyshift=3/4` after
  `inside=0`. The view came out as `center-mag=0/5/0.1666667` for the default
  corners (`par-w-ifs3d`).
- Such a file does not load back by itself: `@wifs.par/wifs` with `pf.ifs`
  next to it reports `'pf' file entry item not found` (`par-rt-wifs`,
  §10.1). With `ifsfile=pf.ifs` on the command line it reproduces the image
  (`par-rt-wifs-cmd` = `par-ref-pf`).

### 10.4 Recommendation for FractalForge

Write `ifsfile=` and `ifs=` as XFractint does, and also copy the entry
into the same `.par` as `ifs:NAME {` + newline + the numbers + `}`. That
makes the file self-contained for XFractint (§10.2) and for us. When
reading, look for `ifs:NAME` in the same `.par` first, then the `ifsfile=`
next to the `.par`, then FractalForge's own directories.
