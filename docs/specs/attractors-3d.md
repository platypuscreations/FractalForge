<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Continuous and 3D orbit ("strange attractor") types

Status: **black-box derived.** Everything here comes from GIFs that XFractint
(Debian `xfractint 20.4.10-5`, amd64) saved, compared against our own
re-renders. Published mathematics (the Lorenz, Rössler and Pickover systems,
the one-lobe "proto-Lorenz" system and its n-fold covers, the KAM-torus map)
was used only as a hypothesis and is marked confirmed only where XFractint's
pixels reproduce. No Fractint, XFractint or Iterated Dynamics source code,
headers, help text or documentation was consulted. See
[CLEANROOM.md](../CLEANROOM.md). Names and notation are our own.

Evidence: `corpus/xfractint/attractors-3d/` (command lines in `cases.txt`
there; file names below are relative to that folder). Unless a case says
`FULL` it is a 400×300 batch render (`viewwindows=2`). Tools:
`tools/blackbox/attractors-3d/` — `sim.py` re-renders a GIF from its stored
parameters and prints the colour match and the drawn/blank match,
`ordersearch/` is the arithmetic-order search harness (§3.3), `orbit.pl`
computes trig-based orbits inside the container (§6), `intr.sh` produced the
interrupt/resume files, `resume_info.py` decodes them.

Block offsets refer to [gif-fractint-blocks.md](gif-fractint-blocks.md)
(`fractint001`, §2.1). P1…P4 are the f64 parameters at 44, 52, 270, 278.
"Match" means pixels whose RGB equals XFractint's (indices that share an RGB
value count as equal, see [types.md](types.md) §1.3: XFractint never stores
colour 16, 31 or 248–255; they appear as 0, 15 and 0).

**Confirmed** = our re-render reproduces whole images (100 % unless stated).
**Hypothesis** = fits the evidence, other explanations remain.
**Unknown** = not determined.

---

## 1. Types covered

Names found by trying `type=NAME` in batch mode (an unknown name writes
`stopmsg.txt` "couldn't understand the argument"). Type numbers are the i16 at
`fractint001`[10].

| Name | Number | Default corners (xmin/xmax/ymin/ymax) | Default P1/P2/P3/P4 (P5, P6) | Status |
|---|---|---|---|---|
| `lorenz` | 63 | −15/15/0/30 | 0.02 / 5 / 15 / 1 | confirmed (§3) |
| `lorenz3d` | 84 | −30/30/−30/30 | 0.02 / 5 / 15 / 1 | confirmed (§3, §4) |
| `rossler3d` | 86 | −30/30/−20/40 | 0.04 / 0.2 / 0.2 / 5.7 | confirmed (§5) |
| `pickover` | 89 | −8/3 / 8/3 / −2 / 2 | 2.24 / 0.43 / −0.65 / −2.43 | confirmed (§6) |
| `lorenz3d1` | 124 | −30/30/−30/30 | 0.02 / 5 / 15 / 1 | equations confirmed, arithmetic unknown (§7) |
| `lorenz3d3` | 125 | −30/30/−30/30 | 0.02 / 10 / 28 / 2.66 | hypothesis (§7) |
| `lorenz3d4` | 126 | −30/30/−30/30 | 0.02 / 10 / 28 / 2.66 | hypothesis (§7) |
| `kamtorus3d` | 42 | −3/3/−1/3.5 | 1.3 / 0.05 / 1.5 / 150 | partial (§8) |
| `icons3d` | 160 | −2/2/−1.5/1.5 | −2.34 / 2 / 0.2 / 0.1 (P5 0, P6 3) | mostly unknown (§9) |

Names that do **not** exist: `lorenzfp`, `lorenz3dfp`, `lorenz3d2`, `lorenz1`,
`lorenz2`, `rossler`, `rossler3dfp`, `pickover3d`, `chua`, `duffing`,
`thomas`, `halvorsen`. Related 2D orbit types that exist but are outside this
spec: `kamtorus` (40), `icons` (159), `henon`, `hopalong`, `martin`, `chip`,
`threeply`, `quadruptwo`, `gingerbreadman`, `latoocarfian`, `dynamic` (139).

The 3D view fields of `fractint001` used here: rotation x/y/z (i16 at
128/130/132, default 60/30/0), perspective (144, default 0), x/y shift
(146/148, default 0). Scale x/y/z (134–138) has **no effect** on these types
(`l3-mi2-scale` during development: byte-identical pixels).

`float=yes` / `float=no` change nothing: `lorenz-mi2-float` differs from
`lorenz-mi2` only in the random-seed byte 98 (also checked for lorenz3d,
rossler3d, pickover, kamtorus3d, icons3d, lorenz3d1 during development).
There is no separate integer algorithm to specify.

---

## 2. Common pieces

### 2.1 Window mapping (confirmed)

A plot coordinate pair (u, v) maps to pixel

    col = trunc((u − xmin) / (xmax − xmin) · (W − 1))
    row = trunc((ymax − v) / (ymax − ymin) · (H − 1))

`trunc` rounds toward zero (C `(int)` conversion). This is the escape-time
mapping of gif-fractint-blocks §2.2 with truncation instead of pixel
centres. Evidence: every confirmed case; e.g. in `lorenz-mi2` the start point
(x=1, z=1) is drawn at (212, 289) = trunc(212.8), trunc(289.03); rounding
fails on 798 of 2049 points. Values outside the int range are treated as
"far" (§2.5). Resolution only enters through W and H: `lorenz-800-mi2` and
`lorenz3d-800-mi5` (800×600) match with the same rules and the same number
of iterations.

### 2.2 Number of iterations and colour (confirmed)

All Lorenz/Rössler/Pickover-family types run **N = 1024 · maxiter** plotted
iterations (maxiter = i32 at 408). Colour of the thing drawn for global
iterate number `j` (§3.2, §4.4 say which j):

    c(j) = 2                              if j ≤ 999
    c(j) = 3 + floor((j − 1000) / 1001)   if j ≥ 1000
    colour index = ((c(j) − 1) mod 255) + 1

So colour 2 lasts 999 iterates, every later colour 1001; colours wrap from 255
to 1, never 0. Evidence: `lorenz-mi2` (boundaries at iterates 999/1000 and
2000/2001 seen pixel by pixel), `lorenz-mi300` (max colour 54) and
`lorenz-mi1000` (wraps several times), both 100 % only with the wrap that
skips 0 (plain mod 256 gives 72–73 %). Background is colour 0. Later drawing
overwrites earlier pixels. maxiter=1 is rejected by XFractint
("couldn't understand the argument").

### 2.3 Lines (confirmed)

Consecutive points are joined by a line when both end pixels are inside the
image. The line visits every integer value of the major axis (|dx| ≥ |dy| ⇒
x is major; for |dx| = |dy| either gives the same pixels); for step i
(0…m, m = max(|dx|, |dy|)) the minor coordinate is the exact value
`s0 + i·d/m` **rounded half up** (ties go to the larger column / row):

    minor_i = floor((2·(s0·m + i·d) + m) / (2·m))

Both end pixels are drawn. Evidence: about 5 600 tie cases in the sparse corpus
images go "up"; the only contrary case, one 16:8 segment in a zoomed
development image (`zw-1`, 16 px), is unexplained (unknown, maybe sub-pixel
endpoint differences, see §4.6).

### 2.4 Points partly off-screen (confirmed)

If the previous pixel is outside the image and the new one inside, only the
new pixel is set (no line). If the new pixel is outside, nothing is drawn.
So no clipping of lines against the border ever happens. Evidence:
`lorenz3d-mi2-persp100` and the development image `pc-100-mi20`, where the
only pixel of two border-crossing segments that XFractint drew is their
shared on-screen end point; a model that clips lines adds 30 wrong pixels.

### 2.5 Far points stop the orbit (confirmed behaviour, hypothesis limits)

If a new point's pixel column or row is very far outside the image, XFractint
stops plotting for the rest of the image (it still saves a finished image,
status 4). The limits are absolute pixel counts, not relative to the image
size:

- column: the limit L_c satisfies 5413 ≤ L_c < 6400 (`lorenz3d-mi2-zoom08`
  reaches |col| = 5412 and is drawn completely; `-zoom06` and the 800×600
  development image `zl-800-w1.6` stop);
- row: 4200 ≤ L_r < 5400 (from development renders `zx-0.3`, `zx-0.4`).

`sim.py` uses |col| > 6000 or |row| > 4800. The exact test is **unknown**.
Effects: `lorenz-mi2-zoom1` and `lorenz3d-mi2-zoom06` are completely black,
although part of the orbit passes through the window; `kamtorus3d` ends
because of it (§8). Normal windows never hit the limit.

---

## 3. lorenz (type 63)

### 3.1 Equations and integration (confirmed)

Lorenz system with dt = P1, a = P2, b = P3, c = P4 (defaults 0.02, 5, 15, 1),
forward Euler, start (x, y, z) = (1, 1, 1). Exactly, in IEEE double
arithmetic with this operation order (round after every operation, no fused
multiply-add):

    xdt = dt*x
    ydt = dt*y
    x' = x + ((dt*a)*y − (dt*a)*x)
    y' = y + (((dt*b)*x − ydt) − xdt*z)
    z' = z + (ydt*x − (dt*c)*z)

All three use the old x, y, z. The order matters: the system is chaotic and a
one-ulp difference changes the pixels after about 3 000 iterations. This
ordering was the unique survivor of 4 900 candidate orderings × 3 forms
(`ordersearch/mk_lorenz2d.py` on `lorenz-mi10`), and reproduces
`lorenz-def` (153 600 iterates), `lorenz-mi1000` (1 024 000) and the 800×600
image exactly. The same steps are used by lorenz3d (§4).

### 3.2 Plotting (confirmed)

The plot pair is (u, v) = (x, z): horizontal x, vertical z. No hidden
iterations, no centring. Let P0 = (1,1,1) and Pj the j-th Euler iterate. For
j = 1 … N: draw P(j−1) → Pj (§2.3/§2.4) in colour c(j). Stop early on a far
point (§2.5).

### 3.3 How the ordering was found

Black-box only: for each candidate set of expressions the harness simulates
the orbit and stops at the first segment whose pixels are dark or have a
lower colour than the segment's own colour in the XFractint image (colours
only grow until they wrap). Wrong orderings fail at 3 300–4 300 iterates,
the right one never does. The same harness found the Rössler ordering (§5).

### 3.4 Interrupt, save and resume (confirmed)

An interrupted lorenz image (`lorenz-int.gif`, `s` after 12 s) is saved with
status 2 (`fractint001`[202]) and a 64-byte `fractint002` block (extension
length 601 = 521 + 80):

| Off | Type | Meaning | `lorenz-int` |
|---|---|---|---|
| 0 | i32 | 1 (format?) | 1 |
| 4 | i32 | position in the current colour period, (C − 1002) mod 1001 | 699 |
| 8 | i32 | current colour index (c(C) wrapped) | 202 |
| 12 | i32 | row of the pixel of iterate C − 2 | 143 |
| 16 | i32 | column of that pixel | 159 |
| 20 | f64 | x of iterate C − 1 | −2.85496… |
| 28 | f64 | z of iterate C − 1 | 15.3367… |
| 36 | f64 | y of iterate C − 1 | −1.40458… |
| 44 | 12 bytes | 0 | |
| 56 | i32 | iteration counter C | 6 582 275 |
| 60 | 4 bytes | 0 | |

The f64 state equals our own simulation of iterate C − 1 bit for bit (also
for two more development saves at C = 3 242 875 and 5 704 850), which
independently confirms §3.1 over millions of iterations. Note the order
x, z, y.

Loading with `filename=` and Enter restarts from scratch (as for escape-time
types, fractint-resume.md §5.1). Loading with the in-program `r` command
**resumes** (`lorenz-rload.gif`): the counter went on to 10 792 100, the
calculation time accumulates (1145 → 1872), earlier pixels stay, and the
saved state is again exactly on the uninterrupted orbit (iterate 10 792 098).
After the resume the stored counter is one higher than the uninterrupted
relation (state = iterate C − 2, [4] = (C − 1003) mod 1001); the colour [8]
still equals c(C). So a resumed run's colour changes come one iterate earlier
than an uninterrupted one (hypothesis; one pixel's worth of difference).

A writer that saves an unfinished lorenz image should write this block with
the fields above; a reader may resume from (x, y, z) at iterate C − 1.

---

## 4. lorenz3d (type 84) and the common 3D pipeline

lorenz3d integrates exactly as lorenz (§3.1, same start, same arithmetic) and
draws lines. Its pixels depend on the 3D view fields.

### 4.1 Rotation (confirmed)

With angles rx, ry, rz (degrees, i16 at 128/130/132, any value, e.g.
200/−45/370 in `lorenz3d-mi2-rotbig`), build

    R = Rz(−rz) · Ry(−ry) · Rx(−rx)
    Rx(t) = [[1,0,0],[0,cos t,−sin t],[0,sin t,cos t]]
    Ry(t) = [[cos t,0,sin t],[0,1,0],[−sin t,0,cos t]]
    Rz(t) = [[cos t,−sin t,0],[sin t,cos t,0],[0,0,1]]

(t in radians, t = deg·π/180) and (u, v, d) = R · (x, y, z): u horizontal, v
vertical (up), d depth. With 0/0/0 the plot is (x, y). Confirmed by
`lorenz3d-mi2-rot0`, `-rotx90`, `-roty80`, `-rotz90`, `-rot10-20-30`,
`-rotbig` and the default 60/30/0. The angle conversion may use a slightly
inexact π: one point of `pickover-def` (out of 153 600) lands one column
left of our model (309.0000027 → XFractint 308); a π of 3.1416 would explain
it, an exact π does not (hypothesis, not tested further).

### 4.2 Hidden prefix and centring (confirmed)

The first `K` iterates are not drawn; they only fix the view centre. K
depends on the type: **100** for lorenz3d (and pickover), **500** for
rossler3d, **750** for lorenz3d1/3/4 (hypothesis for 3/4: §7). For
j = 1 … K (not P0), take (u, v, d) of Pj and update per coordinate

    if value < lo: lo = value
    else if value > hi: hi = value

with lo = +∞, hi = −∞ initially. Note the **else**: a value that lowers `lo`
is never tested against `hi`. While a coordinate only decreases, `hi` stays
−∞, the centre becomes −∞ and nothing at all is drawn (`lorenz3d-mi20-origin`
is completely black; `rossler3d-mi2-probe0` too). A coordinate that only
increases is fine. Then

    cu = (lo_u + hi_u)/2 − xshift/100 · (xmax − xmin)
    cv = (lo_v + hi_v)/2 − yshift/100 · (ymax − ymin)

(x/y shift = i16 at 146/148, percent of the window, positive = right / up;
`lorenz3d-mi2-shift` 20/10, `-shiftneg` −15/25 with asymmetric corners). The
plot pair is (u − cu, v − cv) mapped with §2.1. So the attractor sits at the
window's centre whatever the window's centre is (rossler3d's default window is
centred at y = 10, and its attractor is drawn around row 199).

Evidence for "else if": the default-rotation `lorenz3d-mi2` is off by about
0.25 pixel with a plain min/max; `lorenz3d-mi2-roty80` (development `ry80`,
`ry89`, `ry91`, `ry120`, `l3-mi2-roty90`) are off by 19–28 columns with a
plain min/max and exact with "else if". Evidence for P1…PK (not P0…): bounding
boxes from P0 fail `lorenz3d-mi2` (1453 px). K = 100 was pinned with
`lorenz3d-mi2-probe` (only K = 100 matches; 99 and 101 give 1–4 px errors).

### 4.3 Perspective (confirmed for 30 ≤ p ≤ 999)

If the perspective field p (i16 at 144) is non-zero, with the depth range of
the hidden prefix (same "else if" bounding box, E = hi_d − lo_d):

    den = (p/100)·E + (d − hi_d)
    if den ≤ 0: stop plotting for the rest of the image
    f = (p/100)·E / den
    plot pair = (f·(u − cu), f·(v − cv))

i.e. the viewer stands p % of the prefix's depth range in front of the far
face of the prefix box; points on that face keep their size, nearer points
grow. The centring of §4.2 (including shift) is applied before the scaling.
Evidence: `lorenz3d-mi2-persp100/150/300/75/50` and development renders at
30, 60, 90, 120, 200, 500, 999 all match exactly; for p ≤ 90 the viewer is
inside the box and XFractint stops at the first point behind it (e.g.
`-persp75` shows only the first two colour-2 arcs). p = 1 and 10 draw
nothing.

### 4.4 Drawing order and colours (confirmed)

Let K be the prefix length. After the prefix, iterate K+1 becomes the first
"previous" point (not drawn for line types). Then for j = K+2 … N+1, draw
P(j−1) → Pj with colour **c(j − 1)** (§2.2). So a 3D image uses N+1 iterates
in total, and the hidden ones still count for the colour. Far points (§2.5)
and points behind the viewer (§4.3) stop the loop.

### 4.5 Interrupt / resume (confirmed)

An interrupted lorenz3d (`lorenz3d-int.gif`) or pickover (`pickover-int.gif`)
image is saved with **status 3** and no `fractint002` (extension length
521). Loading it with `r` (`lorenz3d-rload.gif`) shows it unchanged and saves
it again with status 3: 3D orbit types cannot be resumed. A writer should
save unfinished 3D orbit images the same way (status 3, no 002).

### 4.6 Known residuals

- One 16:8 line in a zoomed development image breaks ties the other way (§2.3).
- The far-point limits of §2.5.
- The possible inexact π (§4.1).

---

## 5. rossler3d (type 86) (confirmed)

Rössler system, dt = P1, a = P2, b = P3, c = P4 (0.04, 0.2, 0.2, 5.7),
start (1, 1, 1), Euler, 3D pipeline of §4 with prefix K = 500, lines.
Exact arithmetic:

    xdt = dt*x;  ydt = dt*y
    x' = x + (−ydt − (dt*z))
    y' = y + (xdt + (dt*a)*y)
    z' = z + (((dt*b) + xdt*z) − (dt*c)*z)

(`−ydt − dt*z` is exactly `−(dt*y) + −(dt*z)`.) Unique survivor of 9 504
orderings on `rossler3d-def` (153 600 iterates, 100 %). K = 500 is the only
value that matches `rossler3d-mi2` and `-probe` (dt = 0.001; 499 and 501
miss by one pixel).

---

## 6. pickover (type 89) (confirmed)

Pickover's 3D map (a map, not an ODE), a = P1, b = P2, c = P3, d = P4:

    x' = sin(a*y) − z*cos(b*x)
    y' = z*sin(c*x) − cos(d*y)
    z' = sin(x)

start (1, 1, 1), 3D pipeline of §4 with K = 100, but **points instead of
lines**: iterate K+1 is plotted in colour c(K) (= 2), then every later
iterate j = K+2 … N+1 is plotted as a single pixel in colour c(j−1).

The map is chaotic and the result depends on the last bit of sin and cos:
macOS's libm gives a different orbit after two iterations. XFractint uses
glibc's `sin`/`cos` (the container's glibc 2.41); `orbit.pl` computes the
orbit with Perl inside the black-box container (Perl calls the same libm) and
then all of `pickover-mi2`, `-mi2-rot0` and the development `pk-mi10` match
100 %, `pickover-def` to one pixel (§4.1). An implementation must use
correctly rounded sin/cos (or glibc's) to reproduce images exactly; any
other libm gives the same attractor with different pixels.

Defaults for corners, ±8/3 × ±2, are the 4:3 window around 0.

---

## 7. lorenz3d1, lorenz3d3, lorenz3d4 (types 124–126)

### 7.1 lorenz3d1: the one-lobe Lorenz system (equations confirmed)

With dt = P1, a = P2, b = P3, c = P4, r = √(x² + y²):

    x' = x + dt·( −(a+1)·x + (a − b + z)·y + (1 − a)·r )
    y' = y + dt·( (b − a − z)·x − (a+1)·y + (b + a − z)·r )
    z' = z + dt·( y/2 − c·z )

This is the Lorenz system written in the coordinates (x² − y², 2xy, z) of
the original — it was derived as a hypothesis from that substitution and the
pixels confirm it. Start (1, 1, 1), 3D pipeline of §4, lines, prefix
K = 750 (`lorenz3d1-mi2` matches only for K = 750 or 751 with the first
drawn point at 751; `-mi3-dt01` and the rot0 image allow 710–850).

Images of up to about 3 000 iterates match exactly with the straightforward
formulas above (`lorenz3d1-mi2`, `-mi2-rot0`, `-mi3-dt01`: 100 %). The
**arithmetic order is unknown**: none of 1.17 million candidate orderings
(`ordersearch/lorenz3d1.spec`) stays on XFractint's orbit past iterate
≈ 3 800, so longer images diverge chaotically (`lorenz3d1-def`: colours
15.6 %, but drawn/blank pixels largely agree; see `sim.py` output). An
implementation following this spec gives the same attractor and the same
first ~3 000 iterates.

### 7.2 lorenz3d3, lorenz3d4: n-fold covers (hypothesis)

Hypothesis: with W = x + iy, the n-fold system (n = 3 for 125, n = 4 for
126) is the cover W^n = w of the one-lobe system: compute w = W^n,
(u, v) = (Re w, Im w), r = |w|, the one-lobe derivatives
u̇ = −(a+1)u + (a−b+z)v + (1−a)r, v̇ = (b−a−z)u − (a+1)v + (b+a−z)r,
then Ẇ = W·(u̇ + i v̇)/(n·w), ż = v/2 − c·z, and an Euler step of length dt.
For n = 2 this reproduces lorenz3d exactly (checked: 0 px on `l3-mi2`), which
supports the construction. For n = 3 and 4 the attractors have the observed
3- and 4-fold symmetry, size and shape, start (1, 1, 1) with K = 750
(745–755 from the colour-2 part of `-mi2-rot0`), and the early (colour 2)
part of the orbit matches; the images diverge from about iterate 1 700
(`lorenz3d4-mi2-rot0` 99.6 %, `lorenz3d3-mi2-rot0` 98.5 %). Whether the
divergence is only arithmetic order or a different formula is **unknown**.

---

## 8. kamtorus3d (type 42) (partial)

Observed / hypothesised (`kamtorus3d-*`):

- Orbit map (KAM torus, angle a = P1): x' = x·cos a + (x² − y)·sin a,
  y' = x·sin a − (x² − y)·cos a. Orbits o = 0, step, 2·step, … (step = P2,
  accumulated by repeated addition), each started at x = y = o/3 and
  iterated **P4 + 1** times (151 for the default 150). The same counts and
  colour rule reproduce the 2D `kamtorus` (type 40) exactly in development.
- Each iterate is a point (x, y, o) plotted as one pixel; colour c(i − (P4+1))
  where i counts all iterates from 1 (§2.2; so the first orbit is colour 2).
- **The orbit limit P3 is not used to stop**: the default image has 43 orbits
  (o up to 2.1) and changing P3 to 1.0 or 3 (`-rot0-stop3`) keeps about the
  same count; with step 0.1 (`-rot0-step01`) there are 22. The loop ends when
  an orbit escapes and its points hit the far-point limit (§2.5). P3 still
  changes a few pixels (hypothesis: orbits beyond P3 also plot their start
  point (o/3, o/3); seen on the diagonal in `kamtorus3d-rot0`).
- No hidden prefix and **no centring**: with rotation 0/0/0 the plot is
  (x, y) in the window directly (`kamtorus3d-rot0` 99.97 %, `-stop3`
  99.98 %).
- With the default rotation 60/30/0 our §4.1 rotation of (x, y, o) gives the
  right picture but only about half the pixels in the right place
  (`kamtorus3d-def` 96.5 % colour match); a depth scale of 0.97 on o improves
  it. The exact 3D transform for this type is **unknown**.
- maxiter has no effect (`kamtorus3d` with maxiter 20 = default).

---

## 9. icons3d (type 160) (mostly unknown)

Defaults P1…P6 = −2.34, 2, 0.2, 0.1, 0, 3 (the λ, α, β, γ, ω, degree of the
symmetric-icon map of the 2D `icons` type, hypothesis). Points are plotted
(dust-like image `icons3d-def`). Colour counting follows §2.2 with about
2 000 hidden iterates: `icons3d-mi3-rot0` (3 072 iterates) contains only
colours 4 and 5 plus one pixel of colour 3, as expected if iterates up to
≈ 2 000 are hidden and the first plotted one has colour 3. The map, the
third coordinate, the start point and the exact prefix are **unknown**.

---

## 10. Implementation checklist

1. Read P1…P4 (P5, P6 for icons3d), corners, maxiter (408), rotation
   (128–132), perspective (144), shift (146–148), W, H.
2. N = 1024·maxiter. Colour c(j) of §2.2 with the 255-wrap.
3. lorenz: §3; 3D types: §4 with the type's K, equations and point/line mode.
4. Use IEEE double, no fused multiply-add, the stated operation orders, and
   glibc-equivalent (correctly rounded) sin/cos for pickover.
5. Lines §2.3, partial off-screen §2.4, far stop §2.5, perspective stop §4.3.
6. Saving unfinished: lorenz → status 2 + 64-byte `fractint002` (§3.4);
   3D types → status 3, no `fractint002`.

## 11. Evidence summary (`sim.py` on the corpus)

All `lorenz-*`, `lorenz3d-*`, `rossler3d-*`, `pickover-mi2*` and
`lorenz3d1-mi2*`, `lorenz3d1-mi3-dt01`: 100 % colour match.
`pickover-def`: one pixel. `lorenz3d1-def`, `lorenz3d3-*`, `lorenz3d4-*`:
chaotic divergence (§7). `kamtorus3d-rot0*`: 99.97–99.98 %;
`kamtorus3d-def`: 96.5 %. `icons3d-*`: not modelled. (The drawn/blank
figure `sim.py` also prints is lowered by the duplicate-palette artefact:
pixels whose colour is 16 or 248–255 are stored as 0, e.g. `lorenz-mi1000`
is 100 % in colour but 90.6 % drawn/blank.)
