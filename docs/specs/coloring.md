<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Colouring options: inside, outside, decomp, biomorph

Status: **Phase 2 draft, derived from black-box analysis only.** Each rule
below was found by rendering images with XFractint, computing every pixel's
orbit ourselves, and fitting a formula that gives the saved colour index. No
Fractint, XFractint or Iterated Dynamics source code, help text or
documentation was consulted. See [CLEANROOM.md](../CLEANROOM.md). The names
and formulas here are ours.

Scope: `type=mandel` and `type=julia`, single pass (`passes=1`), with
`inside=`, `outside=`, `decomp=` and `biomorph=`. Resume data is not covered.

Conventions:

- **Confirmed** means the rule matched ≥ 99.9 % of all pixels of every image
  listed as evidence, and the remaining pixels are explained or are boundary
  pixels with long orbits (§10). **Hypothesis** means it fits but wasn't
  isolated. **Unknown** means we couldn't find a rule.
- Evidence names are files in `corpus/xfractint/coloring/` (`<name>.gif`).
  Their parameters are in `tools/blackbox/coloring/cases.txt`. Every case also
  has `passes=1 periodicity=0 viewwindows=2 map=uniq.map` (400×300 pixels,
  see the corpus README), unless the case overrides `periodicity`.
- Name prefixes: `m-` is mandel with the default view and maxiter 150. `m50-`
  and `m300-` are the same with maxiter 50 and 300. `z-` is mandel with
  `corners=-0.80/-0.70/0.05/0.15` and maxiter 1000. `b-` is `m-` with
  `bailout=100`. `j-` is julia with its default parameters (0.3, 0.6) and
  maxiter 150. That set has no inside pixels. `j2-` is julia with
  `params=-0.8/0.156` and maxiter 500, and `j3-` is julia with
  `params=-0.12/0.75` and maxiter 150.
- Pixel coordinates follow `gif-fractint-blocks.md` §2.2. Match rates count
  **all** 120,000 pixels of an image unless they say "inside pixels".
- `T(x)` truncates toward zero, like a C cast from double to int:
  `T(-0.9) = 0`, `T(-2.4) = -2`. Rounding direction matters for several modes.
  `floor` gives measurably worse fits (§5.2).

---

## 1. Measuring colour indices: duplicate palette colours

XFractint does **not** always save the index it computed. When the palette
has two entries with the same colour, the saved pixel is the **first** index
with that colour. The default palette has these duplicates:

| Index | Saved as |
|---|---|
| 16 | 0 (black) |
| 31 | 15 (white) |
| 248–255 | 0 (black) |

Evidence: with the default palette, every pixel with iteration count 16 was
saved as 0 and every pixel with count 31 was saved as 15
(`defpal-m-iter`: 206 and 38 pixels, which are exactly the "0.3 % unexplained" pixels in
`gif-fractint-blocks.md` §9). With `uniq.map` (256 distinct colours) the same
render gives the computed index everywhere (`m-iter`, 100 %). Running our
model through "first index with the same colour" reproduces the
default-palette files: `defpal-m-iter` 100 %, `defpal-z-iter-1000` 99.992 %.

Consequences:

- Every measurement in this spec uses `map=uniq.map`.
- A writer that wants XFractint-identical GIFs while using the default palette
  must apply the same collapse. A reader can't tell index 16 from index 0 in
  such files. **Confirmed.**
- `gif-fractint-blocks.md` §9's rule "n mod 256" is wrong for n ≥ 256. See
  §3 here.

---

## 2. Orbit, escape count and final value

Let B be the bailout: `bailout=` if given, otherwise **4**, but **100** when
`biomorph=` is on and no `bailout=` was given (§8). The second value of
`decomp=K/B` sets the bailout too (`m-decomp-4-50`, stored in the bailout
field). The escape test is |z|² > B. We couldn't tell `>` from `≥`.

### 2.1 Mandelbrot (`type=mandel`)

z₀ = 0, z₁ = c, z_{k+1} = z_k² + c.

- The test is applied to z₂, z₃, … z_maxiter. **z₁ = c is never tested.**
- If z_k is the first tested iterate past B, the pixel is **outside**. Its
  escape count is **N = k − 1**, so 1 ≤ N ≤ maxiter − 1, and its
  final value is **Z = z_k**.
- Otherwise the pixel is **inside**, with final value Z = z_maxiter.

Evidence: a pixel with |c| > 2 gets N = 1, and `outside=real`
uses the real part of z₂, not of c (`m-out-real`, `m-decomp-*`, `m-out-fmod`).
Testing z₁ as well drops `m-out-real` and `m-decomp-4` to 82 %.

### 2.2 Julia (`type=julia`)

z₀ = pixel, z_{k+1} = z_k² + c with c = (param1, param2).

- For almost all modes, the iteration and test are the **same as for mandel**:
  the test starts at z₂, N = k − 1 and Z = z_k. This holds for `iter`, `real`,
  `imag`, `mult`, `summ`, `atan`, `fmod`, biomorph and all inside modes (`j-*`,
  `j2-*`, `j3-*`, all ≥ 99.99 %).
- **Exception: z₁ past B.** Three modes treat a pixel whose first iterate is
  already past B differently:
  - decomposition takes the angle of **z₁** (`j-decomp-16`: 100 % with this
    rule, 55 % without);
  - `outside=tdis` gives the value 0, so colour 1 (§5.8);
  - `inside=epscross` skips its axis test, so the pixel gets its outside
    colour, 1 (§6.8).

  The other modes take that pixel as N = 1 with Z = z₂: `outside=real`
  gives 1 + 7 + T(Re z₂) (`j-out-real`: 100 % this way, 53 % with Z = z₁).
  **Confirmed** (black-box, though inconsistent).

---

## 3. Two value-to-index maps

Most modes compute an integer value v and then map it to an index in 0…255.
There are two maps.

**W (wrap)**, used by inside modes, decomposition and the biomorph colour:

    W(v) = v                      if v ≤ 255
    W(v) = ((v − 1) mod 255) + 1  if v ≥ 256

So 256 → 1, 300 → 45, 511 → 1, 1000 → 235. Index 0 is produced only by a
value of exactly 0. Evidence: `m-in-300` (inside → 45), `m-in-256` (→ 1),
`m-in-511` (→ 1), `m-in-maxiter-256` (→ 1), `m-in-maxiter-1000` (→ 235),
`m-bio-300` (biomorph colour → 45). **Confirmed.**

**P (outside)**, used by outside modes iter, real, imag, mult, summ, atan
and tdis:

    P(v) = 1      if v ≤ 0 or v > maxiter
    P(v) = W(v)   otherwise

Evidence: iteration counts ≥ 256 (`z-iter-1000`: 256 → 1, 257 → 2,
510 → 255, 511 → 1). Negative and zero values of `summ` and `mult` become 1
(`m-out-summ`: 32,562 pixels, all 1). Values above maxiter become 1:
`mult` 151–5000 at maxiter 150 (`m-out-mult`: 629 pixels), `atan`
151–256 (`m-out-atan`). A value equal to maxiter is kept. **Confirmed.**

`outside=` fixed colours and `fmod` don't go through P (§5.1, §5.7).

---

## 4. Precedence

For each pixel:

1. `inside=epscross` is checked during iteration and can claim outside
   pixels too (§6.8).
2. **Inside** pixels (never past B) get their inside colour. `decomp`
   and `biomorph` don't affect inside pixels (`m-decomp-4-in-zmag`,
   `m-bio-0-in-zmag`: 100 %).
3. **Outside** pixels get:
   1. the decomposition colour if `decomp` is on. This beats both biomorph
      and `outside=` (`m-bio-0-decomp-4`, `m-decomp-4-out-real`: 100 %);
   2. otherwise, if `biomorph` is on and the biomorph test passes, the
      biomorph colour (§8);
   3. otherwise the `outside=` colour.

With biomorph and decomp together, the bailout still becomes 100
(`m-bio-0-decomp-4`: 100 % at B = 100, 32 % at B = 4).

---

## 5. `outside=` modes

Z = x + iy is the final value and N is the escape count (§2). All results go
through P (§3) except 5.1 and 5.7.

| § | Mode | Stored code (`fractint001`[182]) | Value v |
|---|---|---|---|
| 5.1 | n (0…255) | n | index n directly |
| 5.2 | `iter` (default) | -1 | N |
| 5.3 | `real` | -2 | N + 7 + T(x) |
| 5.4 | `imag` | -3 | N + 7 + T(y) |
| 5.5 | `mult` | -4 | T(N · x / y) |
| 5.6 | `summ` | -5 | N + T(x + y) |
| 5.6a | `atan` | -6 | T(\|atan2(y, x)\| · 256 / π) |
| 5.7 | `fmod` | -7 | see below (no P) |
| 5.8 | `tdis` | -8 | T(path length) |

### 5.1 Fixed colour

`outside=n` paints every outside pixel with index n, with no mapping:
`outside=0` gives 0, `outside=255` gives 255 (`m-out-0`, `m-out-9`,
`m-out-255`: 100 %). XFractint rejects `outside=256` and `outside=300`
("couldn't understand the argument"), and no image is saved. **Confirmed.**

### 5.2 `iter`

v = N, mapped by P. N is never 0 (§2), so the lowest index is 1, and counts
of 256 and up wrap to 1, 2, … (not to 0). Match: `m-iter`,
`m-iter-float`, `m-b16-iter`, `j-iter`, `j3-iter` 100 %; `z-iter-1000`
99.992 %; `j2-iter` 99.997 %. **Confirmed.**

Note: the "escape on the first iterate is colour 1" rule in
`gif-fractint-blocks.md` §9 is the same as this one. That spec's n counts from
z₁, so its n = 0 is our N = 1.

### 5.3 `real` and 5.4 `imag`

v = N + 7 + T(x), and v = N + 7 + T(y). T applies to the coordinate alone:
`T(N + 7 + x)` fails on pixels with negative x, such as N = 1, x = −0.947,
which saves 8 (not 7). Match: `m-out-real` 100 %, `b-out-real` 100 %,
`j-out-real` 100 %, `z-out-real` 99.991 %, `m-out-imag` 99.994 %,
`b-out-imag` 100 %. **Confirmed.**

### 5.5 `mult`

v = T(N · x / y). Values ≤ 0 (x and y of opposite sign) and values above
maxiter become 1 through P. Match: `m-out-mult` 100 %, `b-out-mult` 100 %,
`z-out-mult` 99.991 %, `j2-out-mult` 99.990 %. **Confirmed.** y = 0 exactly was
never seen. **Unknown** what happens there.

### 5.6 `summ`

v = N + T(x + y). The sum is truncated as a whole: N + T(x) + T(y) fails
(N = 3, Z = −3.357 + 1.387i saves 2). Match: `m-out-summ` 100 %, `b-out-summ`
100 %, `z-out-summ` 99.990 %. **Confirmed.**

### 5.6a `atan`

v = T(|atan2(y, x)| · 256 / π), so v is 0…256 and doesn't depend on N. At
maxiter 150, every pixel with v > 150 (Z in the left half-plane) becomes 1,
and v = 0 becomes 1. The constant doesn't depend on maxiter: `m50-out-atan`,
`m-out-atan`, `m300-out-atan` (99.994 %), `b-out-atan`, `j-out-atan` are
100 %, and `z-out-atan` is 99.984 %. **Confirmed.**

### 5.7 `fmod`

Look at the orbit iterates before the final value: z₂ … z_N for mandel, and z₁
… z_N for julia. Find the **last** iterate with |z|² < 0.01. Then

    index = T(|z|² / 0.01 · 256)     (0…255)

If no iterate qualifies, the index is **0** (not 1, and P isn't applied).
"Last" fits 100 % (`m-out-fmod`). "First" fits 99.86 % and "smallest" fits
99.90 %. Including z₁ for mandel breaks pixels with |c|² < 0.01. Including z₁ for
julia is required: `j2-out-fmod` gives 99.998 % with z₁ and 99.86 % with
z₀ and z₁. **Confirmed** for the default threshold 0.01. The
`fractint001` f64 at offset 458 is 0.01 in every file. That it holds this
threshold is a **hypothesis** (no option was varied to change it).

### 5.8 `tdis`

v = T(L), where L is the length of the path through consecutive orbit points
up to z_N, the last point **before** the final value:

- mandel: L = Σ |z_{k+1} − z_k| for k = 1…N−1 (from c to z_N)
- julia: L = Σ |z_{k+1} − z_k| for k = 0…N−1 (from the pixel to z_N),
  and L = 0 if |z₁|² > B (§2.2)

Not scaled by maxiter or bailout: `m50-out-tdis`, `m-out-tdis`,
`b-out-tdis`, `j3-out-tdis` 100 %; `m300-out-tdis`, `j2-out-tdis` 99.997 %.
**Confirmed.**

---

## 6. `inside=` modes

Z is z_maxiter (§2). Results go through W (§3) except fmod.

| § | Mode | Stored code (`fractint001`[104]) | Index |
|---|---|---|---|
| 6.1 | n | n | W(n) |
| 6.2 | `maxiter` | -1 | W(maxiter) |
| 6.3 | `zmag` | -59 | W(T(\|Z\|² · maxiter / 2) + 1) |
| 6.4 | `bof60` | -60 | W(T(√m · 75)) |
| 6.5 | `bof61` | -61 | W(k_m) |
| 6.6 | `atan` | -104 | W(T(\|atan2(Im Z, Re Z)\| · 256 / π)) |
| 6.7 | `fmod` | -103 | as §5.7 over the whole orbit, else 0 |
| 6.8 | `epscross` | -100 | 2, 6 or W(maxiter) |
| 6.9 | `period` | -102 | partly known |
| 6.10 | `startrail` | -101 | unknown |

m is the smallest |z_k|² for k = 1…maxiter, and k_m is the first k where it
occurs.

### 6.1 Fixed colour

`inside=0` (default), 7, 255 give that index. 256 → 1, 300 → 45, 511 → 1
(`m-in-0`, `m-in-7`, `m-in-255`, `m-in-256`, `m-in-300`, `m-in-511`: 100 %).
**Confirmed.**

### 6.2 `maxiter`

W(maxiter): 150 → 150, 255 → 255, 256 → 1, 1000 → 235, 500 → 245
(`m-in-maxiter*`, `j2-in-maxiter`: 100 % of inside pixels). **Confirmed.**

### 6.3 `zmag`

W(T(|Z|² · maxiter / 2) + 1). The factor scales with maxiter: `m50-in-zmag`,
`m-in-zmag`, `b-in-zmag` and `j3-in-zmag` match 100 % of inside pixels,
`m300-in-zmag` 99.987 %, and `j2-in-zmag` (maxiter 500, so wrapping) 98.9 %
of 638 inside pixels (§10). Values above maxiter are **not** turned into 1.
**Confirmed.**

### 6.4 `bof60`

W(T(√m · 75)), with m the smallest |z_k|² over z₁ … z_maxiter. **z₁ = c counts**
for mandel. 1,234 inside pixels of `m-in-bof61` have their minimum at z₁. For
julia, z₀ (the pixel) does **not** count. In `j3-in-bof61` there are 4 pixels
where it would be the minimum, and they match only without it. The factor 75
is a constant: it stays the same at maxiter 50, 150, 300 and 500
(`m50-in-bof60`, `m-in-bof60`, `m300-in-bof60`, `j2-in-bof60`). Match: 100 %
of inside pixels in every file. **Confirmed.**

### 6.5 `bof61`

W(k_m): the iterate number (z₁ = 1) where the smallest |z|² first occurs, over
the same range as §6.4. Ties were never tested. Match: 100 % of inside pixels
(`m-in-bof61`, `m300-in-bof61`, `j2-in-bof61`, `j3-in-bof61`). **Confirmed.**

### 6.6 `atan`

W(T(|atan2(Im Z, Re Z)| · 256 / π)), the same formula as `outside=atan` but
applied to z_maxiter and without P. Match: 100 % of inside pixels in
`m-in-atan`, `m300-in-atan`, `j3-in-atan`, and 98.7 % (630/638) in
`j2-in-atan`. `inside=atan` stores periodicity 0
(`gif-fractint-blocks.md` §2.1). **Confirmed.**

### 6.7 `fmod`

Like §5.7 over the inside orbit: mandel z₂ … z_maxiter, julia z₁ …
z_maxiter. The last |z|² < 0.01 gives T(|z|²/0.01 · 256), and no such
iterate gives 0. Mandel z₁ = c must be left out: 16 pixels with
|c|² < 0.01 save 0 (`m-in-fmod`). Match: 100 % of inside pixels in
`m-in-fmod`, `m300-in-fmod`, `j3-in-fmod`, and 98.6 % in `j2-in-fmod`.
**Confirmed.**

### 6.8 `epscross`

This mode changes **outside pixels too**. While iterating, look at each
iterate before the escaping one: mandel z₂, z₃, …, julia z₁, z₂, …. At the
first iterate where

- |Re z| < 0.01, the pixel gets index **2**;
- otherwise, if |Im z| < 0.01, it gets index **6**.

Within one iterate the real part is checked first ("real first" fits 100 %,
"imaginary first" 99.98 %). Pixels with no such iterate get the normal colour:
inside pixels W(maxiter), outside pixels their outside colour. For a julia
pixel with |z₁|² > B there is no check (§2.2). Match: `m-in-epscross`,
`j3-in-epscross`, `j2-in-epscross` 100 %. With `periodicity=1`
(`m-in-eps-p1`) it drops to 99.94 %. 70 pixels then save 150, apparently because
periodicity detection stopped the orbit first. **Confirmed** with
`periodicity=0`. Threshold 0.01: the same **hypothesis** as §5.7.

### 6.9 `period` (partly known)

- With `periodicity=0`, every inside pixel gets W(maxiter) (`m-in-period`: all 150).
- With periodicity on (`m-in-period-p1`, `-p3`, `j3-in-period-p1`), inside
  pixels get 1, 2 or 3, or maxiter (150) when nothing was detected. 2 pixels
  get 121. The value tracks the orbit's true attracting period for periods 1–3:
  the main cardioid gives 1, the period-2 disc 2, the period-3 bulbs 3, and
  every `j3-` inside pixel (a 3-cycle) gives 3. But the period-4 and longer
  bulbs gave 150, some true period-2/3 pixels gave 1, and c = 0 gave 121.
  `periodicity=3` gives almost the same image as 1.
- With `periodicity=-3`, every detected pixel gets **7** instead
  (`m-in-period-pn`).

Reproducing this exactly needs XFractint's periodicity-detection schedule and
tolerance. Simple save-and-compare schemes with power-of-two or fixed
intervals and tolerances 1e-2…1e-6 matched at most 82 % of inside pixels.
**Unknown.**

### 6.10 `startrail` (unknown)

Indices are 0…15 (15 is over-represented, which suggests a cap at 15).
This mode changes outside pixels too: in `m-in-startrail`, 103,178 of 104,718 outside pixels
differ from `iter`, mostly becoming 0. The image doesn't change with
`periodicity=1` (`m-in-star-p1`). A search over simple functions of the
final or first iterates (coordinates, modulus, angle, tangent, ratios,
scaled by 1…256 or 2ⁿ/π, then mod 16 or capped at 15) found nothing better
than "always 0". **Unknown.**

---

## 7. Decomposition (`decomp=K`)

Only outside pixels change. Take the angle of the final value Z (for julia
with |z₁|² > B, of z₁, §2.2), measured in [0, 2π):

    f = (atan2(Im Z, Re Z) mod 2π) / 2π          in [0, 1)
    index = T(f · L) + (1 if K < 256 else 0)

The number of sectors L depends on K:

| K given | L | index range | Evidence |
|---|---|---|---|
| 1 | 4 | 1–4 | `m-decomp-1` |
| 2 | 2 | 1–2 | `m-decomp-2` |
| 3 | 4 | 1–4 | `m-decomp-3` |
| 4–7 | 4 | 1–4 | `m-decomp-4`, `-5`, `-6`, `-7` |
| 8–255 | largest power of 2 ≤ K | 1…L | `-8`, `-9`, `-15`, `-16`, `-17`, `-31`, `-32`, `-33`, `-64`, `-100` (64), `-128`, `-129`, `-200` (128), `-255` (128) |
| 256 | 256 | **0**–255 | `m-decomp-256`, `j2-decomp-256` |
| > 256 | **128** | **0**–127 | `-257`, `-300`, `-511`, `-512`, `-1024`, `-2048`, `-4096` |

Sector 1 starts at angle 0 (positive real axis) and goes counter-clockwise,
so `decomp=2` gives 1 in the upper half-plane and 2 in the lower. Bailout is the normal
B (`b-decomp-8` uses 100, `m-decomp-4-50` uses 50). Match: 100 % for all
`m-decomp-*`, `b-decomp-8`, `j-decomp-16`, `j3-decomp-8`; `z-decomp-4` 99.990 %;
`j2-decomp-256` 99.996 %. **Confirmed** for every K tested. Odd but observed:
the K = 1 and K = 3 rows, and the 128-sector result above 256.

---

## 8. Biomorph (`biomorph=b`)

- Bailout becomes 100 unless `bailout=` is given (§2). Every `m-bio-*` file
  without a bailout fits only with B = 100. N and Z then come from the
  B = 100 orbit, so N is larger than in the same image without biomorph.
- After escape, let s = √B. If |Re Z| < s **or** |Im Z| < s, the pixel
  gets W(b). Otherwise it gets its normal outside colour (`iter` by
  default, or the `outside=` mode).
- Evidence: s = √B, not a fixed 10. `m-bio-0-b4` (s = 2) 99.998 %,
  `m-bio-0-b50` (s ≈ 7.07) and `m-bio-0-b1000` (s ≈ 31.6) 100 %. The default
  B = 100: `m-bio-0`, `m-bio-5`, `b-bio-0`, `j-bio-0`, `j3-bio-0` 100 %;
  `z-bio-0` 99.992 %; `j2-bio-3` 99.994 %. `biomorph=300` paints 45
  (`m-bio-300`). The biomorph test comes before `outside=real`
  (`m-bio-0-out-real`: 100 % with B = 100). **Confirmed.**
- The test is strict `<` (hypothesis): the 2 misses in `m-bio-0-b4` are
  pixels where |Re Z| or |Im Z| is within rounding of s.

---

## 9. Interactions

- **maxiter.** It sets the inside/outside split (§2), the cut-off in P (values
  above maxiter → 1), the `inside=maxiter` colour and the `zmag` scale. It
  doesn't affect `bof60`, either `atan` or `tdis`.
- **Indices ≥ 256.** Values wrap with W: 256 → 1, so 0 is never reached by
  wrapping. This applies to iteration counts (`z-iter-1000`) and inside values.
- **Negative values.** Outside modes give 1 (P). Inside modes can't produce
  negative values.
- **Bailout.** It changes N and Z, and with them every outside mode. The
  formulas themselves are unchanged (`b-*` 100 %). It sets the biomorph
  threshold √B.
- **`float=yes`.** Pixel-identical to the default (`m-iter-float`).
- **Periodicity.** The default `periodicity=1` makes a few slowly escaping
  pixels inside: 14 of 120,000 in `m-iter-p1` saved 0 instead of N =
  81–145. All measurements here use `periodicity=0` to avoid this.
- **Guessing passes.** The older 800×600 corpus files (`passes=g`, periodicity
  on, default palette) fit our model plus §1 at 99.98 % (`mandel-pass1`,
  `mandel-outside-real`, `mandel-inside-bof60`), 99.97 % (`mandel-maxiter-1000`),
  99.96 % (`julia-params`), and 99.64 %/99.61 % (`mandel-decomp-2`,
  `mandel-biomorph`), on 20,000 random pixels each. The extra misses there are
  presumably guessing artefacts.

---

## 10. Residuals

Across the 141 single-pass images with a rule (all files except the
`*-period*` and `*-startrail*`/`*-star-*` ones), the model matches
16,919,674 of 16,920,000 pixels (99.998 %).

| Option | Images | Pixels matched | Worst image |
|---|---|---|---|
| iter | 8 | 99.997 % | `m-iter-p1` 99.988 % (periodicity on) |
| outside real / imag / mult / summ | 5 / 5 / 5 / 5 | 99.996–99.998 % | `z-out-*` ≈ 99.990 % |
| outside atan | 7 | 99.996 % | `z-out-atan` 99.984 % |
| outside fmod / tdis | 2 / 6 | 99.999 % | `j2-out-tdis` 99.997 % |
| outside n / inside n | 3 / 6 | 100 % | |
| inside maxiter / zmag / bof60 / bof61 / atan / fmod | 6 / 7 / 7 / 5 / 5 / 4 | 99.997–99.999 % | `j2-in-*` (inside pixels 98.6–98.9 % for zmag, atan, fmod) |
| inside epscross | 4 | 99.985 % | `m-in-eps-p1` 99.94 % (periodicity on) |
| decomp | 37 | 99.9996 % | `z-decomp-4` 99.990 % |
| biomorph | 14 | 99.999 % | `z-bio-0` 99.992 % |

Almost every remaining miss is a pixel with a long orbit: escape counts in
the hundreds in the zoomed `z-` view, or julia `j2-` inside pixels whose
z_maxiter is still moving after 500 iterations. Modes that depend on z_maxiter
(zmag, atan, fmod) are sensitive there. Modes that depend on the whole orbit
(bof60/61, maxiter) are not. Our orbits use IEEE double precision. XFractint's
arithmetic (for example x87 extended precision) may round differently. That
explanation is a **hypothesis**.

---

## 11. Not covered

- `inside=period` exact rule (§6.9) and `inside=startrail` (§6.10).
- `mult` with Im Z = 0, `bof61` ties, `>` vs `≥` at bailout, `<` vs `≤`
  for the 0.01 thresholds and the biomorph limit.
- Whether the 0.01 threshold of fmod/epscross can be changed, and where it is
  stored (offset 458 hypothesis, §5.7).
- Other fractal types, `bailoutest=` other than the default, potential, distance
  estimator, `logmap=`, rotated views.

---

## 12. Reproduction

- `tools/blackbox/coloring/render.sh OUTDIR NAME ARGS...` renders one case.
  `run-cases.sh OUTDIR [CASES]` renders every case of `cases.txt`, four at a
  time. `uniq.map` is the distinct-colour palette.
- Analysis: decode the GIF LZW, read corners, params, maxiter and the
  inside/outside/biomorph/decomp/bailout fields from `fractint001`
  (`gif-fractint-blocks.md` §2.1), iterate every pixel as in §2 in IEEE double
  precision, and apply §§3–8. A one-off Python checker did exactly this. Like
  all analysis scripts it isn't kept in the repo.
