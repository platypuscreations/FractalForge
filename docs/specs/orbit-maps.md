<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# 2D orbit-plotting map types

Status: **black-box derived.** Everything here comes from images and blocks
that XFractint (Debian `xfractint 20.4.10-5`, amd64, run under emulation in
`fractalforge-blackbox`) saved, compared with our own simulations. Published
mathematics (the Hénon map, Martin's "hopalong" map and its relatives, the
Field–Golubitsky symmetric icons, the Pickover "latoocarfian" and "popcorn"
maps, the KAM-torus map, inverse iteration for Julia sets) was used only as a
hypothesis and kept only where XFractint's pixels confirm it. No Fractint,
XFractint or Iterated Dynamics source code, headers, help text or
documentation was consulted (see [CLEANROOM.md](../CLEANROOM.md)). Names and
notation are our own.

Evidence: `corpus/xfractint/orbit-maps/` (command lines in its `cases.txt`,
`resume/cases.txt` and `res800/cases.txt`). A name like `henon-mi10` means
`corpus/xfractint/orbit-maps/henon-mi10.gif`. All images are 400×300
(`viewwindows=2`) unless they are in `res800/`. Scripts:
`tools/blackbox/orbit-maps/` (`verify.py` reproduces every number in §12).

Labels: **Confirmed** = our simulation reproduces whole images (normally
100 % of the 120,000 pixels) or a one-parameter change shows the effect
directly. **Hypothesis** = fits the evidence, other explanations remain.
**Unknown** = not determined.

Other specs used: [gif-fractint-blocks.md](gif-fractint-blocks.md) (block
layout, `fractint001` offsets), [types.md](types.md) §1.2 (escape-count
colours) and §1.3 (duplicate palette entries), [fractint-resume.md](fractint-resume.md)
(the 392-byte work list).

---

## 1. Types covered

All names below were accepted by this build. The type number is
`fractint001`[10]; corners and params are what a render without `corners=` /
`params=` stores (the `-def` files).

| Type | # | Family (§) | Default corners xmin, xmax, ymin, ymax | Default params P1…P6 | `-def` match |
|---|---|---|---|---|---|
| henon | 88 | single orbit, time colour (§3) | −1.399999976158142, 1.399999976158142, −0.5, 0.5 | 1.4, 0.3 | 100 % |
| hopalong | 120 | single orbit, time colour (§3) | −2, 3, −1.625, 2.625 | 0.4, 1, 0 | 100 % |
| martin | 122 | single orbit, time colour (§3) | −32, 32, −24, 24 | 3.14 | 100 % |
| gingerbreadman | 90 | single orbit, time colour (§3) | −4.5, 8.5, −4.5, 8.5 | −0.1, 0 | 100 % |
| chip | 166 | single orbit, time colour (§3) | −760, 760, −570, 570 | −15, −19, 1 | 100 % |
| quadruptwo | 167 | single orbit, time colour (§3) | −82.93364969889323, 112.27488199869792, −55.76382827758789, 90.64257049560547 | 34, 1, 5 | 100 % |
| threeply | 168 | single orbit, time colour (§3) | −8000, 8000, −6000, 6000 | −55, −1, −42 | 100 % |
| icons | 159 | single orbit, hit count (§4) | −2, 2, −1.5, 1.5 | −2.34, 2, 0.2, 0.1, 0, 3 | 100 % |
| latoocarfian | 171 | single orbit, hit count (§4) | −2, 2, −1.5, 1.5 | −0.966918, 2.879879, 0.765145, 0.744728 | 100 % (see §4.3) |
| kamtorus | 40 | family of orbits, time colour (§5) | −1, 1, −0.75, 0.75 | 1.3, 0.05, 1.5, 150 | 100 % |
| popcorn | 61 | one orbit per pixel, row colour (§6) | −3, 3, −2.25, 2.25 | 0.05, 0, 3 | 100 % |
| popcornjul | 105 | escape time, **not** an orbit plot (§7) | −3, 3, −2.25, 2.25 | 0.05, 0, 3 | 99.99 % |
| dynamic | 139 | one orbit per grid point (§8) | −20, 20, −20, 20 | 50, 0.1, 1, 3 | 100 % |
| mandelcloud | 146 | one orbit per grid point (§9) | −2.5, 1.5, −1.5, 1.5 | 50 | 99.40 % (hypothesis) |
| julia_inverse | 145 | inverse iteration, hit count (§10) | −2, 2, −1.5, 1.5 | −0.11, 0.6557, 4, 1024 | 98.89 % (hypothesis) |

Default maxiter is 150 for all of them; the function slots
(`fractint001`[218–221]) hold the usual defaults 0, 6, 2, 3 (sin, sqr, sinh,
cosh; gif-fractint-blocks §6.2), which matters for latoocarfian, popcorn,
popcornjul and dynamic, the only ones here that use them.

Names that were **rejected** as unknown by this build: `icon`, `icon3d`,
`iconfp`, `gingerbread`, `henonfp`, `hopalongfp`, `martinfp`, `kamtorusfp`,
`popcornfp`, `dynamicfp`, `gingerbreadfp`, `mandel_inverse`, `julia_inv`,
`inverse`, `inversejulia`, `cloud`, and the guesses `standard`, `standardmap`,
`tent`, `ikeda`, `duffing`, `lozi`, `tinkerbell`, `clifford`, `dejong`,
`gumowski`, `mira`, `bedhead`, `svensson`, `zaslavsky`, `logistic`,
`chirikov`, `baker`, `arnold`, `kaplan`, `symicon`, `symmetricicon`, `orbit`,
and `*3d` variants of the types above other than `kamtorus3d`.

Other accepted names noticed while probing and **not covered** here:
`kamtorus3d` (42), `lorenz`, `pickover` (3D attractors, another spec),
`bifurcation` (32), `biflambda`, `bifmay`, `bifstewart`, `bif+sinpi`,
`bif=sinpi` (bifurcation diagrams), `lyapunov` (123), `cellular` (142),
`frothybasin` (154), `plasma` (7), `diffusion` (91) and `ant` (the last never
finished in batch mode).

---

## 2. Common machinery

### 2.1 What the program draws on

The image starts at colour 0 everywhere (background). Orbit types write
colour indices into pixels; a later write replaces an earlier one, except for
the hit-count types (§4), which read the pixel first. Pixels never hit stay 0.

### 2.2 Saved parameters

Nothing type-specific is added to `fractint001`: params 1–4 at 44, 52, 270,
278 and 5–10 at 354…394, maxiter at 8/408, inside at 104, function slots at
218–221, exactly as for escape-time types. The stored corner values are the
ones used for the pixel mapping, including the single-precision-looking
henon x corners (±1.399999976158142) and quadruptwo's odd defaults.

### 2.3 Point → pixel (confirmed)

For a point (x, y) with stored corners xmin, xmax, ymin, ymax and an image of
W × H pixels:

    fc = (x − xmin) · (W − 1) / (xmax − xmin)
    fr = (ymax − y) · (H − 1) / (ymax − ymin)
    column = trunc(fc), row = trunc(fr)        (truncation toward zero)

The point is drawn if 0 ≤ column < W and 0 ≤ row < H. Because truncation is
toward zero, a point with −1 < fc < 0 lands in column 0 (and −1 < fr < 0 in
row 0). Evidence: with `floor` the martin default image had exactly 2
mismatches, both in column 0 from points with fc ≈ −0.3; truncation toward
zero gives 100 % (`martin-def`). Rounding to nearest, or `/W` scaling,
reproduced only 52–74 % of henon's lit pixels. Rotated/skewed views were not
tested.

The same rule holds at 800 × 600 (`res800/*`, 100 %).

### 2.4 Arithmetic (confirmed where stated)

- All maps run in IEEE double precision. `float=yes` gives the same pixels
  (`henon-float` 100 %; hopalong and martin renders with `float=yes` had
  pixels identical to the defaults during analysis).
- The orbits are chaotic, so **every rounding difference changes the image**
  after a few hundred iterations. The operation orders written in this spec
  are the ones that reproduce XFractint bit for bit (e.g. henon computed with
  `a·(x·x)` instead of `(a·x)·x` leaves XFractint's orbit within ~250 points).
- Square roots and the basic operations are exactly rounded, so henon,
  hopalong, gingerbreadman, icons and kamtorus reproduce with any IEEE
  implementation.
- **Fixed sin/cos/log/atan** (martin, chip, quadruptwo, threeply): XFractint's
  results equal glibc 2.41's `sin`, `cos`, `log` and `atan` on x86-64 (we ran
  our map inside the container with Perl, which calls the same libm): 100 %.
  With correctly rounded sin, martin's orbit leaves XFractint's after 4,717
  points, and with macOS libm after 820. A renderer that must match these
  images exactly needs results identical to that libm. Squaring is `v·v`
  (Perl's `**2`, which goes through `pow`, broke chip after 165 points).
- **Function-slot sin/cos** (latoocarfian; probably popcorn and dynamic too):
  the value is the exact function value rounded **to a 64-bit significand
  first and then to double** (what an x87 instruction followed by a store
  would give). With that, latoocarfian matches 100 % up to 40,961 points
  (`latoocarfian-ssss-mi40`); correctly rounded sin already fails before
  10,241 points and glibc sin after 842. Beyond ~41,000 points the orbits
  still separate (§4.3), so this is the best model found, not an exact one.
- **Function-slot tan** (popcorn, popcornjul): best fit is
  `tan(v) = sin(2v) / (cos(2v) + 1)` evaluated in doubles (the complex tangent
  formula with zero imaginary part). It reduced popcorn mismatches near the
  poles of tan from 111 to 41 (`popcorn-st-mi10`). Hypothesis.
- **Overflow in function slots** (popcorn with the default sinh/cosh): where
  the double result would overflow, XFractint behaves as if the function
  returned **1.0** (any value in about 0.95…1.2 fits; 0.9 or 0 does not).
  With this rule `popcorn-def` and `popcorn-mi10` are 100 %, without it
  81.9 % (all mismatches in the left and right columns, where sinh(x +
  cosh(3x)) overflows). Hypothesis for the exact value.

### 2.5 Options with no effect

For the orbit types, `passes=` (1, 2, b, g), `symmetry=none` and
`periodicity=0` change nothing (`henon-pass1`, `henon-sym-none`; henon with
`periodicity=0`, and popcorn with passes 1, 2, b and `symmetry=none`, gave
identical images during analysis).
`outside=` has no effect (`henon-out7`). `inside=` changes colours (§3.3, §5,
§8) except for popcorn and the hit-count types. `maxiter` is ignored by
kamtorus (`kamtorus-mi10` = `kamtorus-def`) and julia_inverse; the others use
it as described per type. None of the models uses random numbers;
`rseed=` did not change julia_inverse (`julia_inverse-rs1`), the only type
where we checked.

---

## 3. Single orbit with time colouring

Types: henon, hopalong, martin, gingerbreadman, chip, quadruptwo, threeply.

### 3.1 Algorithm (confirmed)

    (x, y) = start point                       (table in §3.2)
    plot (x, y) with colour C(0)
    for j = 1 … 1024 · maxiter:
        (x, y) = step(x, y)
        plot (x, y) with colour C(j)           (if it is inside the image, §2.3)

There is **no escape test** and no other stopping rule: an orbit that leaves
the window keeps being iterated (martin's default orbit wanders off after
~21,700 points and never comes back; the remaining iterations draw nothing).
The point count is exactly `1024 · maxiter` iterates: the henon fits for
maxiter 10, 100 and 150 have a unique best length (10,240, 102,400 and
153,600 iterates), and 1,024,000 reproduces maxiter 1000. `chip-mi2z` shows
that the start point is drawn too (without it one pixel is missing).

### 3.2 Maps and start points (confirmed)

`a, b, c` are P1, P2, P3. `s = sign(x)`: +1, −1, or **0 when x = 0** (+1 at
x = 0 would change the first iterate from the start (0, 0) of hopalong with
c ≠ 0, chip, quadruptwo and threeply, and break their 100 % matches).
Operations are evaluated left to right as written; `ln` is the natural log.

| Type | x′ | y′ | Start |
|---|---|---|---|
| henon | (1 + y) − (a·x)·x | b·x | (1, 1) |
| hopalong | y − s·√\|b·x − c\| | a − x | (0, 0) |
| martin | y − sin(x) | a − x | (0, 0) |
| gingerbreadman | (1 − y) + \|x\| | x | (P1, P2) |
| chip | y − (s·cos(L₁·L₁))·atan(L₂·L₂) | a − x | (0, 0) |
| quadruptwo | y − (s·sin(L₁))·atan(L₂·L₂) | a − x | (0, 0) |
| threeply | y − s·\|(sin(x)·cos(b) + c) − x·sin((a + b) + c)\| | a − x | (0, 0) |

with `L₁ = ln|b·x − c|`, `L₂ = ln|c·x − b|`.

- henon ignores P3 and up; the start is fixed (`henon-p13`, a = 1.3, is
  100 %). Starting from (0, 0) instead fails within the first iterations.
- hopalong P3 = c is used (`hopalong-p3`, `hopalong-p`).
- gingerbreadman starts at its params: `gingerbreadman-p` (0.5/1.5) 100 %.
- chip: the squares are inside the cosine and the arctangent. Squaring
  the cos and atan results instead gives a visibly different, much smaller
  attractor.
- Parameter roles of chip, quadruptwo and threeply confirmed with
  non-default values (`chip-p`, `quadruptwo-p`, `threeply-p`, all 100 %).
- Behaviour when `b·x − c` or `c·x − b` is exactly 0 (log of 0) is unknown.

### 3.3 Colour (confirmed)

    C(j) = ((c0 − 1 + ⌊(j + 1) / 1001⌋) mod 255) + 1

So the start and the first 999 iterates (j = 0…999) get c0, the colour then
goes up by one every 1001 points (changes at j = 1000, 2001, 3002, …), and
after 255 it continues at 1. Colour 0 is never used. Evidence: boundaries in
`henon-mi10` (colour 2 for j ≤ 955 observed, 3 from 1001 to 1970, …),
100 % on all henon files, the wrap in `henon-mi1000` and `henon-mi1000-u`.

The start colour c0 depends on `inside` (`fractint001`[104]):

| inside | c0 | Evidence |
|---|---|---|
| 0 (default), −1 (`maxiter`), −60 (`bof60`) | 2 | `henon-def`, `henon-inbof-u`; maxiter checked during analysis |
| 1 … 255 | inside | `henon-in1`, `henon-in5`, `henon-in255-u` (255 then 1, 2, …); 10 and 100 checked during analysis |
| 300 | 1 | `henon-in300-u` |

Other values are untested; "≥ 256 → 1" and "≤ 0 → 2" are hypotheses.

With the default palette, colours 16 and 248–255 appear as 0 in saved files
(types.md §1.3); compare through the palette.

### 3.4 Resolution

The point count and colour period do not depend on the image size
(`res800/henon-mi10-800`, 100 %).

---

## 4. Single orbit with hit-count colouring

Types: icons, latoocarfian.

### 4.1 Algorithm (confirmed)

Same loop as §3.1 (start point plotted, then `1024 · maxiter` iterates, no
stopping rule), but each plot **increments** the pixel:

    v = displayed colour of the pixel, read back as the lowest palette index
        with the same RGB (types.md §1.3)
    pixel = min(v + 1, 255)

With the default palette, index 16 equals index 0 (black), so a count runs
1, 2, …, 15, 16 (shown and read back as 0), 1, 2, … and the saved image looks
like "hits mod 16". With a palette without duplicates (`icons-u`, `map=uniq.map`)
counts go above 16 (68 distinct colours). The cap at 255 is supported by one
pixel: in `latoocarfian-zero-u` the point (0, 0) is hit 2,048 times and is
saved as 255 (wrapping to 1 would give 8). The start point is counted
(`icons-def` is one pixel off without it).

### 4.2 icons (confirmed)

Params: λ = P1, α = P2, β = P3, γ = P4, ω = P5, n = trunc(P6) (degree).
Start (0.01, 0.003). Step, with z = x + iy:

    zz = x·x + y·y
    (u, v) = (x, y); repeat n − 2 times: (u, v) = (u·x − v·y, u·y + v·x)     [u + iv = zⁿ⁻¹]
    w = x·u − y·v                                                          [Re zⁿ]
    p = (λ + α·zz) + β·w
    x′ = (p·x + γ·u) − ω·y
    y′ = (p·y − γ·v) + ω·x

This is the Field–Golubitsky map z′ = (λ + α|z|² + β Re zⁿ + iω) z + γ z̄ⁿ⁻¹.
Evidence: `icons-def`, `icons-u`, `icons-p` (ω = 0.12), `icons-p5` (n = 5),
`icons-in5`, all 100 %. Other starts, e.g. (0, 0), fail (it is a fixed point).

### 4.3 latoocarfian (confirmed structure, numerically limited)

Params a, b, c, d = P1…P4. Start (1, 1). With the four function slots:

    x′ = fn1(y·b) + c·fn2(x·b)
    y′ = fn3(x·a) + d·fn4(y·a)

- Slot order confirmed by putting cos in one slot at a time
  (`latoocarfian-f1c` … `-f4c`, 100 % each; the other assignments give
  94.8–95.7 %).
- The **default slots (sin, sqr, sinh, cosh) make the orbit overflow at
  once**, so `latoocarfian-def` shows only the start pixel (49, 299) in
  colour 1. The familiar picture needs `function=sin/sin/sin/sin`.
- Points that are no longer finite are not drawn (`latoocarfian-def`
  100 %).
- Function values as in §2.4 (64-bit significand, then double). With that:
  `latoocarfian-ssss-mi10` and `-mi40` 100 %, a maxiter-80 render 91.6 %,
  `latoocarfian-ssss` (maxiter 150) 84.8 %. The structure is right (the
  attractor overlays exactly); the orbit separates somewhere between 40,961
  and 81,921 points, which we attribute to the last bit of some sin value
  (x87 `fsin` is not exactly rounded; we did not model its error).

---

## 5. kamtorus (confirmed)

Params: angle α = P1, step = P2, stop = P3, points per orbit = trunc(P4).
`maxiter` is not used. With L = trunc(P4) + 1:

    t = 0, o = 0
    while t ≤ stop:
        x = y = t / 3                      (not plotted)
        for k = 1 … L:
            q = x·x − y
            (x, y) = (x·cos α + q·sin α,  x·sin α − q·cos α)
            plot (x, y) with colour:  c0 if o = 0,  else C((o − 1)·L + k)
        o = o + 1
        t = t + step                      (repeated addition, not o·step)

C and c0 are those of §3.3 (`kamtorus-in5`: inside = 5 shifts every colour
by 3, 100 %). Orbit 0 (t = 0) stays on the origin, so it only colours that
pixel. Evidence: `kamtorus-def`, `-mi10`, `-p4-50`, `-p4-2000`, `-stop`
(stop 0.6, which distinguishes `t ≤ stop` with accumulated t from the
alternatives), `-a`, all 100 %; 800 × 600 also 100 %.

---

## 6. popcorn (confirmed)

popcorn draws an orbit from **every pixel**:

    for row r = 0 … H − 1 (top first), for column c = 0 … W − 1:
        (x, y) = pixel coordinate (gif-fractint-blocks §2.2: xmin + c·Δx, ymax − r·Δy)
        repeat maxiter − 1 times:
            x′ = x − h·fn1(y + fn2(t·y))
            y′ = y − h·fn3(x + fn4(t·x))         (both from the old x, y)
            plot (x′, y′) with colour (r + 1) mod 256
            stop this orbit if x′·x′ + y′·y′ > 4
        (the start pixel itself is not plotted)

h = P1, t = P3 (default 3). The default slots make this
`x − h·sin(y + (3y)²)`, `y − h·sinh(x + cosh(3x))`; the classic picture
needs `function=sin/tan/sin/tan`.

- Colour: start row + 1, modulo 256, so the orbits of row 255 are drawn in
  colour 0, which erases what earlier rows drew there (the model writes the
  0 and still matches 100 %). Evidence: the colour/row relation over the
  whole image, and the fits.
- The escape test comes **after** plotting; testing before plotting, or
  `bailout` 9/16/none, all fail (`popcorn-id-mi3`, a linear map with
  `function=ident/…`, is 100 % only with "plot, then stop if > 4").
- `maxiter − 1` iterates: `maxiter=2` gives exactly one point per pixel
  (100 %), maxiter 3 two.
- Overflowing functions: §2.4 (1.0).
- `inside=` has no effect (`popcorn-in5` = `popcorn-mi10`).
- Evidence: `popcorn-def`, `popcorn-mi10`, `popcorn-id-mi3`,
  `popcorn-st-mi3` 100 %; `popcorn-st-mi10` 99.966 % and `popcorn-st`
  (maxiter 150) 90.8 %, both limited by tan near its poles (§2.4).
- Unknown: the role of P2 and P4, and whether `bailout=` changes the 4.

---

## 7. popcornjul (escape time, confirmed for the default)

popcornjul is **not** an orbit plot: every pixel is computed independently
like the types in types.md. z₀ = (x, y) of the pixel, the step is popcorn's
(§6, same function slots and overflow rule), and the colour is the
escape-count rule of types.md §1.2: e = index of the first iterate with
x² + y² > 4 (e ≥ 1), colour e (wrapped above 255), inside (no escape before
maxiter) colour 0. Evidence: `popcornjul-def` 99.993 % (9 pixels, all
rounding); `popcornjul-st` (sin/tan) 94.9 %, again limited by tan near its
poles. The `e − 1` convention gives 79 %. Symmetry copying (types.md §1.6)
was not tested.

---

## 8. dynamic (confirmed)

An orbit from each point of a grid. n = trunc(P1), dt = P2, a = P3, b = P4:

    idx = 0
    for i = 0 … n            (n + 1 rows, top first)
      for j = 0 … n − 1      (n columns, left first)
        x = xmin + (j + ½)·(xmax − xmin)/n
        y = ymax − (i + ½)·(ymax − ymin)/n
        colour = ((idx + max(inside, 0)) mod 255) + 1
        plot (x, y); then repeat maxiter − 1 times:
            x′ = x − dt·sin(y + a·fn1(b·y))
            y′ = y + dt·sin(x + a·fn1(b·x))      (both from the old x, y)
            plot (x′, y′)
        idx = idx + 1

- There is one extra grid row: the last row's start points lie below the
  window but their orbits can enter it (n rows: 99.92 % / 98.6 %; n + 1 rows:
  100 %; n + 2 rows also fit some files but not `dynamic-def`).
- The outer sine is fixed; only the inner function is the slot `fn1`
  (`dynamic-fn`, `function=cos/cos/cos/cos`, is reproduced by sin outside and
  cos inside; cos outside fails).
- No escape test.
- inside shifts the colours (`dynamic-in5`: `(idx + 5) mod 255 + 1`, 100 %).
  Negative inside values are untested.
- Evidence: `dynamic-def`, `-mi2`, `-mi10`, `-p200-mi10`, `-fn`, `-in5`,
  `res800/dynamic-mi10-800`, all 100 %.

---

## 9. mandelcloud (hypothesis, 99.4–99.999 %)

Same grid as dynamic (n = trunc(P1), n + 1 rows × n columns, cell centres,
colour `idx mod 255 + 1`), but each grid point is a Mandelbrot parameter
c = x + iy:

    z = c
    repeat maxiter − 1 times:
        z = z² + c            (x′ = x·x − y·y + cx, y′ = 2·x·y + cy)
        if |z|² > 4: stop     (the escaping point is not plotted)
        plot z

Match: `mandelcloud-n2-u` 99.999 % (1 pixel), `mandelcloud-n10` 99.98 %,
`mandelcloud-mi10` 99.56 %, `mandelcloud-def` 99.40 %. The residual is
systematic, not rounding: some points of orbits that escape a few steps later
are missing in XFractint, a few escaping points are present, and where two
orbits hit the same pixel XFractint sometimes keeps the earlier one. We found
no rule (tried: plot-after-test, test of the previous value, z₀ = 0,
column-major order, erasing escaped orbits, n or n + 1 rows/columns). The
basic shape, grid, colours and point counts are reliable; the exact
treatment of escaping orbits is **unknown**.

---

## 10. julia_inverse (hypothesis, ~98.9 %)

Inverse iteration for the Julia set of z² + c, c = P1 + iP2. Observed:

- Colour = number of times a pixel was hit, capped at **P3** (default 4):
  P3 = 8 gives colours 1…8 (`julia_inverse-p8`); most curve pixels reach the
  cap.
- P4 (default 1024) had no visible effect (64 gave the identical image,
  `julia_inverse-q64`).
- Deterministic: `rseed=` and `maxiter` change nothing.
- A breadth-first walk of the preimage tree (z → ±√(z − c)) starting at the
  fixed point ½ − √(¼ − c), skipping a point whose pixel already has P3 hits,
  reproduces 98.9 % (`julia_inverse-def`), 99.35 % (`julia_inverse-c`,
  c = −1). Depth-first walks and other start points give 97.8–98.8 %. The
  exact traversal order, start point and role of P4 are **unknown**.

---

## 11. Interrupting, saving and resuming

Interactive renders were saved with `s` part-way (`resume/int-*`), loaded
again with XFractint's `r` command (which resumes, fractint-resume.md §5.2;
`resume/rload-*`) or with `filename=` + Enter (which recalculates;
`resume/enter-*`), and compared with batch renders of the same parameters
(`resume/ref-*`).

### 11.1 What an interrupted save contains

`fractint001`[202] = 2 and a `fractint002` block follow the usual framing
(gif-fractint-blocks §1.2, §2.1 [204] counts it), but **its payload is
type-specific** and much shorter than the 392-byte work list:

| Types | `fractint002` payload | [204] |
|---|---|---|
| henon, martin, icons, latoocarfian (single orbit), kamtorus | 64 bytes (§11.2) | 601 |
| dynamic | 48 bytes (§11.3) | 585 |
| popcorn | 8 bytes (§11.4) | 545 |
| popcornjul | the 392-byte work list of fractint-resume.md | 930 |
| julia_inverse | none; status [202] = **3** | 521 |

mandelcloud finished before any save we tried, so its form is unknown.
Types not listed (hopalong, gingerbreadman, chip, quadruptwo, threeply) were
not interrupted; they share §3's loop and probably henon's block (hypothesis).

### 11.2 64-byte orbit state

Little-endian. Values from `resume/int-henon` (8000-iteration render
interrupted after 3,266,799 points):

| Offset | Type | henon value | Meaning |
|---|---|---|---|
| 0 | i32 | 1 | format? (1 in every orbit block) |
| 4 | i32 | 536 | j mod 1001, j = index of the saved point (confirmed for henon, martin, icons; not for latoocarfian) |
| 8 | i32 | 205 | current time colour C(j) (§3.3; confirmed henon, martin, kamtorus; icons and latoocarfian store values we cannot explain) |
| 12 | i32 | 96 | row of the last plotted point (point j − 1), −1 if it was outside the image |
| 16 | i32 | 257 | column of that point, −1 if outside |
| 20 | f64 | 0.94526160… | x of point j (computed, **not yet plotted**) |
| 28 | f64 | 0.12224752… | y of point j |
| 36 | f64 | 0.40749175… | type-specific: x of point j − 1 for henon and martin, 1.0 for latoocarfian, ? for icons |
| 44 | i32 | 0 | kamtorus: index of the point within its orbit (1161) |
| 48 | f64 | 0 | kamtorus: 0.69 = the current t (also at 36) |
| 56 | i32 | 3266800 | j + 1 (points counted so far, start point included) |
| 60 | i32 | 0 | (high half of 56?) |

For kamtorus (`resume/int-kam`: 1, 888, 158, 169, 148, …, 412300) the
saved index j is one more than (o − 1)·L + k of §5; the fields are otherwise
read the same way.

The image holds exactly the points 0 … j − 1 (`resume/int-henon` vs a
simulation through j − 1: 100 %; through j: 1 pixel off).
`python3 -I tools/blackbox/resume/gif.py FILE` prints a block's first i32
values.

### 11.3 dynamic (48 bytes)

`resume/int-dyn` (n = 100): i32 1, 0, 0, 70, 51, −1, then f64 −10.2, 18.6,
then i32 24, 3. The doubles are the start point of grid point (i = 3,
j = 24), which are the last two values. The image holds grid points 0…3·n +
23 completely (100 % against a simulation), i.e. the saved grid point is the
next one to compute. Values 70, 51, −1 are unknown.

### 11.4 popcorn (8 bytes)

`resume/int-popcorn`: i32 1, 48. The image holds the orbits of rows 0–47 and
of row 48, columns 0–221 (100 % against a simulation), so 48 is the row in
progress; the column is not saved.

### 11.5 Resuming (confirmed)

Loading with `r` continues the calculation and finishes it:

- kamtorus, dynamic, popcorn, popcornjul: the finished image is **identical**
  to the uninterrupted batch render (`resume/rload-*` vs `resume/ref-*`).
- henon, icons, latoocarfian: identical **except one pixel**, which is the
  pixel of the very last iterate (j = 1024·maxiter) of the uninterrupted run:
  after a resume the orbit ends one point early. (henon: pixel (223, 256)
  has 18 instead of 25; icons and latoocarfian one hit fewer.)
- `filename=` + Enter recalculates from scratch (`resume/enter-henon`,
  `enter-popcorn` identical to the references).

A writer that cannot produce these blocks can always save a finished image;
a partial FractalForge image would need the type-specific block above, which
we have not load-tested with crafted values.

---

## 12. Verification

`tools/blackbox/orbit-maps/verify.py` re-renders every corpus file with this
spec (martin, chip, quadruptwo and threeply from orbits computed by
`libm-orbits.sh` inside the container) and compares indices through the
palette (types.md §1.3). Results:

| Files | Match |
|---|---|
| henon-* (def, mi10, mi100, mi1000, mi1000-u, p13, in1, in5, in255-u, in300-u, inbof-u, out7, float, sym-none, pass1) | 100 % each |
| hopalong-def, -p, -p3; martin-def, -p; gingerbreadman-def, -p | 100 % |
| chip-def, -mi2z, -p; quadruptwo-def, -p; threeply-def, -p | 100 % |
| icons-def, -u, -p, -p5, -in5 | 100 % |
| latoocarfian-def, -zero-u, -f1c…-f4c, -ssss-mi10, -ssss-mi40 | 100 % |
| latoocarfian-ssss (maxiter 150) | 84.83 % (§4.3) |
| kamtorus-def, -mi10, -p4-50, -p4-2000, -stop, -a, -in5 | 100 % |
| popcorn-def, -mi10, -id-mi3, -st-mi3, -in5 | 100 % |
| popcorn-st-mi10 / popcorn-st | 99.966 % / 90.84 % (tan, §2.4) |
| popcornjul-def / popcornjul-st | 99.993 % / 94.94 % |
| dynamic-def, -mi2, -mi10, -p200-mi10, -fn, -in5 | 100 % |
| mandelcloud-n2-u, -n10, -mi10, -def | 99.999 / 99.981 / 99.556 / 99.395 % |
| julia_inverse-def, -rs1, -q64 / -p8 / -c | 98.887 / 98.747 / 99.353 % |
| res800/henon-mi10-800, kamtorus-800, dynamic-mi10-800 | 100 % |

---

## 13. Unknown or untested

- mandelcloud's handling of escaping orbits (§9); julia_inverse's traversal
  and P4 (§10).
- Bit-exact function-slot trigonometry (latoocarfian beyond ~41,000 points,
  popcorn/popcornjul with tan near its poles).
- popcorn P2 and P4, `bailout=` for popcorn and popcornjul; popcornjul
  symmetry copying.
- inside values other than those in §3.3 and §8.
- log of 0 in chip/quadruptwo; sign(0) handling is confirmed only through
  the starts at (0, 0).
- Interrupted saves of hopalong, gingerbreadman, chip, quadruptwo, threeply
  and mandelcloud; the unexplained fields of §11.2–11.3; loading crafted
  orbit-state blocks.
- Rotated or skewed views (6-value corners), and `orbitdelay`/`orbitinterval`.
