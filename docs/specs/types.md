<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Escape-time fractal types: iteration, colouring and symmetry

Status: **black-box derived.** Everything here comes from pixels and
parameter blocks that XFractint (Debian `xfractint 20.4.10-5`, amd64) saved,
compared against our own calculations. We used published mathematics (the
standard formulas for the lambda map, Newton's method, the phoenix and magnet
families, and so on) only as hypotheses. No Fractint, XFractint or Iterated
Dynamics source code, headers, help text or documentation was consulted. See
[CLEANROOM.md](../CLEANROOM.md). Names and notation are our own.

Evidence is in `corpus/xfractint/types/`, and `cases.txt` there gives every
command line. Every file is a 400×300 batch render made with
`passes=1 periodicity=0 viewwindows=2`. A file name like `lambda-def` below
means `corpus/xfractint/types/lambda-def.gif`. Block offsets refer to
[gif-fractint-blocks.md](gif-fractint-blocks.md) (`fractint001`, §2.1).

**Confirmed** means our calculation reproduces XFractint's pixels for whole
images (≥ 99.9 % of the 120,000 pixels, usually 100 %), or a one-parameter
diff shows the effect directly. **Hypothesis** means it fits the evidence but
other explanations remain.

---

## 1. Common model

### 1.1 Notation

- Pixel (column `c`, row `r`) maps to the point `w = x + iy` exactly as in
  gif-fractint-blocks §2.2: `x = xmin + c·(xmax−xmin)/(W−1)`,
  `y = ymax − r·(ymax−ymin)/(H−1)`. Row 0 is the top.
- `P1…P4` are the type parameters (`params=`), stored as f64 at 44, 52, 270
  and 278. `a = P1 + i·P2` is the complex parameter.
- `|z|²` means `Re(z)² + Im(z)²`. `M` = maxiter (i32 at 408), `B` = bailout.
- The orbit is `z0, z1, z2, …`, with `z(k+1) = F(z(k))` (some types also use
  `z(k−1)`).

### 1.2 Iteration count and colour index (confirmed for the 13 escape-time types)

For every escape-time type in this document (all except newton and
newtbasin):

1. **The start value `z0` is never tested.**
2. `e` is the index of the first iterate `z(e)`, `e ≥ 1`, that passes the
   type's stop test. That is normally `|z(e)|² > B`, and magnet also has a
   convergence test.
3. If no such `e < M` exists (that includes `e = M` exactly), the pixel is
   **inside** and gets colour 0 (default `inside`).
4. Otherwise the colour index is `e` when `e ≤ 255`, else `((e − 1) mod 255) + 1`.

In other words, the colour is the number of the iteration that escaped. Points
that escape on the first iteration get colour 1. Evidence:

- `e`, not `e − 1`, fits every `-def` file. The alternative `e − 1` matched
  only 14–65 % of pixels (e.g. `lambda-def` 50.3 %, `mandellambda-def`
  14.6 %, `julia4-def` 64.5 %).
- `e = M` drawn as inside: in `lambda-def`, `mandellambda-def` and
  `julia4-def` the only mismatches left after the first fit were pixels with
  `e = 150 = M`, and all of them were colour 0.
- Wrap above 255: in `lambda-mi300` (maxiter 300), e = 256 → 1, 257 → 2,
  268 → 13, 269 → 14, 278 → 23, 286 → 31 (shown as 15, see §1.3). That's only
  12 pixels, so the wrap rule is a **hypothesis** with good support.

Cross-check with the reference types (not part of this spec): `mandel` fits
the same rule if its orbit starts at `z0 = c`. Starting from `z0 = 0` you get
colour `e − 1` with a minimum of 1, which is the same thing, and
gif-fractint-blocks §9 describes it that way. `julia` is the exception we
found. Its colour is `e − 1` (minimum 1) with `z0` = pixel. That gave 100 %
on an uncommitted julia default render (`type=julia` with the flags above)
and 99.995 % on `corpus/xfractint/complete/verify-ref-r2`.
**None of the 15 types below behaves like julia.** All of them use `e`.

newton and newtbasin count differently (§3.12).

### 1.3 Duplicate palette colours (observed; affects comparisons)

The default palette has entries with equal RGB: 16 = 0 (black), 31 = 15
(white) and 248–255 = 0 (black). XFractint's saved GIFs **never contain the
higher index of such a pair**. A pixel whose colour should be 16 is stored as
0, 31 as 15, and 248–255 as 0. Example: in `lambda-def`, every pixel with
e = 16 is stored as 0 and every pixel with e = 31 as 15. Our hypothesis is that
the saver reads pixels back from a true-colour display and maps each RGB to
the first matching index. That is a property of the saver, not of the types.

All match rates in this document compare `canon(predicted)` with the stored
index, where `canon(i)` is the lowest palette index with the same RGB as `i`,
taken from the file's own global colour table. A FractalForge pixel test
should do the same, or test against iteration counts before colouring.

### 1.4 Bailout, maxiter and float

- Default maxiter is **150** for every type here: `-def` files store 150 at 8
  and 408 when no `maxiter=` is given.
- Default bailout is stored as 0 (both 212 and 402), meaning "type default".
  The effective default is **B = 4** for all escape types except magnet1m and
  magnet1j, where it is **B = 100**. newton and newtbasin have no escape test,
  and `bailout=` changes nothing for them (`newton-bo100` gives the same pixels
  as the model without bailout).
- `bailout=N` replaces the default. `lambda-bo100`, `manowar-bo16`,
  `magnet1m-bo10` and `magnet1j-bo1000` are all reproduced with `B = N`
  (100 %, 100 %, 99.995 %, 100 %).
- `>` versus `≥` in `|z|² > B` could not be told apart (no pixel landed on the
  boundary).
- `float=yes` gives pixel-identical images for all 15 types (renders compared
  during analysis, not committed). The default integer/float choice therefore
  doesn't matter for these types in this build. All our computations use IEEE
  double.

### 1.5 Parameters and defaults

Unless a type section says otherwise:

- For "Mandelbrot-like" types (the pixel is the varying constant), `a = P1 +
  iP2` is **added to the start value** `z0`. Default 0.
- For "Julia-like" types (the pixel is `z0`), `a` is **the constant**.
- `P3` and `P4` are ignored unless the type section says otherwise. Checked:
  `lambda params=…/0.5/0.5`, `mandel4 params=0/0/0.5/0.5`, phoenix and
  mandphoenix `P4 = 1`, newton `P3 = P4 = 1`. All of them reproduce with the
  extra values ignored.
- `params=` values are stored exactly as given, and the stored values are the
  ones used.

### 1.6 Symmetry (confirmed; it does change pixels)

With the default `symmetry` (999 at offset 124), XFractint **does not compute
every pixel** for some types. It copies mirrored pixels instead. When the
view is mirror-symmetric about the axis (all default views are), the copies
equal what direct calculation would give, so the pixels are identical with
`symmetry=none`. When the axis lies **off-centre** inside the view, the
mirrored row is not at exactly `−y`, and copying gives different pixels.
Examples: `mandel4-off` differs from `mandel4-offnosym` in 1,501 pixels,
`julia4-off` in 8,943 and `newtbasin-off` in 585. A compatible renderer has
to copy the same pixels.

Rule (confirmed by exact reproduction):

- Rows. Let `Δy = (ymax − ymin)/(H − 1)` and `S = round(2·ymax/Δy)`. So `S/2`
  is the (fractional) row of `y = 0`. Each row `r` with `0 ≤ S − r < r` (that
  is, `S/2 < r ≤ S`) is a **copy of row `S − r`**. All other rows are
  computed. So the rows from the top down to the axis are computed, then
  mirrored downwards, whichever side is larger. Observed (each file against
  its `…n`/`…nosym` partner): `S` = 359 (2·ymax/Δy = 358.8, `mandel4-off`),
  332 (332.22, `mandel4-w1`), 239 (239.2, bottom side larger, `mandel4-w2`),
  528 (527.65, `mandel4-w3`) and 427 (427.14, `mandel4-w4`). If the axis is
  outside the view (`mandel4-w5`, y from 0.1 to 1.5), nothing is copied.
  Exact half-way rounding was not tested.
- Columns. `Δx = (xmax − xmin)/(W − 1)`, `T = round(−2·xmin/Δx)`. Column `c`
  with `0 ≤ T − c < c` is a copy of column `T − c`.
- Kinds of symmetry:
  - **x-axis**: copy rows only, so pixel (r, c) ← (S − r, c).
  - **xy**: copy rows and columns independently, so (r, c) ← (r′, c′), where
    r′ = S − r for copied rows (else r) and c′ = T − c for copied columns
    (else c). Seen for barnsleym1 only (`barnsleym1-xy-s`).
  - **origin**: for a copied row `r` and `0 ≤ T − c < W`, (r, c) ←
    (S − r, T − c). Columns whose partner lies outside the image are computed.
- newtbasin also **swaps colours** on copied pixels (§3.13).
- `symmetry=none` turns all copying off (stored 0 at 124).
- Not tested: rotated or skewed views (6-value corners), symmetry with
  `passes` other than 1, and forced `symmetry=` values other than none.

Which types copy, and when, is listed in the summary table (§2) and in each
type section. "Not used" means: with an off-centre view that has
rounding-sensitive pixels, the default render was identical to
`symmetry=none`, although mirroring the nosym image would have changed
hundreds or thousands of pixels.

Effect on match rate when the rule is left out (off-centre views): mandel4
98.75 % instead of 99.998 % (`mandel4-off`), julia4 92.5 % instead of 100 %
(`julia4-off`), barnsleym1 94.6 % instead of 99.97 % (`barnsleym1-xy-s`),
magnet1j (P2 = 0) 93.5 % instead of 99.998 % (`magnet1j-pr-s`).

### 1.7 How the match rates were measured

We decoded each GIF ourselves, computed every pixel (all 120,000, not a
sample) from the stored corners, params, maxiter and bailout, applied §1.2,
§1.3 and §1.6, and compared. Python `complex` IEEE double arithmetic. The
throwaway scripts are not in the repo.

Residual mismatches (§4) are always at high iteration counts, in chaotic
regions near the set boundary. We showed they are limited by rounding: two
algebraically equal evaluation orders of our own formula disagree with each
other on a similar number of pixels.

---

## 2. Summary

| Type | # (001[10]) | Default corners xmin, xmax, ymin, ymax | Default P1, P2 | B | z0 / constant | Symmetry (default) | Match `-def` / params variant |
|---|---|---|---|---|---|---|---|
| lambda | 75 | -1.5, 2.5, -1.5, 1.5 | 0.85, 0.6 | 4 | z0 = w, λ = a | not used | 100 % / 100 % |
| mandellambda | 82 | -3, 5, -3, 3 | 0, 0 | 4 | z0 = 0.5 + a, λ = w | x-axis if a = 0 | 100 % / 100 % |
| mandel4 | 155 | -2, 2, -1.5, 1.5 | 0, 0 | 4 | z0 = w + a, c = w | x-axis if a = 0 | 100 % / 100 % |
| julia4 | 156 | -2, 2, -1.5, 1.5 | 0.6, 0.55 | 4 | z0 = w, c = a | origin | 99.998 % / 100 % |
| manowar | 9 | -2.5, 1.5, -1.5, 1.5 | 0, 0 | 4 | z0 = w + a, z−1 = w, c = w | x-axis if a = 0 | 100 % / 100 % |
| manowarj | 108 | -2.5, 1.5, -1.5, 1.5 | 0, 0 | 4 | z0 = z−1 = w, c = a | not used | 100 % / 100 % |
| barnsleym1 | 76 | -2, 2, -1.5, 1.5 | 0, 0 | 4 | z0 = w + a, c = w | xy if a = 0 | 99.95 % / 99.93 % |
| barnsleyj1 | 77 | -2, 2, -1.5, 1.5 | 0.6, 1.1 | 4 | z0 = w, c = a | origin | 99.985 % / 100 % |
| spider | 93 | -2.5, 1.5, -1.5, 1.5 | 0, 0 | 4 | z0 = w + a, c0 = w | x-axis if a = 0 | 100 % / 100 % |
| magnet1m | 96 | -4, 4, -3, 3 | 0, 0 | 100 | z0 = a, c = w | x-axis if a = 0 | 99.98 % / 99.99 % |
| magnet1j | 97 | -8, 8, -6, 6 | 0, 0 | 100 | z0 = w, c = a | x-axis if P2 = 0 | 100 % / 99.92 % |
| newton | 5 | -2, 2, -1.5, 1.5 | 3, 0 | — | z0 = w, degree = trunc(P1) | x-axis | 100 % / 100 % |
| newtbasin | 2 | -2, 2, -1.5, 1.5 | 3, 0 | — | z0 = w, degree = trunc(P1) | x-axis + colour swap | 100 % / 100 % |
| phoenix | 148 | -2, 2, -1.5, 1.5 | 0.56667, -0.5 | 4 | z0 = w, z−1 = 0, p = P1, q = P2 | x-axis | 100 % / 100 % |
| mandphoenix | 150 | -2.5, 1.5, -1.5, 1.5 | 0, 0 | 4 | z0 = w + a, z−1 = 0, p = x, q = y | not used | 100 % / 100 % |

All 15 names were accepted by this build. Type numbers, corners and default
params were read from the `-def` files (no corners or params given). The
"params variant" is the `-p…` file named in each section. The third corner
(184/192) always equals (xmin, ymin).

Note: gif-fractint-blocks §6.1 lists newtbasin with "newton p1 = 3, newtbasin
p1 = 5 (`params=5`)". That 5 came from the command line. newtbasin's real
default is 3 (`newtbasin-def`).

---

## 3. Types

Each section gives the start, the step, the stop test and the colour. All
types use §1.2 for the count and colour and §1.6 for symmetry.

### 3.1 lambda (75)

- Start `z0 = w`, constant `λ = a` (default 0.85 + 0.6i).
- Step `z ← λ·z·(1 − z)`.
- Stop `|z|² > B` (B = 4).
- Symmetry: none copied. The default view is centred on the set's point of
  symmetry (0.5, 0) and the image is point-symmetric. Even so, XFractint
  copied nothing in `lambda-w1` (`corners=-1/2/-1/1.5`; compare
  `lambda-w1n`), where a point mirror about (0.5, 0) would have changed 8,079
  pixels. It also copied nothing with real
  `λ = 1.2`, where an x-axis mirror would have changed 476.
- Evidence: `lambda-def` 100 %, `lambda-p` (`params=1.2/0.3`) 100 %,
  `lambda-mi300` 99.999 % (1 pixel at e ≈ 203).

### 3.2 mandellambda (82)

- Start `z0 = 0.5 + a`, constant `λ = w`.
- Step `z ← λ·z·(1 − z)`.
- Stop `|z|² > B`.
- Symmetry: x-axis when `a = 0`. Not used for `a ≠ 0` (checked with
  0.1 + 0.2i). The set is also symmetric about Re λ = 1, but no column
  copying happens. `mandellambda-v-s` (corners -2/5/-3/3) is identical to
  `mandellambda-v-sn`, although mirroring about Re λ = 1 would change 11,120
  pixels.
- Params: `P1, P2` shift the start. With `params=0.1/0.2`, `z0 = 0.5 + a`
  scored 100 %, `z0 = a` 48 %.
- Evidence: `mandellambda-def` 100 %, `mandellambda-p` 100 %.

### 3.3 mandel4 (155)

- Start `z0 = w + a`, constant `c = w`.
- Step `z ← z⁴ + c` (we computed `(z·z)·(z·z)`).
- Stop `|z|² > B`.
- Symmetry: x-axis when `a = 0`. Not used for `a = 0.2 + 0.1i` or `a = 0.2`
  (real).
- Evidence: `mandel4-def` 100 %, `mandel4-p` (`0.2/0.1`) 100 %, `mandel4-off`
  99.998 %, `mandel4-w2` 100 % (bottom side larger).

### 3.4 julia4 (156)

- Start `z0 = w`, constant `c = a` (default 0.6 + 0.55i).
- Step `z ← z⁴ + c`.
- Stop `|z|² > B`.
- Symmetry: origin (z ↔ −z), used for complex `c` (`julia4-off`, and
  `params=0.4/0.3` in an off-centre view).
- Evidence: `julia4-def` 99.998 % (2 pixels, e ≈ 147), `julia4-p` (`0.4/0.3`)
  100 %, `julia4-off` 100 %.

### 3.5 manowar (9)

- Two-term recurrence. Start `z0 = w + a`, previous `z(−1) = w` (the offset
  is **not** added to it), constant `c = w`.
- Step `z(k+1) = z(k)² + z(k−1) + c`.
- Stop `|z|² > B`.
- Alternatives that failed on `-def`: `z0 = z(−1) = 0` (18.5 %) and
  `z0 = w, z(−1) = 0` (56 %). On `manowar-p` (`0.1/0.1`): adding `a` to both
  values gave 85.7 %, adding it to `z(−1)` only gave 76.6 %, and `z0 = w + a,
  z(−1) = w` gave 100 %.
- Symmetry: x-axis when `a = 0`. Not used for `a = 0.1` (real).
- Evidence: `manowar-def` 100 %, `manowar-p` 100 %, `manowar-bo16` 100 %.

### 3.6 manowarj (108)

- Start `z0 = z(−1) = w`, constant `c = a` (default 0).
- Step and stop as manowar.
- Symmetry: none copied, even with real `c` (`params=0.2/0`) or the default
  `c = 0`, in an off-centre view (`manowarj-off` is identical to
  `manowarj-offnosym`). There, an x-axis mirror would have changed 2,304 and
  3,568 pixels.
- Evidence: `manowarj-def` 100 %, `manowarj-p` (`0.2/0.3`) 100 %.

### 3.7 barnsleym1 (76)

- Start `z0 = w + a`, constant `c = w`.
- Step: if `Re(z) ≥ 0` then `z ← (z − 1)·c`, else `z ← (z + 1)·c`. `≥` and
  `>` at 0 could not be told apart.
- Stop `|z|² > B`.
- Symmetry: **xy** (rows and columns copied independently) when `a = 0`. The
  off-centre view `barnsleym1-xy-s` reproduces with 0 extra mismatches under
  xy, and differs by 3,961–4,471 pixels under x-only, y-only or origin. Not
  used for `a = 0.2 + 0.1i`.
- Evidence: `barnsleym1-def` 99.95 %, `barnsleym1-p` (`0.2/0.1`) 99.93 %.
  Residuals in §4.

### 3.8 barnsleyj1 (77)

- Start `z0 = w`, constant `c = a` (default 0.6 + 1.1i).
- Step and stop as barnsleym1.
- Symmetry: origin. `barnsleyj1-o-s` (corners -1.2/2/-1.2/1.5) differs from
  `barnsleyj1-o-sn` in 20,060 pixels. Origin-mirroring the nosym image
  reproduces it exactly, while x, y or xy mirroring leaves ≥ 51,000 pixels
  different. `barnsleyj1-o-s` model match: 99.988 %.
- Evidence: `barnsleyj1-def` 99.985 %, `barnsleyj1-pa` (`1.1/0.6`) 100 %.
- Caution: `params=0.4/1.2` gives a very chaotic image where only 96.4 % match.
  Of the pixels with e < 100, 0 of 68,386 mismatch. Two equivalent evaluation
  orders of our own formula disagree on 3,759 pixels. Use `barnsleyj1-pa` for
  tests.

### 3.9 spider (93)

- Two sequences. Start `z0 = w + a`, `c0 = w`.
- Step: `z(k+1) = z(k)² + c(k)`, then `c(k+1) = c(k)/2 + z(k+1)` (it uses the
  **new** z).
- Stop `|z|² > B`.
- Alternative `z0 = 0`: 33–65 %.
- Symmetry: x-axis when `a = 0`. Not used for `a = 0.1 + 0.1i` or `a = 0.1`.
- Evidence: `spider-def` 100 %, `spider-p` (`0.1/0.1`) 100 %.

### 3.10 magnet1m (96)

- Start `z0 = a` (default 0), constant `c = w`.
- Step `z ← ((z² + c − 1) / (2z + c − 2))²`.
- Stop **either** `|z|² > B` (B = 100) **or** `|z − 1|² < 2⁻¹⁵` (convergence
  to the fixed point 1). Both kinds of stop are coloured by `e` as in §1.2.
- The convergence threshold was found from the data. Over all converging
  pixels of `magnet1m-def`, the largest `|z(e) − 1|²` at the stopping iterate
  was 3.05131·10⁻⁵, and the smallest `|z(e−1) − 1|²` one iterate earlier was
  3.05190·10⁻⁵. That brackets 2⁻¹⁵ = 3.05176·10⁻⁵. Distances measured
  differently fit worse at their best threshold: L1 or L∞ distance to 1, or
  the step size `|z(k) − z(k−1)|²`, all at ≤ 98.3 %. `<` versus `≤` is
  unknown.
- B = 4 instead of 100 gives at most 68.6 %.
- Symmetry: x-axis when `a = 0`. Not used for `a ≠ 0`, even real `a = 0.3`.
- Evidence: `magnet1m-def` 99.98 %, `magnet1m-p` (`0.3/0.2`) 99.99 %,
  `magnet1m-bo10` 99.995 %.

### 3.11 magnet1j (97)

- Start `z0 = w`, constant `c = a` (default 0).
- Step and stop as magnet1m: B = 100 and `|z − 1|² < 2⁻¹⁵`. `magnet1j-def`
  fits any threshold from 3·10⁻⁸ to 10⁻⁴, so the threshold value comes from
  magnet1m.
- Symmetry: x-axis when `P2 = 0`, i.e. `c` real (`magnet1j-pr-s`,
  `params=1.5/0`). Not used for `c = 1.5 + 0.5i`.
- Evidence: `magnet1j-def` 100 %, `magnet1j-p` (`1.5/0.5`) 99.92 %,
  `magnet1j-bo1000` 100 %.

### 3.12 newton (5)

- Degree `d = trunc(P1)`, default 3. `params=3.5` gives the same pixels as
  `d = 3`, and the file stores 3.5. Also reproduced: d = 2, 4, 5, 8.
- Start `z0 = w`.
- Step (Newton's method for `z^d = 1`): `z ← z − (z^d − 1)/(d·z^(d−1))`. We
  computed it as `((d − 1)·z^d + 1)/(d·z^(d−1))`.
- Stop test, **applied before each step, starting with z0**: stop at the first
  `k ≥ 0` with `|z(k)^d − 1|² < C/d`.
  - `C` lies in **(0.9424745, 0.9424869]**. That is the intersection of the
    intervals that fit `newton-def`, an off-centre nosym render, and the
    d = 2, 4, 5 and 8 renders. Each interval was consistent for every pixel.
    `C = 0.942478` works. (3π/10 = 0.9424778 is inside the interval, but
    we have no further support for it, so treat that as a coincidence.)
  - Distance to the nearest root, or the step size, did not separate the
    pixels for any threshold (≤ 90 % at best).
- Colour: `n = k + 1`, the number of steps plus one, so a start point that
  already passes gets colour 1. If `n ≥ M` the pixel is inside (0)
  (`newton-mi20`, maxiter 20: 100 %). Otherwise `n` is coloured as in §1.2.
- `bailout=` and `P2…P4` have no effect (`newton-bo100`, and renders with
  `params=3/1` and `3/0/1/1`).
- Symmetry: x-axis, for every degree tested (3 and 4 in off-centre views).
- Evidence: `newton-def`, `newton-p2`, `newton-p5`, `newton-p8`,
  `newton-mi20` and `newton-bo100` are all 100 %. A d = 4 default view scored
  99.997 % (4 pixels at n ≈ 77).

### 3.13 newtbasin (2)

- Degree, start, step and stop test exactly as newton (§3.12).
- Colour = `1 + j`, where `j ∈ {0, …, d−1}` is the index of the root
  `exp(2πi·j/d)` nearest to the final `z(k)`. So the root 1 gets colour 1,
  and the colours go anticlockwise. Inside (no stop with `k + 1 < M`) is 0.
- **Stripes**: if `P2 ≠ 0` (tested 1, 2 and −1), add 8 when `k` is even. A
  start point that already passes (k = 0) counts as even. So colours are
  `1…d` and `9…8+d`.
- Symmetry: x-axis, and copied pixels **swap roots**: colour `1 + j` becomes
  `1 + ((d − j) mod d)`, and stripe colours are swapped the same way. Evidence:
  `newtbasin-off` (d = 3), and off-centre renders for d = 4, d = 5 and
  d = 3 with stripes all reproduce exactly.
- Evidence: `newtbasin-def` (3 colours), `newtbasin-p5`, `newtbasin-p5-1`
  (stripes) and `newtbasin-off` are all 100 %. d = 4 and 6 also gave 100 %.
- Untested: d > 8. With stripes the indices would pass 16.

### 3.14 phoenix (148)

- Two-term recurrence with real coefficients `p = P1` and `q = P2`
  (default 0.56667, −0.5). Start `z0 = w`, previous `z(−1) = 0`.
- Degree selector `D = trunc(P3)`:
  - `D = 0` (default): `z(k+1) = z(k)² + p + q·z(k−1)`.
  - `D ≠ 0`: `z(k+1) = z(k)^D + p·z(k)^(D−1) + q·z(k−1)`. Confirmed for
    D = 1, 2, 3, 5 and −2 (negative powers), with P3 = 2.5 behaving as 2.
- Stop `|z|² > B`.
- P4 is ignored. A swapped-axes start, or `z(−1) = w`, fails (44–54 %).
- Symmetry: x-axis (also checked with `0.3/0.2` and with D = 3).
- Evidence: `phoenix-def` 100 %, `phoenix-p` (`0.3/0.2`) 100 %, `phoenix-p3d3`
  (`0.56667/-0.5/3`) 100 %.

### 3.15 mandphoenix (150)

- The pixel gives the coefficients `p = x`, `q = y`. Start `z0 = w + a`,
  previous `z(−1) = 0`.
- Same recurrence and degree selector `D = trunc(P3)` as phoenix.
- Stop `|z|² > B`.
- Alternatives that failed: `z0 = 0` (≤ 46 %) and swapping p and q
  (≤ 27 %).
- Symmetry: none copied. The formula has no mirror symmetry in `w`. An
  x-axis mirror would change 16,797 pixels of the off-centre view.
- Evidence: `mandphoenix-def` 100 %, `mandphoenix-p` (`0.1/0.1`) 100 %,
  `mandphoenix-p3d3` (`0/0/3`) 100 %.

---

## 4. Verification and residuals

Per-file results for the committed evidence. These are full images,
120,000 pixels each, after §1.3 canonicalisation and with §1.6 symmetry
applied:

| File | Match | Mismatches (lowest colour involved) |
|---|---|---|
| lambda-def, lambda-p | 100 % | 0 |
| lambda-mi300 | 99.999 % | 1 (203) |
| mandellambda-def, -p | 100 % | 0 |
| mandel4-def, -p, -w1…-w5, -w1n, -w3n…-w5n | 100 % | 0 |
| mandel4-off | 99.998 % | 2 (141) |
| julia4-def | 99.998 % | 2 (147) |
| julia4-p, julia4-off | 100 % | 0 |
| manowar-def, -p; manowarj-def, -p | 100 % | 0 |
| barnsleym1-def | 99.950 % | 60 (112) |
| barnsleym1-p | 99.932 % | 82 (92) |
| barnsleym1-xy-s | 99.968 % | 38 (114) |
| barnsleyj1-def | 99.985 % | 18 (138) |
| barnsleyj1-pa | 100 % | 0 |
| spider-def, -p | 100 % | 0 |
| magnet1m-def | 99.983 % | 20 (67) |
| magnet1m-p | 99.993 % | 9 (78) |
| magnet1m-bo10 | 99.995 % | 6 (67) |
| magnet1j-def | 100 % | 0 |
| magnet1j-p | 99.918 % | 98 (53) |
| magnet1j-pr-s | 99.998 % | 2 (47) |
| newton-def, -p2, -p5, -p8, -mi20, -bo100 | 100 % | 0 |
| newtbasin-def, -p5, -p5-1, -off | 100 % | 0 |
| phoenix-def, -p, -p3d3 | 100 % | 0 |
| mandphoenix-def, -p, -p3d3 | 100 % | 0 |
| lambda-bo100, manowar-bo16, magnet1j-bo1000 | 100 % | 0 |
| lambda-w1, lambda-w1n, mandellambda-v-s, -v-sn, manowarj-off, -offnosym | 100 % | 0 |
| barnsleyj1-o-s / -o-sn | 99.988 % / 99.988 % | 14 (135) / 15 (131) |
| nosym partners: mandel4-offnosym, mandel4-w2n, julia4-offnosym, magnet1j-pr-sn | 99.999 % | 1 each (≥ 47) |
| nosym partners: newtbasin-offnosym | 100 % | 0 |
| barnsleym1-xy-sn | 99.963 % | 45 (114) |
| magnet1m-pr-s / -pr-sn (symmetry not used) | 99.986 % | 17 (61) |

All residuals are high-count pixels in chaotic boundary regions. barnsley and
magnet have the most, because their maps are discontinuous (barnsley) or
involve a division (magnet), so rounding differences grow quickly. Evidence
that the rounding is the limit: for `magnet1j-p`, Python's complex division
and a plain real-arithmetic division disagree with each other on 84 pixels
and match XFractint equally well (98 vs 95 mismatches). `magnet1m-def`: 21
disagreements. barnsleyj1 `0.4/1.2`: 3,759. We did not try to emulate x87
extended precision or any particular operation order. Exact agreement on
those pixels may not be achievable.

Also checked during analysis, with renders that are not committed. Rerun
any `-def` case with `float=yes` or `symmetry=none` added to reproduce them.
`float=yes` is pixel-identical for all 15 types.
`symmetry=none` is pixel-identical on all default (symmetric) views.
`symmetry=none` on off-centre views matches our model without mirroring.

## 5. Unknown or untested

- Behaviour with `passes` other than 1 (guessing, boundary tracing),
  periodicity checking on, non-default `inside`/`outside`, `bailoutest=`,
  `decomp`, `biomorph`, `potential`, `distest`, and rotated or skewed views.
- `>` vs `≥` in the escape tests. `<` vs `≤` in the magnet and newton
  convergence tests.
- newton/newtbasin with d ≤ 1 or d > 8. The newtbasin stripe offset when
  `8 + d > 15`.
- Symmetry: exact half-way rounding of `S`/`T`, and forced `symmetry=` values.
- julia4 and the other Julia-like types with `P3`/`P4` set.
