<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Formula fractals (`type=formula`): what a formula computes

Status: **black-box derived.** Everything here comes from images that
XFractint (Debian `xfractint 20.4.10-5`, amd64) rendered from formula files we
wrote ourselves, compared pixel by pixel with our own calculations. Published
complex analysis (principal branches, the usual log/sqrt formulas for the
inverse functions) was used only as a source of hypotheses. No Fractint,
XFractint or Iterated Dynamics source code, headers, help text, documentation
or bundled formula files were consulted. See [CLEANROOM.md](../CLEANROOM.md).
Names and wording are ours.

Scope: what a parsed formula **means**: execution, values, operators,
functions, colouring and symmetry. How formula files are laid out, tokenised
and parsed (comments, separators, precedence, number syntax, entry headers,
errors) is the subject of [frm-syntax.md](frm-syntax.md). Where the two
overlap, that spec owns the syntax and this one the result.

Evidence: `corpus/xfractint/frm-semantics/`. Every case is a 400×300 batch
render with `type=formula formulafile=sem.frm passes=1 periodicity=0
viewwindows=2 map=uniq.map` plus the case's own parameters
(`tools/blackbox/frm-semantics/cases.txt`). The formulas are the entries of
`tools/blackbox/frm-semantics/sem.frm`; a case named `x` normally uses entry
`x` (the case line says which). A name like `quad` below means
`corpus/xfractint/frm-semantics/quad.gif`.

Labels: **Confirmed** = our calculation reproduces XFractint's pixels for
whole images (≥ 99.9 % of the 120,000 pixels; almost always 100 %), or a
probe shows the effect directly and unambiguously. **Hypothesis** = fits the
evidence, but other explanations remain. **Unknown** = not determined.

---

## 0. Method and notation

- Pixel (column `c`, row `r`) maps to `pixel = x + iy` as in
  [gif-fractint-blocks.md](gif-fractint-blocks.md) §2.2, using the corners
  **stored** in `fractint001`. XFractint may adjust corners that are close to,
  but not exactly, 4:3 (`corners=-200/199/-150/149` was stored as
  x ∈ [−199.83, 198.83]); always read the stored values.
- `M` is maxiter, `|w|²` is `Re(w)² + Im(w)²`, `T(x)` truncates toward zero.
- Colour indices are compared through `uniq.map` (distinct colours), so the
  duplicate-palette effect of [types.md](types.md) §1.3 does not arise. All our
  arithmetic is IEEE double (numpy complex128).
- Probe techniques used throughout:
  - **Truth map**: `z = pixel: z = z, COND` makes every pixel where COND is
    true inside (colour 0) and every other pixel colour 1 (§1.2). It shows
    the truth value of COND for every pixel value.
  - **Corner measurement**: init `v = EXPR`, condition
    `real(pixel) < real(v) && imag(pixel) < imag(v)`; the inside region is the
    quadrant below-left of `v`, so its corner reads `v` to one pixel step.
  - **Counter**: init `z = 0`, iteration `z = z + 1`, condition
    `real(z) < real(t)`; the escape count is the smallest integer k ≥ 1 with
    k ≥ t, so the colour reads a real number t (or a sum of 0/1 flags packed
    into t).
  - **Escape test of a value**: init `c = EXPR(pixel)`, `z = (0.3,0.2)` (or
    `(-0.1,0.35)`), iteration `z = z*z + c`, condition `|z| <= 4`. The
    non-real start value makes the image sensitive to sign, conjugation and
    branch errors in `c`. This is how every function and operator was checked
    on whole images.
- Exact zero rows/columns: `corners=-1.5625/1.5546875/-2.328125/2.34375`
  (view "F", steps 1/128 × 1/64), `corners=-3.125/3.109375/-4.65625/4.6875`
  ("F2", 1/64 × 1/32) and `corners=-200/199/-298/300` ("G", 1 × 2) contain
  x = 0 and y = 0 exactly, and F/F2 contain ±1 and ±i, so branch cuts and
  singular points are sampled.

---

## 1. Execution model

### 1.1 Per-pixel procedure (confirmed)

A formula has an initialisation part (before `:`) and an iteration part
(after it). For each pixel:

1. Run the initialisation statements once, in order.
2. Repeat, for k = 1, 2, …:
   1. run the iteration statements in order;
   2. the **value of the last statement** of the iteration part is the
      continue condition; if it is false (§1.2), the pixel escapes with
      **escape count e = k**;
   3. if k = M − 1 and the condition is still true, stop: the pixel is
      **inside**.

So the iteration part runs at most **M − 1** times. The start state (after
initialisation) is **never tested**.

Evidence:

- `quad` (`z = pixel: z = z*z + pixel, |z| <= 4`, default view): e as above,
  colour = e: 100 %. Colour e − 1: 60.8 %; e + 1: 12.7 %.
- `startt` (`z = 3*pixel: z = pixel, |z| <= 4`): pixels whose start value
  fails the test but whose first iterate passes are inside (100 %).
- `count` (counter with t = Re(pixel), corners 0/399, M = 300): column x = n
  gets colour n for 2 ≤ n ≤ 255, n − 255 for 256 ≤ n ≤ 299 (wrap, §1.3), and 0
  (inside) for n ≥ 300 = M. x = 0 and x = 1 give 1. 100 %.
- `carry` / `carry-m200` (an iteration counter `k` carried to the next pixel,
  §1.5): an inside pixel ran the iteration part **299 times at M = 300 and
  249 times at M = 250**.
- `quad-in-zmag`: `inside=zmag` matches 100 % with Z taken after M − 1
  iterations and 96.5 % with Z after M iterations (§1.4).
- The last statement decides, wherever the comparison sits:
  `nc2` (`|z| <= 4` first, then `z = z*z + pixel`) and `nc1` (no comparison
  at all) both behave as "continue while Re(z) ≠ 0" applied to the assigned
  value (all pixels inside, 100 % with that model); `nc4` (comparison followed
  by `w = 1`) is all inside; `nc5` (followed by `w = 0`) escapes everywhere
  at k = 1.

### 1.2 Truth of a value (confirmed)

Every value is complex. A value used as the condition (or by `if`, `&&`,
`||`, §5) is **false exactly when its real part equals 0**; the imaginary part
is ignored. A NaN real part counts as **true** (IEEE `x != 0`).

- `tz-g` (condition `z` = pixel on view G): false only on the column x = 0,
  for every y including y ≠ 0. `tre-g` the same; `tim-g` (`imag(z)`): false
  only on the row y = 0. 100 % each.
- `er7`: a value whose real part is NaN (`1e300*1e300 - 1e300*1e300`) as the
  condition keeps every pixel inside.

### 1.3 Colour index (confirmed)

With the default `outside=iter`, an escaped pixel gets
`W(e) = e` for e ≤ 255, else `((e − 1) mod 255) + 1`; inside pixels get the
`inside=` colour (default 0). This is types.md §1.2 with "e < M" replaced by
the M − 1 limit of §1.1 (the two are the same statement). Evidence: `count`
(e = 256 → 1, 299 → 44), `quad`, `quad-mi1000` (M = 1000, zoomed view
corners −0.80/−0.70/0.05/0.15: 99.978 %, the 26 misses are long chaotic
orbits that differ by rounding).

### 1.4 `inside=` / `outside=` modes: Z is the variable `z` (confirmed)

The modes of [coloring.md](coloring.md) work on formulas with:

- **N = e** (the escape count of §1.1, not e − 1 as for `type=mandel`);
- **Z = the value of the formula variable named `z`** at the end of the last
  iteration run: at escape for outside pixels, after M − 1 iterations for
  inside pixels. It does not matter which variable the condition tests.
- the orbit for orbit-based modes is the values of `z` after iterations
  1 … M − 1. The start value is excluded.

Evidence (all 100 %): `quad-out-real`, `quad-out-summ`, `quad-in-zmag`,
`quad-in-maxiter`, `quad-in-bof60` (minimum over iterations 1…M − 1; including
the start value gives 99.19 %). `zzv` iterates `w = w*w + pixel`, sets
`z = (0.5,-1.5)*w` and tests `|w| <= 4`: `zzv-out-real`, `zzv-out-imag` and
`zzv-in-zmag` match 100 % with Z = z and only 17–88 % with Z = w.

Not tested on formulas: decomposition, biomorph, `outside=mult/atan/fmod/tdis`,
the other inside modes (hypothesis: the same substitution works).

### 1.5 Variables persist across pixels (confirmed)

Variables are **not reset between pixels**. Every name keeps the value it had
at the end of the previous pixel (in calculation order) unless the formula
assigns it. At the start of the image every user variable (and `z`) is 0.
Predefined names that XFractint sets itself (`pixel`, `scrnpix`, `whitesq`,
…) are refreshed for every pixel. The parameters `p1`… are set once at
image start and then behave like ordinary variables (`ap1`); the same is
assumed (hypothesis) for `pi`, `e` and the other constants (§2.3).

- `mz` (init `v = z`, iteration `z = pixel`): every pixel sees the previous
  pixel's value of z (pixel (0,0) sees 0). 100 %.
- `zun` (`c = pixel: z = z*z + c` with z never initialised): every pixel
  escapes at k = 1, because z starts with the huge escaped value of the
  previous pixel.
- `ap1` (init `p1 = p1 + (0.001,0)`, condition `real(p1) < 50`): p1 grows by
  0.001 per pixel in row-major order (left to right, top to bottom); a
  sequential double-precision sum reproduces the image 100 %.
- `carry` row 0 column 0 reads 0 for the carried counter.

Consequence: a formula that reads a variable before assigning it depends on
the **order** pixels are computed in. With `passes=1` and no symmetry the
order is row-major. Not tested: the order under guessing (`passes=g`,
XFractint's default) or symmetry, where skipped pixels presumably do not
update anything.

### 1.6 `bailout=`, `float=`, periodicity

- `bailout=` has **no effect**: the formula's own condition decides
  (`quad-bo100` = `quad`, 100 %). **Confirmed.**
- `float=yes` and `float=no` give the same pixels as the default
  (`quad-float`, `quad-floatno`: 0 differences). **Confirmed** for this
  formula. Hypothesis: formulas are always evaluated in floating point.
- Periodicity checking applies: `quad-per1` (`periodicity=1`) turns 6 slowly
  escaping pixels (e = 132–138) inside. The rule is the one coloring.md §6.9
  could not pin down. **Unknown**; use `periodicity=0` to compare.

### 1.7 Math errors end the pixel (confirmed)

A complex division by zero, `log(0)` or `0^w` does not produce a value that
the formula can test. It makes the pixel **escape at the current iteration**,
even if the result is never used:

- `er1`, `er3`, `er4`: an unused `w = (1,1)/0`, `log(0)` or `0^2` in the
  initialisation gives every pixel colour 1 (escape count 1).
- `er5`: overwriting the bad result before it is used still gives colour 1.
- `er2`: `w = (1,1)/0` in the iteration part: colour 1 everywhere.
- `er9`: a division by zero that happens first at iteration 3 gives every
  pixel colour 3.
- `er8-g`: `1/real(pixel)` errors only at pixel 0 (colour 1 where the formula
  would otherwise be inside); elsewhere the image is the Mandelbrot set.
- Every probe that gave colour 1 in §§2–4 for "NaN-like" results
  (`k_div0` (1,1)/0, `k_div00` 0/0, `k_recip0` 1/(0,0), `k_pow02` 0^2,
  `k_pow00` 0^0, `k_pow0c` 0^(0.5,0.5), `k_log0`, `k_cotan0` cotan(0),
  `k_tan` tan(1.5707963267948966)) is, we assume, this effect: the probe
  cannot distinguish an error from a value that fails every comparison, but
  `er1`–`er5` show the error behaviour directly for ÷ 0, log 0 and 0^2.
- Pixel 0 in `o-pw2`, `o-pw3`, `o-pwh`, `o-pw25`, `o-pwc`, `o-pw0`
  (`pixel^…`) has colour 1 while its neighbours are inside, as 0^w predicts.
  (In `o-div`/`o-divm` its neighbours escape at once too, so it tells
  nothing.)

Overflow is **not** an error: `1e300*1e300` is +∞ and `∞ − ∞` is NaN
(`k_huge`, `er6`), and both are ordinary values (§1.2). Which other operations
raise the error (sqrt of negative numbers does not; `k_sqrtm1` is fine) is
otherwise **unknown**. The tan(π/2) case suggests that tan divides by a
denominator that is exactly 0 there (hypothesis).

---

## 2. Predefined names

### 2.1 Values (confirmed unless marked)

| Name | Value | Evidence |
|---|---|---|
| `pixel` | the pixel's coordinate (§0); reset every pixel | all images |
| `z` | not set by XFractint: 0 at image start, then carried (§1.5) | `zv`, `mz` |
| `p1` | (P1, P2) from `params=`, default 0 | `mp1`, `mp-params` |
| `p2` | (P3, P4) | `mp2`, `mp2-params` |
| `p3` | (P5, P6) | `mp3-params` |
| `p4` | (P7, P8) | `mp4-params` |
| `p5` | (P9, P10) | `mp5-params` |
| `pi` | (π, 0), double precision | `n_pi`: 3.14159265338 < π ≤ 3.14159265363 |
| `e` | (e, 0) | `n_e`: 2.71828182832 < e ≤ 2.71828182857 |
| `maxit` | (M, 0) | `mmaxit` (150), `mmaxit-500` (500), `c_maxit` (1000) |
| `scrnmax` | (image width, height) = (400, 300) | `mscrn-big`, `c_scrr`, `c_scri` |
| `scrnpix` | (column, row) of the pixel, 0-based, row 0 at the top | `mscrnpix`, `mscrnpix-big` 100 % |
| `whitesq` | ((row + column) mod 2, 0) | `mwhitesq` 100 % |
| `ismand` | (1, 0) in these renders | `mismand` |
| `center` | ((xmin+xmax)/2, (ymin+ymax)/2) | `mcenter` (0,0), `mcenter-v` (1, −0.5), `c_cenr`/`c_ceni` |
| `magxmag` | (mag, xmag) of par.md §5.2: mag = 2/(ymax − ymin) for unrotated views | `mmagxmag` (1/6, 1), `mmagxmag-v` (2/3, 1), `c_magr`/`c_magi` (0.5, 1.5) |
| `rotskew` | (rotation, skew) in degrees, as in `center-mag=` | `mrotskew` (0,0), `c_rotr`/`c_roti` (30, 10) |
| `lastsqr` | read as ≈ 0 in every probe | `mlastsqr`, `lsq1`, `lsq2`, `lsq4`, `lsq5` |
| `rand` | a different finite, non-zero value per pixel | `mrand`, `k_rand`, `rnd1`, `rnd2` |

Notes:

- The `p` and `scrn*` values were read to one pixel step (0.02–4 units)
  with corner measurements; integers and the 0.1-multiples given in
  `params=` fit every measurement. The `c_*` cases are counter probes whose
  `z` is not initialised, so only their **first pixel** is meaningful (the
  later pixels continue the carried count, §1.5).
- `lastsqr` (**hypothesis: always 0 in formulas**): computing `|z/2|` or
  iterating did not change it (`lsq1`/`lsq4`: `real(lastsqr) < 1` true
  everywhere although |pixel/2|² reaches 6.25; `lsq5`: its imaginary part is
  within ±0.5). Its intended meaning is unknown.
- `ismand` (hypothesis): a flag that may change when XFractint switches
  between Mandelbrot and Julia views; only the value 1 was observed.
- `rand` and `srand` (**unknown**): `rand` differs between pixels and its
  real part (`rnd1`) and imaginary part (`rnd2`) were larger than 40,000 in
  every pixel but one; `log(rand)` and `srand(1)` gave values outside every
  measured range. Repeated renders with the same `rseed=` differed in 1–2
  pixels (`rnd1-s1`, `rnd1-s1b`, `rnd1-s1c`). The generator was not
  identified, so formulas using `rand` cannot be reproduced pixel for pixel.

### 2.2 `params=` mapping (confirmed)

`params=a/b/c/d/…` gives `p1 = a + bi`, `p2 = c + di`, `p3`, `p4`, `p5` from
the 5th–10th values, in pairs. Missing values are 0. Stored as in
gif-fractint-blocks §2.1 (P1, P2 at 44/52, P3, P4 at 270/278, P5–P10 at 354…).

### 2.3 Assigning to predefined names (confirmed)

All of these are ordinary variables that can be assigned:

- `pixel`: `apix` (init `pixel = 2*pixel`) is the Mandelbrot set at twice
  the scale (100 %); `apx2` (`pixel = pixel + 1` in the iteration part) keeps
  growing during one pixel (100 %); `apix2` shows that the next pixel gets a
  fresh `pixel` (100 %).
- `p1` (and so presumably `p2`…): keeps the assigned value for later pixels
  (`ap1`, §1.5).
- `pi`, `e`: `api` (`pi = 3`) and `ae` (`e = 2`) take the new values (100 %).
  Hypothesis: the changed constant also carries to later pixels.

---

## 3. Operators

Arithmetic is complex double precision.

| Operator | Meaning | Status, evidence |
|---|---|---|
| `a + b`, `a - b`, `a * b` | complex sum, difference, product | confirmed: `quad`, `o-sub`, all images |
| `-a` | negation | confirmed: `o-neg` 100 % |
| `a / b` | complex quotient; b = 0 is a math error (§1.7) | confirmed: `o-div`, `o-divm` 100 % |
| `a ^ b` | `exp(b · log a)` with the principal log (§4); a = 0 is a math error | confirmed on images, see below |
| `\|a\|` | **`Re(a)² + Im(a)²`** (squared modulus) as (value, 0) | confirmed: `modb` 100 % (radius 2 for `\|z\| < 4`; plain modulus 62.5 %); `mmod` |
| `a < b`, `<=`, `>`, `>=` | compare **real parts only**; result (1, 0) or (0, 0) | confirmed: `clt-g` … `cge-g`, `mcmplt` ((3,1) < (2,5) is false), `mtrue`, `mfalse` |
| `a == b`, `a != b` | compare **real parts only** | confirmed: `ceq-g` true only on the column x = 10, whatever the y |
| `a && b`, `a \|\| b` | logical and/or of the truth values (§1.2); result (1, 0) / (0, 0) | confirmed: `land`, `lor`, `mand`, `mor`, `n_and2` ((2,0) && (0,5) = 0), `n_or2` ((0,3) \|\| 0 = 0), `n_and3` |
| `!` | not an operator (error "Undefined Operator") | `lnot` |

Power details:

- `o-pw2`, `o-pw3`, `o-pwh` (`^0.5`), `o-pw25`, `o-pwc` (`^(0.5,0.5)`),
  `o-pwcb` (`(0.5,0.5)^pixel`), `o-pwpp` (`pixel^pixel`), `o-pw0`: 100 % except
  pixel 0 (§1.7), using `exp(b·log a)`.
- Integer exponents are **not** done by repeated multiplication: `(-2)^2` has
  a small negative imaginary part, `(-2)^3` and `(0,1)^2` have non-zero
  imaginary/real rounding residue, while `(0,1)*(0,1)` and `sqr((0,1))` are
  exactly (−1, 0) and `(2,0)^2`, `(2,0)^0.5` are exactly real (`q_pm22`,
  `q_pm23`, `q_pi2`, `q_ii`, `q_sqri`, `q_p22`, `q_p2h`). This fits a polar
  or exp/log evaluation (**hypothesis**: `r^b·(cos bθ + i sin bθ)` for real b);
  it changes pixels only through rounding.
- `(-1)^0.5` = (≈0, 1) and `(-4)^(0.5,0)` = (≈0, 2) (`mpowneg`, `mpowneg2`):
  principal branch. `(0,1)^(0,1)` = e^(−π/2) ≈ 0.208 (`mxpow`).
- `x^-1` is rejected (error 32, `o-pwm1`); syntax spec territory.
- The 1e-10 flag bits in the `q_*` probes gave inconsistent results and are
  ignored; we suspect a literal-syntax issue (see frm-syntax.md).

---

## 4. Functions

### 4.1 Built-in functions (confirmed)

Each was checked on the whole image twice: `f-NAME` (c = NAME(pixel), view
F, start (0.3,0.2)) and `g_NAME` (c = 0.5·NAME(pixel), view F2, start
(−0.1,0.35)). "Informative" is the share of pixels with colour > 2; the rest
escape at once and say little.

| Name | Definition that matches (z = x + iy) | f- / g_ match (informative %) |
|---|---|---|
| `sin`, `cos`, `tan` | standard complex functions | 100 / – (25, 19, 72); `g_tan` 100 (6) |
| `sinh`, `cosh`, `tanh` | standard | 100 (46, 32, 35); `g_tanh` 100 (45) |
| `cotan` | cos z / sin z | 100 / 100 (72, 6) |
| `cotanh` | cosh z / sinh z | 100 / 100 (44, 45) |
| `cosxx` | **conj(cos z)** (cosine with the imaginary part negated) | 100 / 100 (19, 20); plain cos on `f-cosxx` 89 % |
| `exp` | e^z | 100 / 100 (24, 13) |
| `log` | principal: ln\|z\| + i·arg z, arg ∈ (−π, π]; arg = +π on the negative real axis (y = +0) | 100 / 100 (16, 61) |
| `sqr` | z·z | 100 (15) |
| `sqrt` | principal: √\|z\|·(cos θ/2 + i sin θ/2), θ = arg z | 100 / 100 (25, 91) |
| `abs` | (\|x\|, \|y\|) | 100 / 100 (16, 24) |
| `cabs` | (√(x² + y²), 0), the plain modulus (unlike `\|z\|`) | 100 / 100 (19, 23); \|z\|² gives 55 % |
| `conj` | (x, −y) | 100 / 100 |
| `real` | (x, 0) | 100 / 100 |
| `imag` | (y, 0) (a real number, not (0, y)) | 100 / 100; (0, y) gives 48 % |
| `flip` | (y, x) | 100 / 100 |
| `floor`, `ceil`, `trunc` | applied to x and y separately | 100 / 100 |
| `round` | floor(t + 0.5) on x and y separately | 100 / 100; half-away 99.15 %, half-even 99.10 % |
| `asin` | principal (numpy `arcsin`, = −i·log(iz + √(1 − z²))) | 100 / 100 (36, 87) |
| `asinh` | principal (= log(z + √(z² + 1))) | 100 / 100 (38, 66) |
| `acos` | **−i·log(z + √(z² − 1))** with principal log and sqrt (not the principal arccos) | 100 / 100 (21, 64); principal arccos 76 % / 59 % |
| `acosh` | **log(z + √(z² − 1))** (not the principal arccosh) | 100 / 100 (6, 68); principal 94 % / 62 % |
| `atan` | principal, except on the cut: for x = 0, y > 1 the real part is **−π/2** (principal: +π/2); for x = 0, y < −1 it is −π/2 as usual | 100 / 100 (50, 59); principal 99.89 % / 99.80 % |
| `atanh` | principal | 100 / 100 (61, 56) |

Here √ is the principal square root of the complex value `z*z − 1` etc., as
numpy computes it from IEEE operations (the sign of a zero imaginary part
matters on the real axis; our grids produced +0 for y = 0 and −0 for products
like 2·x·0 with x < 0, and the images matched).

Singular points (**confirmed** for the four points, both views): `atan(±i)`
returns **±i** and `atanh(±1)` returns **±1**, i.e. the argument itself
(`f-atan`, `g_atan`, `f-atanh`, `g_atanh` 100 % only with this rule; corner
measurements `n_atani-b` (0, ≈1), `n_atanmi-b`, `n_atanh1-b` (≈1, 0),
`n_atanhm1-b`). `cotan(0)`, `log(0)` and `tan(π/2)` are math errors (§1.7).

Names that are **not** formula functions (error 5 "Undefined Function"):
`cot`, `coth`, `ident`, `zero`, `one`, `recip` (`f-cot`, `f-coth`,
`f-ident`, `f-zero`, `f-one`, `f-recip`), and any unknown name (`bogus`).
The four last ones exist as values for `function=` (§4.2).

`srand(…)` parses; its value is unknown (§2.1).

### 4.2 `fn1`…`fn4` (confirmed)

`fn1`…`fn4` call the functions selected by `function=f1/f2/f3/f4`; they are
stored as the indices at `fractint001` 218–221
(gif-fractint-blocks §6.2). Without `function=` they are **sin, sqr, sinh,
cosh** (stored `00 06 02 03`).

- Defaults: `f-fn1` = sin, `f-fn2` = sqr, `f-fn3` = sinh, `f-fn4` = cosh,
  `mfn1` (fn1(1) ≈ 0.84): 100 %.
- `function=cos/exp/log/tan` (`f-fn1-cos` … `f-fn4-tan`, stored 9 4 5 10),
  `function=ident/zero/one/recip` (`f-fn1-ident` … `f-fn4-recip`, stored
  8 16 30 7), `function=cotan/cotanh/sqrt/acos` (`f-fn1-cotan`,
  `f-fn4-acos`, stored 12 13 25 19), `fnx` / `fnx-fn` (`fn1(z)*fn2(z)`):
  100 % with the §4.1 definitions and ident(z) = z, zero(z) = 0,
  one(z) = 1, recip(z) = 1/z. (`recip(0)` gave colour 1 like a math error
  would; not distinguished.)

---

## 5. Statements

### 5.1 Assignment

`name = expression` stores the value; the statement's value is the assigned
value (that is what §1.1's "last statement" uses, `nc1`). Assignment to
predefined names is allowed (§2.3). Values persist (§1.5). **Confirmed.**

### 5.2 `if` / `elseif` / `else` / `endif` (confirmed)

The branch is taken when its condition is true by §1.2 (real part ≠ 0).
`elseif` conditions are tried in order, `else` runs if none was true.

- `ifz` (`if (z)` with z = pixel): true everywhere except the column x = 0.
- `ifim` (`if (imag(z))`): false only on the row y = 0.
- `ifel`: a three-way `if/elseif/elseif/else` chain reproduces 100 %.

The value of an `if` block as the last statement was not tested; all our
probes put a plain expression last.

---

## 6. Symmetry qualifiers

An entry header may carry a qualifier in parentheses, `name(XAXIS) {`. The
qualifier makes XFractint copy mirrored pixels instead of computing them,
**whether or not the formula really is symmetric**. Our probes used
deliberately asymmetric formulas, so every copy is visible. The qualifier is
not stored in the GIF (offset 124 stays 999). Command-line `symmetry=`
overrides it: `sqx-symnone` (XAXIS entry, `symmetry=none`) copies nothing,
`sqx-symy` (`symmetry=yaxis`, stored 2) copies columns, and `quad-off-symx`
(no qualifier, `symmetry=xaxis`, stored 1) copies rows. **Confirmed.**

### 6.1 Copy rule

Use the row/column rule of types.md §1.6 with `S = round(2·ymax/Δy)` and
`T = round(−2·xmin/Δx)`, where an exact half rounds **up**
(`hsh-*-w`: −2·xmin/Δx = 332.5 exactly, T = 333 matches, 332 does not).
A row r is **copied** if `0 ≤ S − r < r`; a **band** row if it is not copied
but `0 ≤ S − r < H`; otherwise a **free** row. Columns analogously
(`0 ≤ T − c < c` copied).

| Qualifier | Copies | Evidence (all 0 pixels different from the model) |
|---|---|---|
| `XAXIS` | rows: (r, c) ← (S − r, c) | `sq-XAXIS` |
| `YAXIS` | columns: (r, c) ← (r, T − c) | `sq-YAXIS` |
| `XYAXIS` | rows and columns independently (types.md "xy") | `sq-XYAXIS`, `hsh-XYAXIS-{c,w,o,n,v}` |
| `ORIGIN` | for copied rows, (r, c) ← (S − r, T − c) where 0 ≤ T − c < W (types.md "origin") | `sq-ORIGIN`, `hsh-ORIGIN-{c,w,o,n,v}` |
| `PI_SYM` | §6.2 | `hsh-PI_SYM-{c,w,o,n,v}` |
| `XAXIS_NOPARM`, `YAXIS_NOPARM`, `XYAXIS_NOPARM`, `ORIGIN_NOPARM`, `PI_SYM_NOPARM` | as without `_NOPARM` when the parameters are 0; **nothing** when P1, P2 or P3 is non-zero | `sp-*-00`, `-r` (P1 = 0.1), `-i` (P2 = 0.1), `-p2` (P3 = 0.1) |
| `XAXIS_NOREAL` | as XAXIS unless a **real** part is non-zero (P1 or P3 non-zero → nothing; P2 alone non-zero → copies) | `sp-XAXIS_NOREAL-*` |
| `XAXIS_NOIMAG` | as XAXIS unless an **imaginary** part is non-zero (P2 non-zero → nothing; P1 or P3 → copies) | `sp-XAXIS_NOIMAG-*` |
| `NOSYM`, unknown words (`FOO`) | nothing, no error | `sq-NOSYM`, `sq-FOO` |
| plain `XAXIS`, `ORIGIN` with non-zero params | still copy | `sp-XAXIS-*`, `sp-ORIGIN-*` |

The `_NOPARM`/`_NOREAL`/`_NOIMAG` tests used P1–P3 only; P4 and higher are
**untested** (hypothesis: they count the same way). Rotated views were not
tested.

### 6.2 `PI_SYM` (confirmed on five views)

`PI_SYM` combines the ORIGIN-style mirror with a period of π in x. Let
`P = floor(π · W / (xmax − xmin))` (columns per period; 104, 125, 314 and 502
in our views). Build the image row by row:

1. **Free rows** (mirror partner outside the image): compute columns
   0 … P − 1, then column c ≥ P is a copy of column c − P of the same row
   (so the row repeats with period P).
2. **Band rows**: column c with `0 ≤ T − c < c` is a copy of column T − c of
   the same row. The other columns split into two classes: *sources*
   (`0 ≤ T − c < W`) and *unpaired* columns (partner outside the image). In
   each maximal run of consecutive columns of one class, starting at column
   a, the columns c ≥ a + P copy column c − P; the rest are computed. Then
   the mirrored columns take their (final) source values.
3. **Copied rows**: for `0 ≤ T − c < W`, (r, c) ← final (S − r, T − c). The
   remaining (unpaired) columns form runs treated as in step 2.

Match: `hsh-PI_SYM-c` (default view), `-w` (−5…7 × −3…6), `-o`
(−2.2…1.8 × −1.3…1.7), `-n` (−1…1.5 × −0.5…1.375), `-v`
(−7…3 × −2…5.5): 100 % each (`hsh` is a smooth, asymmetric 200-colour
pattern; `hsh-NONE-*` are the computed references). Simpler readings fail:
plain XY copying differs in 12,032–79,077 pixels, and periodic copying across
whole rows differs in 0.9–46 thousand pixels. In the centred view
(`-c`, and `sq-PI_SYM` there) PI_SYM equals XYAXIS except for the
period copies, which XY copying then overwrites. Other formulas of P (e.g.
round(π/Δx)) fit some views but not all. This description is our model of
the result; how XFractint organises it internally is unknown, and views where
several periods fit inside a band-row run were only covered by `-w`.

### 6.3 Interactions (unknown)

Not tested for formulas: symmetry together with `outside=`/`decomp`
(coloring.md §13 found that some modes turn copying off for mandel), with
`passes` other than 1, and its effect on variables carried between pixels
(§1.5): copied pixels are presumably not computed, so carried state skips
them.

---

## 7. Residuals and precision

- All whole-image comparisons above are 100 % except: `quad-mi1000` 99.978 %
  (deep, long orbits; rounding), `quad-per1` 99.995 % (periodicity on, §1.6),
  and the single pixel 0 of `o-pw*` before §1.7 is applied.
- Where evaluation order matters only in the last bit (power, sqrt via polar
  form, division formula), we could not tell variants apart from pixels; any
  IEEE-correct evaluation reproduced our images. Deep zooms and maxiter in the
  thousands will show isolated differences, as for the built-in types
  (types.md §1.7, coloring.md §10).

---

## 8. Summary for implementers

1. Per pixel: refresh `pixel`, `scrnpix`, `whitesq` (and presumably `rand`);
   keep every other variable from the previous pixel (all 0 at image start;
   `p1`…`p5`, `pi`, `e`, `maxit`, `scrnmax`, `center`, `magxmag`, `rotskew`,
   `ismand` = 1, `lastsqr` = 0 set at image start).
2. Run the init part. A math error (complex ÷ 0, log 0, 0^w) → escape, count 1.
3. For k = 1 … M − 1: run the iteration part; a math error → escape with
   count k; otherwise if Re(last statement value) == 0 → escape with count k.
   If the loop finishes, the pixel is inside.
4. Colour with the coloring.md modes, N = k and Z = variable `z`
   (default `outside=iter`: W(k); default inside 0). Ignore `bailout=`.
5. Comparisons and `==`/`!=` use real parts only and return (1,0)/(0,0);
   `|w|` is the squared modulus; functions as §4.1 (note `cosxx`, `acos`,
   `acosh`, the `atan` cut and the `atan`/`atanh` singular points).
6. Apply the entry's symmetry qualifier (§6), unless `symmetry=` is given.

## 9. Not covered / unknown

- `rand`, `srand` (generator unknown), `lastsqr` meaning, `ismand` changes.
- Periodicity-checking rule (shared with coloring.md §6.9).
- Pixel order under guessing passes and symmetry, and its effect on carried
  variables.
- Which operations other than complex ÷ 0, log 0, 0^w (and functions built on
  them) count as math errors; what the error does in the middle of
  `if`/`elseif` chains.
- Colouring modes beyond those in §1.4, `bailoutest=`, rotated views,
  `XAXIS_NO*` with P4 and higher.

## 10. Reproduction

- `tools/blackbox/frm-semantics/render.sh OUTDIR NAME ARGS...` renders one
  case; `run-cases.sh OUTDIR [CASES]` renders all of `cases.txt`, four at a
  time. Probe formulas: `sem.frm` (our own).
- Analysis: decode each GIF, read corners/params/maxiter/inside/outside from
  `fractint001`, evaluate the probe formula for every pixel in IEEE double
  according to §§1–6, compare colour indices. The throwaway Python scripts are
  not in the repo (as for the other black-box specs).
