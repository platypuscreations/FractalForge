<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# 3D transform of an existing image (planar and spherical)

Status: **Phase 5 draft, derived from black-box analysis only.** No Fractint,
XFractint or Iterated Dynamics source code, help text or documentation was
consulted (see [CLEANROOM.md](../CLEANROOM.md)). Every claim is backed by
XFractint output in `corpus/xfractint/3d-transform/` and is marked
**confirmed** (reproduced pixel-for-pixel by our simulator or seen directly in
the bytes), **hypothesis** (fits the evidence but not fully pinned down) or
**unknown**. All names are our own.

Out of scope here: light-source fill types 5 and 6 and their keys (`lightsource=`,
`smoothing=`, `ambient=`, `haze=`, `randomize=`), stereo pairs (`stereo=` and
friends) and random-dot stereograms. Other specs cover them.

Evidence: `corpus/xfractint/3d-transform/cases.txt` lists every case as
`NAME INPUT ARGS`; outputs are `out/NAME.gif`, inputs `in/INPUT`. Case names
are quoted in `this font` below. Tools: `tools/blackbox/3d-transform/`
(`mkin.py` writes synthetic inputs, `gen-inputs.sh` regenerates all of them,
`runcases.sh` reruns the cases, `sim3d.py` is the reference simulator of this
spec). XFractint: Debian `xfractint 20.4.10-5` (amd64), batch mode.

Conventions: source image `src` is `sw × sh` pixels, `(x, y)` with row 0 at
the top, colour index `c`. Output image is `W × H`. `trunc` rounds toward
zero, `floor` toward −∞. Angles are degrees. Percent parameters are divided by
100 (`scalexyz=90/90/30` → `sx = 0.9, sy = 0.9, r = 0.3`).

---

## 1. Invocation, inputs, output size

### 1.1 Command line (confirmed)

    xfractint batch=yes 3d=yes filename=IN.gif [3D keys] savename=OUT.gif

renders the 3D transform of `IN.gif` and saves it. No other key is needed.
Without `3d=yes` the same command only re-saves the loaded image (`nl-80`).
3D keys used in this spec (stored fields per
[gif-fractint-blocks.md](gif-fractint-blocks.md) §2.1):

| Key | Values | Default | Stored at |
|---|---|---|---|
| `rotation=ax/ay/az` | degrees, any sign, > 360 allowed (`big-rot`) | 60/30/0 | 128/130/132 |
| `scalexyz=sx/sy/r` | percent, negative allowed (`neg-sx`) | 90/90/30 | 134/136/138 |
| `roughness=r` | percent (same field as 3rd `scalexyz` value), negative allowed (`neg-r`) | 30 | 138 |
| `waterline=n` | colour index | 0 | 140 |
| `filltype=n` | −1 … 6; 7 is rejected with "couldn't understand the argument" (`fill7`, no image) | 0 | 142 |
| `perspective=p` | percent, 0 = off | 0 | 144 |
| `xyshift=dx/dy` | percent of image size | 0/0 | 146/148 |
| `transparent=lo/hi` | colour range | 0/0 (off) | 254/256 |
| `sphere=yes` | | no | 126 |
| `longitude=a/b`, `latitude=a/b`, `radius=n` | sphere only (§5) | see §5 | 128/130, 132/134, 136 |

**The 3D fields of the input file are ignored**: `blk3d` loads a file whose
`fractint001` block holds rotation 11/22/33, scale 50/70/40 and fill 1; the
render and the saved fields use the defaults (confirmed: pixels 100 % equal to
the default simulation, saved fields = defaults).

### 1.2 Accepted inputs (confirmed)

- Any GIF works. Files without Fractint blocks (all `mkin.py` inputs) and
  XFractint renders (`m3d-def`) are both accepted. A Fractint block in the
  input changes nothing in the pixels (`flat5` vs `flat5b`: identical).
- 256-colour and 16-colour global tables were tested (`h-16`, `h-16k3`). The
  colour count matters for heights (§2.1) and for the output palette (§7).
- Only the pixel indices and the palette are used.

### 1.3 Output size: the canvas (partly confirmed)

The output image has the size of the **canvas** XFractint creates when it
loads the GIF. The same canvas appears when the file is only loaded
(`nl-80`), so this is a property of the loader.

- If `|4·sh / (3·sw) − 1| ≤ 0.04` (aspect within 4 % of 4:3), the canvas is
  `sw × sh` (confirmed: 98×75, 99×75, 101×77, 97–104×75, 100×72, 100×73,
  100×77, 100×78, 60×45 and all 4:3 sizes). 96×75 (ratio 1.0417) is already
  enlarged.
- Otherwise the canvas is often enlarged to 4:3 by adding rows
  (`H = round(sw·3/4)`, halves rounded up: 105×75 → 105×79, 110×75 → 110×83,
  130×80 → 130×98, 120×75 → 120×90, 200×100 → 200×150, 250×125 → 250×188,
  320×200 → 320×240, 400×200 → 400×300, 640×400 → 640×480) or columns
  (`W = round(sh·4/3)`: 80×75 → 100×75, 100×80 → 107×80, 100×100 → 133×100,
  300×300 → 400×300, 150×300 → 400×300).
- **Unknown rule**: some sizes far from 4:3 are kept unchanged: 240×120,
  256×128, 257×128, 260×130, 280×140, 300×150, 300×200 (`cz-*`, `u-w-r1`).
  We did not find what separates them from 250×125 or 200×100 (repeat runs
  are deterministic).
- The loaded image sits at the canvas's top-left. **Canvas pixels outside the
  image are not drawn at all** (`pad-s0`: with all columns collapsed onto one
  output column, the last image column wins, so the 20 padding columns
  produce nothing; `pad-s50`, `padh-s50`). All centring below uses the canvas
  size `W × H`, not the image size (confirmed by `a-*`, `b-*`, `pad-s50`).

---

## 2. Source points and heights

Every source pixel `(x, y)` with `0 ≤ x < sw, 0 ≤ y < sh` is one surface
point. There is no subsampling (confirmed: identity transform reproduces the
image, `id-*`).

### 2.1 Height (confirmed)

    c' = max(c, waterline)              # waterline raises low colours
    z  = (c' − Z0) · r                  # in pixel units, r = roughness/100
    Z0 = (N − 1) / 2                    # N = entries in the input's global colour table

`Z0 = 127.5` for 256-colour files (`zy100`, `zy50`, `zx100`, `zxm100`,
`zym100`, `zyb100`: 100 %; `Z0 = 128` gives at most 17.6 %), `Z0 = 7.5` for a 16-colour
file (`h-16`: 100 %). The height is in output pixels and **does not scale with
the image size** (`flat5` 400×300 vs `flat5s` 100×75 look different for this
reason; `zyb100` uses a 400-wide image with the same constant). Negative
roughness inverts the relief (`neg-r`, 100 %).

The **waterline also changes the drawn colour**: a pixel below the waterline is
drawn with colour `waterline` (`h-wl100`: 100 % only when both height and
colour are clamped). Whether the test is `<` or `≤` can't be told (equal
results).

### 2.2 Colour 0 (confirmed)

Colour 0 is an ordinary colour: its points are drawn and overwrite earlier
ones (`h-st0`: rows of colour 100 interleaved with rows of colour 0, both
landing on the same output rows, give an all-0 image; `h-st7` with colour 7
instead of 0 behaves the same way).

### 2.3 Transparent range (confirmed)

With `transparent=lo/hi`, a point whose (waterline-adjusted) colour satisfies
`lo ≤ c ≤ hi` is **never drawn**, but it **is still a vertex** for lines and
triangles of its neighbours (§6). The range is off when `hi = 0`:
`transparent=0/0` (the default) draws colour 0 (`h-st0t`), `transparent=0/1`
hides 0 and 1 (`h-st0t1`), `transparent=100/100` hides 100 (`h-st100t`),
`transparent=7/0` hides nothing (`h-st5t`). `h-tr` (50/80): 100 % with the
inclusive range, 99.5 % with either end exclusive. `combo`, `combo1`: 100 %
only with the "still a vertex" rule.

### 2.4 Preview modes

Not examined (batch mode has no preview). **Unknown.**

---

## 3. Planar projection

### 3.1 Transform (confirmed)

Each point becomes, with `hx = (W − 1)/2`, `hy = (H − 1)/2`:

    u = (x − hx) · sx
    v = (y − hy) · sy          # v grows downwards, like screen rows
    w = z                      # §2.1; w grows towards the viewer

then is rotated about x, then y, then z (order confirmed: `o-345`, `o-scl`
100 % only for this order; the five others give ~32 %):

    about x by ax:  v, w = v·cos ax − w·sin ax,   v·sin ax + w·cos ax
    about y by ay:  u, w = u·cos ay + w·sin ay,  −u·sin ay + w·cos ay
    about z by az:  u, v = u·cos az + v·sin az,  −u·sin az + v·cos az

Signs: with `rotation=0/0/30` a point to the right of centre moves up on the
screen (`rz30-*`); with `0/30/0` a high point moves right (`zy100`); with
`30/0/0` a high point moves up (`zx100`). The `w` formulas only matter with
perspective and are confirmed there (§3.3).

### 3.2 Shift and screen position (confirmed)

    shx = trunc(dx · W / 100)          # xyshift=dx/dy
    shy = trunc(dy · H / 100)
    u  += shx;   v −= shy              # +dy moves the image up
    X = trunc(W/2 + 0.5 + u)
    Y = trunc(H/2 + 0.5 + v)

- With the identity transform (`rotation=0/0/0 scalexyz=100/100/0`) every
  point lands **one pixel right and one down**: `X = x + 1`, `Y = y + 1`; the
  last column and row fall off the image (`id-*`, `sx50-*`, `sy50-*`: 100 %).
- Shift truncation: `xyshift=10/20` on 200×150 moves by +20/−30 pixels
  (`sh-*`); `9 %` of 150 = 13.5 → 13 (`o-sh`); `−3 %` of 150 = −4.5 → −4,
  i.e. toward zero (`sh2`).
- Points that land outside `0 ≤ X < W`, `0 ≤ Y < H` are dropped (not clamped).
  For off-screen line endpoints the `trunc` toward zero matters (fill 1 with
  perspective: 99.8 % with `trunc`, 97.1 % with `floor`).
- The shift is applied **before** the perspective division (§3.3) when
  perspective is on (`cb-all`, `cb-nowl`: 100 % only this way; 46 % if the
  shift is added afterwards).

### 3.3 Perspective (confirmed)

With `perspective=p` (p > 0), compute the depth range of the rotated bounding
box of the whole canvas and colour range:

    corners = (±hx·sx, ±hy·sy, ±Z0·r), all 8, rotated as in §3.1
    zmax, zmin = max and min of their w
    D = trunc(p/100 · (zmax − zmin))

and for every point (after rotation and after adding the shift):

    den = D − zmax − w
    if den ≤ 0: the point is not drawn
    f = D / den
    pcx = floor(W/2) − 0.5 − hx        # 0 for even W, −0.5 for odd W
    pcy = floor(H/2) − 0.5 − hy
    u = (u − pcx)·f + pcx
    v = (v − pcy)·f + pcy

then §3.2's `X`, `Y`. Notes:

- The eye sits `D` beyond the nearest corner of the box; points on the plane
  `w = zmax − D` keep their size. Unrotated, this reduces to
  `f = 255/(255 − c)` for `p = 100` and any roughness ≥ 20 % (`pf-*`, `pg-*`),
  and an image of colour 255 vanishes (`pf-255-100-*`: empty).
- `D` is truncated: the measured eye distance in colour units at `p = 100` was
  200, 250, 240, 250, 255, 253.3 for roughness 1, 2, 5, 10, 20, 30 %, which is
  `trunc(2.55·r·100)/r` (`t-x0-r1`, `u-r*`).
- Roughness 0 with perspective gives `D = 0` and an **empty image**
  (`p150-*`, `p60-*`).
- The odd-size centre `pcx`/`pcy` was found on 100×75 images (`pf0b`: 81 %
  with centre `hy`, 100 % with `pcy`); even sizes are unaffected.
- Evidence: `l-*`, `m-*` (lattices with unique colours and permuted heights,
  fitted as a general projective map), `q-*`, `t-*`, `v-*`, `w-*`, `pz-x`,
  `pr-wav`, `pd-wav`, `pf0`, `pf0b`, `lp-*`, `cb-*`: all 100 % or ≥ 99.99 %.

### 3.4 Arithmetic precision (hypothesis / unknown)

The formulas above are exact for scale factors such as 100, 90, 50, 400 %.
With `scalexyz=60/60/…` XFractint resolves exact ties (a computed position that
is mathematically an integer) **downwards for some points and upwards for
others** (`f0-id60`: sx = 2, 7, 12, 17, 57, 62, 67, 72 go down; 22–52 and
77–97 go up). Plain IEEE double evaluation of our formulas, single precision,
and fixed-point scale factors each reproduce only part of this. The simulator
gets 92–96 % on those cases (`f0-id60*`, `f0-x30s60`, `f4b-x30`, `f4b-xm`) and
the same effect explains the last few pixels of `u-r*`, `pz-x`, `o-yz`.
**Unknown**: the exact arithmetic (likely x87 extended precision with a
particular evaluation order). An implementer should use doubles in the order
shown; mismatches are limited to exact ties.

---

## 4. Drawing order (confirmed)

Source points are processed row by row from the top, left to right within a
row. Each point draws its primitives (§6) immediately, overwriting whatever is
there: **the later point wins**. There is no depth buffer and no hidden-surface
test in planar mode (e.g. `sx50-*`: of two source columns landing on one
output column the right-hand one remains; `zy100` etc. depend on this).

---

## 5. Spherical projection (`sphere=yes`)

### 5.1 Keys share storage with the planar keys (confirmed)

`longitude=a/b` is stored in the rotation-x/y fields (128/130),
`latitude=a/b` in rotation-z/scale-x (132/134), `radius=n` in scale-y (136)
(`sp-lon`, `sp-lat`, `sp-rad`). Without these keys the planar defaults are
used, so the default sphere is longitude 60→30, latitude 0→90, radius 90
(`sp-def`). Hypothesis: `rotation=`/`scalexyz=` given with `sphere=yes` set
the same values.

### 5.2 Mapping with roughness 0 (confirmed, 99.1–99.6 %)

    lon = lon_a + (lon_b − lon_a) · y / H          # note: divided by H, not H−1
    lat = lat_a + (lat_b − lat_a) · x / W
    R   = radius/100 · H/2
    X = trunc(W/2 + R·sin(lat))
    Y = trunc(H/2 + R·cos(lat)·cos(lon))
    drawn only if cos(lat)·sin(lon) > 0

- The latitude key spans the image **columns** and moves points left/right;
  the longitude key spans the **rows**. Example: `longitude=-90/90
  latitude=-90/90` draws the lower half of a disc of radius `H/2`
  (`s1-*`); `longitude=-180/180` a full disc (`s2-*`).
- There is no `+ 0.5` here, unlike §3.2 (lattice fit `sl1`–`sl4`: 143 of 144
  unique points exact with this form; with `+0.5` in either axis only 90–93).
- Points with `cos(lat)·sin(lon) ≤ 0` (the far side) are not drawn: in `sl1`
  no lattice point from the upper half of the source survives.
- Simulator: `s1-*`, `s2-*`, `s3-*` 99.1–99.6 %, `sl*` ≥ 99.98 %. The
  remaining pixels are ±1 column differences where many source pixels share an
  output pixel, i.e. precision (§3.4). **Unknown** exact arithmetic.

### 5.3 Roughness on the sphere (hypothesis)

Height changes the radius. At the point facing the viewer the fitted radius is

    R(c) ≈ R · (1 − r · (N − 1 − c) / 800)

(`srb-*`: lowest visible row moves by ~22 px between colour 1 and 255 at
radius 100, roughness 100 %, and ~11 px at 50 %; lattice fit `slr*` best at
1/800). But the shape is **not** a scaled sphere: with roughness > 0 the left
and right edges are cut off by vertical walls even for colour 255 (`srb-255-100`
spans columns 43–156 instead of 25–174; 35–164 at roughness 50 %), so some
points near `lat = ±90°` are hidden by a rule we did not find. Simulator with
the formula above: 89–99.9 % (`sr*`, `srb*`, `slr*`, `sp-*`). **Unknown**:
the exact height law and the culling rule. Perspective, shift and fill types
other than 0 on the sphere were **not examined**.

---

## 6. Fill types

`filltype` accepts −1 … 6. Types 5 and 6 use a light source and belong to
another spec (our simulator gets 16 % on them, as expected). In all types the
primitives are drawn in the order of §4, with the colour of the **current**
point `P = (x, y)`, and a transparent `P` (§2.3) draws nothing. Points that
failed the perspective test (`den ≤ 0`) do not exist as vertices
(hypothesis: our simulator does this; no case isolates it).

### 6.1 Line rasteriser (confirmed)

All lines are drawn by a rounded DDA from the first to the second endpoint:

    n = max(|x1 − x0|, |y1 − y0|)
    for i in 0..n:  plot( floor(x0 + (x1−x0)·i/n + 0.5),  floor(y0 + (y1−y0)·i/n + 0.5) )

(`fill1`: 100 % with this DDA, 88–98 % with Bresenham variants.) Both
endpoints are plotted. Pixels outside the image are skipped.

### 6.2 Triangle rasteriser (confirmed)

A triangle is filled by drawing its three edges with the DDA of §6.1 (A→B,
B→C, C→A) and, for every row touched, filling from the leftmost to the
rightmost edge pixel of that row (`fill3`: 100 %; a geometric scan-line fill
gives 73 %).

### 6.3 Type 0: points (default; confirmed)

Plot `P` at `(X, Y)`. 100 % on every planar case without ties (§8).

### 6.4 Type 1: lines along rows (confirmed)

For `x = 0` plot the point; for `x > 0` draw a line from the point `(x−1, y)`
to `P` in `P`'s colour. A transparent left neighbour is still used as the
start (`combo1`). Rows are **not** connected vertically (`g1`, `k1-*`).

### 6.5 Type −1: coarse grid (confirmed)

Only grid points are used:

    ystep = max(1, trunc(H / 20))
    xstep = max(1, trunc(ystep · W · 3 / (4 · H)))      # integer division
    grid rows    = 0, ystep, 2·ystep, … plus the last image row sh−1
    grid columns = 0, xstep, 2·xstep, … plus the last image column sw−1

Grid points are processed row by row. Each grid point `P` draws a line from
its left grid neighbour to `P`, then a line from its upper grid neighbour to
`P`, both in `P`'s colour; the very first grid point is plotted as a point.
Evidence: 23 image sizes (`sg-*`), `g-1`, `gr-1`, `fill-1`, `dm1`, `pfm1`:
100 % except when the canvas was enlarged (§1.3; `sg-105-75` … 97.5–98.9 %,
mismatches only at the bottom image rows), where the grid's last row is
**unknown**. Step examples (canvas W×H → xstep/ystep): 60×45 → 2/2,
98×75 → 2/3, 100×75 → 3/3, 101×77 → 2/3, 105×79 → 2/3, 107×80 → 4/4,
110×83 → 3/4, 130×98 → 3/4, 200×150 → 7/7, 400×300 → 15/15, 640×480 → 24/24.

### 6.6 Type 3: flat triangles (confirmed)

Row 0 is drawn like type 1. For `y > 0`, with `U` = point `(x, y−1)`,
`UR` = `(x+1, y−1)`, `L` = `(x−1, y)`:

    if x+1 < sw: fill triangle (P, U, UR)  in P's colour
    if x   > 0 : fill triangle (P, L, U)   in P's colour

using §6.2. So each cell between two rows is split along the diagonal from its
upper-right to its lower-left corner. Evidence: `g3`, `gr3`, `k3-*`, `fill3`,
`pf3`, `combo`: 100 %.

### 6.7 Type 2: interpolated triangles (partly unknown)

Type 2 covers **exactly the same pixels as type 3** (confirmed: identical
non-background masks for `fill2`/`fill3`, `k2-rx`/`k3-rx`, `g2`/`g3`). Inside,
colours vary smoothly between the vertex colours, but **not** as plain
Gouraud interpolation:

- On linear ramps (`k2-rx`, `k2-ry`) values are within ±1 of linear
  interpolation, with vertex pixels typically one index off.
- With one vertex of colour 200 among 100s (`k2-one`) the peak pixel shows 150,
  and pixels on edges between two colour-100 vertices show values up to 133
  (`(2·100 + 200)/3`) and 116 (`(5·100 + 200)/6`).
- Uniform regions keep their colour (`h-chkf2` 90 % with the type-3 rule).

**Unknown**: the colour rule. The simulator draws type 2 like type 3
(0–99.4 %, the high values on mostly-background images).

### 6.8 Type 4: bars down to the base (confirmed)

For every point draw a DDA line from `P` to its **base point** `B`, in `P`'s
colour. `B` is the same source pixel transformed with colour 0
(`w = −Z0·r` before rotation, with the same shift and perspective), and **`B`
is clamped into the image** (`0 ≤ BX ≤ W−1`, `0 ≤ BY ≤ H−1`) before drawing;
`P` is not clamped. Evidence: `f4b-def` 100 % with the clamp, 73 % without;
`f4b-y30`, `f4b-p`, `f4-*`, `fill4`, `combo4`: ≥ 99.9 %; `f4b-x30`, `f4b-xm`
(scale 60 %, §3.4) 95 % and 90 %. With roughness 0 type 4 equals type 0
(`g4`, `k4-*`).

---

## 7. The saved GIF (confirmed)

- Size: the canvas (§1.3). Layout as in
  [gif-fractint-blocks.md](gif-fractint-blocks.md) §1: image, then only a
  `fractint001` block (no `002`, `003`, …).
- **Background** (pixels no primitive touched) is **black**. The saver stores
  it as the lowest palette index whose colour is black (`h-k250`: black put at
  entry 250 → background 250; `h-16k3` → 3; `h-16` → 16, the first black entry
  of the default palette part), or 0 when the palette has no black entry (all
  `mkin.py` palettes). Drawn colours are the source indices (after waterline).
  The usual duplicate-colour remapping of the saver
  ([types.md](types.md) §1.3) applies.
- Palette: 256 entries. Entries `0 … N−1` are the input's table; for a
  16-colour input, entries 16–255 are XFractint's default palette (`h-16`:
  entry 16 = black, 17 = (20,20,20), …, as in `in/mandel.gif`).
- `fractint001` (offsets as in gif-fractint-blocks.md §2.1):
  - type (10) = **7** for every 3D output, whatever the input was (`m3d-def`:
    mandel's 4 becomes 7; files without a block also give 7);
  - corners, maxiter, pass mode and other fractal fields are copied from the
    input's block when it has one (`m3d-def`, `blk3d`: corners
    −2.5/1.5/−1.5/1.5), else corners are −1/1/−1/1 with 3rd corner −1/−1
    (`flat5`);
  - 3D fields 126–162 and 254–256 hold the 3D parameters actually used (from
    the command line or the defaults, never from the input file);
  - status (202) = 4, extension length (204) = 521, screen size (70/72) =
    800/600 for every size, offset 248 = 1, and **offset 252 = 1** (0 in normal
    renders and in a plain reload `nl-80`): a "3D-transformed" marker
    (hypothesis for the meaning, confirmed for the value);
  - seed (98) and time (214) vary as usual.

### 7.1 Interrupt and resume (unknown)

Batch 3D runs always finish and save status 4 without a `fractint002` block.
Interactive runs with `3d=yes` (key scripts `keys3d.sh`: up to six Enter
presses, then `s`) never produced a save; three of them crashed XFractint
(segmentation fault after an X `BadWindow` error). Whether a 3D transform
can be interrupted, saved part-way and resumed is **unknown**.

---

## 8. Verification

`tools/blackbox/3d-transform/sim3d.py corpus/…/cases.txt in out` implements
§§1–7 (with §5.3 and type 2 as stated) and compares every case pixel by pixel
(colour indices; background mapped as in §7). 302 cases have an output
(303 listed; `fill7` has none):

| Group | Cases | 100 % | Range |
|---|---|---|---|
| fill 0, no perspective (incl. waterline, transparency, shift, 16 colours, canvases) | 86 | 83 | 99.993–100 |
| fill 0, perspective | 80 | 66 | 99.64–100 (ties) |
| fill 1 / 3 / −1 / 4, no ties | 57 | 45 | 97.49–100 (all below 99.8 are type −1 on enlarged canvases) |
| scale 60 % (ties, §3.4) | 11 | 6 | 89.6–100 |
| fill 2 (§6.7, colour rule unknown) | 9 | 0 | 0–99.4 |
| sphere, roughness 0 | 11 | 0 | 99.08–99.997 |
| sphere, roughness > 0 (§5.3) | 38 | 2 | 89.3–100 |
| light fills 5/6 (other spec), plain reload | 3 | 0 | — |
| canvas size not predicted (§1.3) | 7 | — | — |

Default settings (`rotation=60/30/0 scalexyz=90/90/30`, fill 0, planar) are
reproduced exactly (`m3d-def`, `flat5*`, `o-def`, `fill0`, `blk3d`: 100 %).

---

## 9. Open questions

1. The canvas rule for images far from 4:3 (§1.3).
2. Exact arithmetic at ties (§3.4, §5.2).
3. Type 2 colour interpolation (§6.7).
4. Sphere roughness law and edge culling; sphere with perspective, shift or
   other fill types (§5.3).
5. Interrupt/resume of 3D renders (§7.1).
6. Type −1 last grid row on enlarged canvases (§6.5).
7. Whether `rotation=`/`scalexyz=` act as the sphere keys (§5.1).
