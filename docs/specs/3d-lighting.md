<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# 3D light-source fills and stereo ("funny glasses") output

Status: **Phase 5 draft, black-box only.** Everything here comes from images
and files that XFractint (Debian `xfractint 20.4.10-5`, amd64 under qemu)
produced from inputs we made, and from its accept/reject behaviour on the
command line. No Fractint, XFractint or Iterated Dynamics source code, help
text or documentation was read. See [CLEANROOM.md](../CLEANROOM.md). Names
are our own.

Companion specs: 3D geometry (height mapping, rotation, perspective, fill
types 0–4) is being specified separately in `3d-transform.md`. This document
only uses the simplest geometry, a **top view** (`rotation=0/0/0`, no
perspective), and states the few geometric facts it needs with its own
evidence. Block offsets refer to [gif-fractint-blocks.md](gif-fractint-blocks.md);
the palette-saver artefact is [types.md](types.md) §1.3.

Markers: **confirmed** = pixels or bytes show it directly and our simulator
reproduces them; **hypothesis** = fits the evidence but is not proven or not
exact; **unknown** = not determined.

## Sources

- Evidence: `corpus/xfractint/3d-lighting/` (GIFs, `cases.txt` with the exact
  arguments of every file, `README.md`, `SHA256SUMS`).
- Tools: `tools/blackbox/3d-lighting/`
  - `mkinput.py` writes the height-field inputs (`inputs/*.gif`, all 400×300,
    palette `inputs/uniq.map`, which has 256 colours that stay distinct at 6 bits
    per channel); `mkpal.py` writes the palette-test inputs `inputs/pal/*`.
    `inputs/mandel.gif` was made by XFractint itself
    (`type=mandel maxiter=255 viewwindows=2`).
  - `render.sh` / `run-cases.sh` run
    `xfractint batch=yes 3d=yes filename=in.gif ARGS savename=ff.gif` in a fresh
    `mktemp` directory.
  - `sim.py` (light fills, top view), `stereo_check.py` (stereo modes),
    `score_flat.py` (colour formula on flat surfaces), `verify.sh` (runs all
    three over the corpus).
  - `keyrun.sh` / `keys.sh` / `xwd2png.py`: interactive runs with key presses
    and screen captures (§9).
- Inputs used most: `flat100` (every pixel 100), `flat0`, `stepx` (100 left of
  input column 200, 110 from column 200 on), `stepy` (same in rows, step at
  row 150), `spike` (100 with a single 110 at (200,150)), `sx1`/`sy1` (ramps
  rising one colour per input pixel), `plateau` (50 with a 100×100 square of
  200), `pyramid`, `noise` (pseudo-random 20..219), `rampx`/`rampy`.

In this document the input pixel at column `i`, row `j` has colour index
`h(i,j)`; rows count downward as in the GIF.

---

## 1. Basics common to both features

1. **Output size and palette.** With `viewwindows=2` and our 400×300 inputs
   the output is 400×300. The output keeps the **input file's palette** (all
   fill types and the light fills; checked byte-for-byte after 6-bit scaling,
   `fl5A0`, `fl6A0`). The stereo modes 1 and 2 are the exception (§7.5).
   *Confirmed.*
2. **Colour index is height** and the RGB values of the palette are not used:
   the same flat input with the palette reversed (`pal-rev`: 86, as `fl5`)
   or with entry 100 set to black, white, red, green, blue or grey
   (`s1_p100_*`, `s2_p100_*`) gives the same indices. (`pal-grey`, a plain grey
   palette, saves 84 instead of 86 only because entries 84–87 become equal at
   6 bits, §1.4.) *Confirmed.*
3. **Background** is index 0 wherever nothing is drawn. The `background=`
   key is accepted but changed neither pixels nor palette
   (`k_background_10_20_30`). *Confirmed for the GIF.*
4. **Saver artefact.** Pixels are saved through the first palette index with
   the same RGB (types.md §1.3). Our input palette has no duplicates, so this
   only matters for stereo output, which uses the default palette (§7.5).
5. **Top-view geometry used here** (`rotation=0/0/0`, no perspective):
   with `scalexyz=100/100` the input pixel `(i,j)` sits on output pixel
   `(i+1, j+1)` (fill 0 draws columns 1..399 and rows 1..299, `e_fl0`). With
   the default `scalexyz=90/90` the drawing covers columns 20..380 and rows
   15..285 (`fl0`). Height does not move pixels in a top view. *Confirmed for
   these two scales only; general geometry belongs to 3d-transform.md.*

---

## 2. Which fill types use a light source

| `filltype=` | Behaviour seen |
|---|---|
| −1 .. 4 | no lighting; output colours are input colours (or interpolations of them). Not covered here. |
| **5** | light source, light vector **fixed relative to the viewer** (§4.1) |
| **6** | light source, light vector **transformed together with the surface** (§4.2) |
| 7 and above | rejected: XFractint stops with an unknown-argument message and saves nothing (`t7`, marked REJECTED in `cases.txt`) |

Both light fills produce the same kind of output: each surface point gets a
colour index from its surface normal (§3, §5), and the image is filled by the
same interpolating rasteriser as fill type 2 (§6). *Confirmed.*

---

## 3. Surface normal

For every input point `(i,j)` with `i ≥ 1` and `j ≥ 1` the normal is built
from the point, its **left neighbour** `(i−1,j)` and its **upper neighbour**
`(i,j−1)`:

```
z(i,j)  = h(i,j) * R / 100                 R = roughness (scalexyz 3rd value), default 30
gx      = (z(i,j) − z(i−1,j)) / (Sx / 100) Sx = x scale in percent
gy      = (z(i,j) − z(i,j−1)) / (Sy / 100) Sy = y scale in percent
N(i,j)  = (−gx, +gy, 1)                    then normalised
```

- Axes of `N` and of the light vector: **x to the right, y up the screen
  (opposite to rows), z towards the viewer.** A surface rising to the right
  (`sx1`) and one rising downwards (`sy1`) are equally dark under the default
  light `1/−1/1` (`rx5`/`ry5`, `sx1f5r*`/`sy1f5r*`); `sy1` lit by `0/1/1` is
  brighter than flat (`n_y1`). *Confirmed.*
- Units: the gradient is in "output pixels of rise per output pixel": doubling
  the roughness or halving the x scale doubles `gx` (`n_r60`, `n_x50`, `n_s50`,
  `n_s200`, `n_xy50`, `n_y50`). *Confirmed* for top views at 50–200 %. Whether
  the input-to-screen size ratio enters when the input is not the window size
  is **unknown** (all our inputs were 400×300 in a 400×300 window).
- Points in input row 0 or column 0 get no normal and are not drawn (§6).
- With a rotation the normal turns with the surface for fill 5: a flat
  surface under light `1/0/1` gives the colour of a normal tilted 30° about x
  (`o_x30f5`, 98) or about y (`o_y30f5`, 8, i.e. tilted towards +x), and is
  unchanged by a z rotation (`o_z90f5`); a ramp rotated 90° about z is lit as
  a y-ramp (`o_sxz90f5`). *Confirmed for these rotations*; the rotation sign
  conventions and the general case are left to 3d-transform.md.

---

## 4. Light vector

`lightsource=x/y/z` (three integers, stored as i16 at fractint001 150/152/154;
default `1/−1/1`). The vector is normalised; only its direction matters
(`1/0/1` = `100/0/100`, `1/0/3` = `10/0/30` = `100/0/300`, `1/1/1` =
`10/10/10`: `m0_*`, `m20_*`). `0/0/0` behaves as if the cosine were 0
(`z_l000`, `z_l000s`: ambient-only colour everywhere). A fourth value is
rejected. *Confirmed.*

### 4.1 Fill type 5

The normalised light vector `L` is used as given, in viewer space:
`c = N · L` with `N` from §3 after the view rotation. *Confirmed* (flat and
ramp tests at rotation 0, and the rotations listed in §3).

### 4.2 Fill type 6

The light vector is first multiplied by the same scale factors as the surface
and then rotated with it:

```
L6 = normalise( Lx * Sx/100,  Ly * Sy/100,  Lz * R/100 )
```

- Rotation 0: flat surface, light `1/0/1`: 181 at scale 100/100/30
  (`q_f6a`), 123 at 50/50/30 (`q_f6b`), 74 at 100/100/100 (`q_f6c`, same as
  fill 5), 181 at 100/50/30 (`q_f6d`). The default `1/−1/1` gives 156 for a
  flat surface at the default 90/90/30 (`fl6`). *Confirmed.*
- With a rotation the flat-surface colour does not change (`o_x30f6`,
  `o_y30f6`, `o_z90f6` all 173, the same as rotation 0 at 90/90/30), and a
  ramp keeps its relation to the light (`o_sxz90f6`). So the light turns with
  the surface. *Confirmed for these cases.*
- A consequence: for a slope whose normal is exactly perpendicular to `L6`
  (e.g. `sx1` with light `1/0/1` at 100/100/30), `c` is 0 up to rounding and
  XFractint gives 253 or 254 in a position-dependent pattern (`q_f6e`,
  `sx1f6r30`). We cannot reproduce which (≈15 % of pixels differ by 1).

---

## 5. Brightness to colour index

```
c      = max(0, N · L)                         (cosine; 0 when facing away)
A      = ceil(255 * ambient / 100)             ambient = 0..100, default 20
colour = max(1, floor( 254 * (1 − c) * (255 − A) / 255 ))
```

So **more light gives a lower index**: a surface facing the light gets 1, a
surface facing away (or edge-on) gets `floor(254·(255−A)/255)` (254 with
ambient 0, 203 with the default 20). Only indices 1..254 occur. The palette's
RGB values play no part (§1.2).

- **Ambient 0: confirmed exactly.** 59 of 59 flat-surface cases (31 cosines
  from 0 to 1, `sw5k*`, `d_k*`, `g_a0_*`, `h*_0`, `m0_*`) and every
  ambient-0 full image the simulator reproduces (§10).
- **Ambient > 0: hypothesis, exact in 71 % and otherwise ±1.** 220 of 308
  flat cases match; all others are off by exactly one. The form
  `floor(K * (255 − A) / 255)` with `A = ceil(2.55·ambient)` fits the two
  100-point ambient sweeps (`h100_0..99`, `h10_0..99`) **exactly**, but only
  with `K` = 228.10–228.16 for light `100/0/10` (where `254(1−c)` = 228.73) and
  `K` = 74.70–74.74 for `10/0/10` (formula 74.39). Per-light `K` ranges we
  measured:

  | light (flat surface) | cos c | K range from data | 254(1−c) |
  |---|---|---|---|
  | `100/0/10` | 0.0995 | 228.10 – 228.16 | 228.73 |
  | `40/0/10` | 0.2425 | 192.25 – 192.64 | 192.40 |
  | `3/0/1` | 0.3162 | 173.00 – 173.75 | 173.68 |
  | `20/0/10` | 0.4472 | 140.31 – 140.55 | 140.41 |
  | `1/1/1`, `1/−1/1` | 0.5774 | 107.50 – 108.00 | 107.35 |
  | `10/0/10`, `1/0/1` | 0.7071 | 74.70 – 74.74 | 74.39 |
  | `5/0/10`, `1/0/2` | 0.8944 | 26.73 – 27.00 | 26.82 |
  | `1/0/3` | 0.9487 | 13.75 – 14.00 | 13.04 |
  | `2/0/10` | 0.9806 | 4.45 – 5.00 | 4.93 |

  So the light-dependent part is computed with a small error (about ±0.003 in
  `c`) that is invisible at ambient 0 for these lights. Its origin is
  **unknown**; we tried quantising the light vector, the cosine and the
  ambient to 1/64 … 1/65536 and integer pipelines with 254/255/256 scales, and
  none fits all points.
- Ambient outside 0..100 or non-numeric is rejected (`k_ambient_*`). Stored at
  fractint001 258. *Confirmed.*

---

## 6. How the vertex colours become pixels

Each drawn input point `(i,j)` gets the colour `V(i,j)` of §5. The image is
then filled exactly as fill type 2 fills input colours. In the top view at
scale 100 (point `(i,j)` on pixel `(i+1,j+1)`):

```
pixel(x, y), i = x−1, j = y−1:
  if V(i,j), V(i,j+1), V(i−1,j+1) all exist:  (3·V(i,j) + 2·V(i,j+1) + V(i−1,j+1)) div 6
  else if V(i,j), V(i,j+1) exist (i = 1):      (3·V(i,j) + V(i,j+1)) div 4
  else:                                       0 (background)
```

(`div` = integer division of non-negative values.) The first branch is a fixed
interpolation inside the triangle `(i,j)`, `(i,j+1)`, `(i−1,j+1)`. Evidence:

- Fill 2 with zero roughness on `noise` (every vertex a different colour):
  the formula matches **118604/118604** interior pixels; `floor` of the exact
  weighted mean matches only 98.5 % (`nz2`). *Confirmed.*
- Light fills: the formula with §3–§5 reproduces `spike`, `stepx`, `stepy`,
  `sx1` and their variants pixel for pixel (§10). The first drawn column uses
  the second branch (`e_sy5`, column 2). *Confirmed.*
- Other scales (e.g. the default 90 %, or 150–400 %, `x_sc*`, `y_sc*`,
  `p_sc*`) mix neighbouring vertices with weights that vary with the column
  and row; the same rasteriser is assumed (**hypothesis**) and its general
  form is left to the fill-type-2 description in 3d-transform.md.
- **Unknown:** in `v_md5` (the XFractint mandel input, roughness 10) 129 pixels
  next to steep pits stay 0 where the formula draws a dark colour; the cause
  was not found.

---

## 7. Other keys that affect light fills

### 7.1 `smoothing=` (fractint001 156, default 0)

A running average of **unit** normals along each row, restarted at the start
of every row; nothing is averaged vertically:

```
n(i,j) = N(i,j) / |N(i,j)|
M(1,j) = n(1,j);   M(i,j) = (s · M(i−1,j) + n(i,j)) / (s + 1)   for i > 1
c = max(0, M(i,j) · L) / |M(i,j)|
```

`sm1`, `sm2`, `x_sm` (s = 3), `sm5` and `smy3` (a horizontal step, which is
unaffected) match 100 %; on `noise` with s = 2 98.6 % match and the rest are
±1 except 31 pixels (`v_nz5s`). *Confirmed* (the ±1 has the same unknown
precision cause as §5).

### 7.2 `haze=` (fractint001 260, 0..100)

No effect on any saved pixel we produced: top views, the default 60/30
rotation and perspective 150 at haze 0, 20, 50, 100 are identical
(`hz*`, `hzp*`, `hzpp*`). 101 is rejected. What it does (perhaps only for the
full-colour output, §7.4) is **unknown**.

### 7.3 `randomize=` (fractint001 262, 0..7)

No effect on light fills (`rz1`, `rz3`, `rz7`, `rz7s`, `v_py5r` = `v_py5d`).
It does change fill 0 and fill 2 output (`rz7f0`, `rz7f2`), which belongs to
3d-transform.md. 8 and −1 are rejected. *Confirmed for light fills.*

### 7.4 Other keys tried

| Key | Result | Stored |
|---|---|---|
| `coarse=10` | accepted, no pixel change | fractint001 158 (default 20) |
| `transparent=1/2` | accepted, no change on `flat100` | 254/256 |
| `preview=yes`, `showbox=yes`, `brief=yes`, `sphere=no`, `usegrayscale=yes`, `monitorwidth=10`, `3dmode=monocular`, `lightname=ff.tga`, `background=10/20/30` | accepted, no pixel change in our cases | nothing found |
| `fullcolor=yes` | accepted; also writes `fract001.tga` (400×300, uncompressed 24-bit, type 2, descriptor 0x20; every pixel the same RGB for `flat100`). Contents otherwise **unknown** | nothing found |
| `ray=1` | accepted; writes `fract001.ray` (a scene description) and the GIF is different (`k_ray_1`). Out of scope | nothing found |
| `longitude=`, `latitude=`, `radius=` | accepted; reuse the rotation/scale slots (sphere keys, 3d-transform.md) | 128/130, 132/134, 136 |
| `light=`, `glassestype=`, `grayscale=`, `targa_out=` | rejected | |
| `lightsource` with 4 values, `ambient=−5/101/a`, `haze=101`, `randomize=8/−1`, `filltype=7`, `stereo=−1`, `bright=1`, `crop=1/2/3` | rejected | |

---

## 8. Stereo ("funny glasses")

`stereo=` 0..4 (−1 rejected; fractint001 180). The two eye images below are
called **A** and **B**. A is the one moved to the right by positive
`interocular`/`converge`; which physical eye/filter colour it is meant for is
**unknown** (the saved palette does not say, §8.5).

### 8.1 Eye colour values (modes 1 and 2)

For a surface colour `c` (input colour for fills 0–4, the §5 colour for light
fills; `st1f5`/`st2f5` vs `st3f5` match 100 %):

```
E(c) = 128 − ceil(c / 2)            0..128
```

- `stereo=1` ("alternate"): pixels with `(x + y)` even show eye A with value
  `E(c)`; pixels with `(x + y)` odd show eye B with value `E(c) + 128`. A pixel
  where its eye's image is absent is 0. *Confirmed* (`t1_*`, `st1`, `s1_*`).
- `stereo=2` ("superimpose"): every pixel is `16·ℓB + ℓA` with
  `ℓ = E(c) div 9 + 1` (1..15) for the eye's colour, 0 where that eye's image
  is absent. *Confirmed* (`t2_*`, `st2`, `s2_*`, 15 levels seen).
- Values pass through the saver with XFractint's **default palette**, so 16,
  248–255 are saved as 0 and 31 as 15 (types.md §1.3). In mode 1 this makes eye
  B invisible for `c` ≤ 15 (B ≥ 248) and turns A = 31 (`c` = 193, 194) into 15.
- **Edge cases not reproduced:** a surface of colour 0 is drawn (A = 128 in
  mode 1; our predictor cannot tell it from background); in mode 2, where both
  levels are 15 (`c` ≤ 4) the saved value is 0 left of column 29 and 240 from
  column 29 on in `t2_ry0` (156 pixels). **Unknown.**

### 8.2 Eye offsets (top view)

Without perspective the two images are the plain image moved sideways, A by
`+d` and B by `−d` pixels:

```
d = W · (interocular / 2 + converge) / 100       W = image width (400 here)
```

Measured: interocular 3, 5, 10 → d = 6, 10, 20; converge 1, 5, −5 → 4, 20,
−20; interocular 5 + converge 5 → 30 (`t1_pl_i*`, `t1_pl_c*`, `t1_pl_i5c5`).
Height does not change `d` in a top view (base and square of `plateau` move
together, `t1_pl_i5`). Defaults are 0 and 0, so a default
stereo image has both eyes on top of each other. Column 0 is never drawn.
*Confirmed for W = 400* (dependence on other widths is a hypothesis).

With perspective the offset depends on depth: at `perspective=150`,
interocular 5 the near square moves ±21 px instead of ±10 (`t1_pl_i5p` vs
`t0_pl_p`). The formula is **unknown**; it needs the perspective model of
3d-transform.md.

### 8.3 `crop=` (fractint001 164–170, default 4/0/0/4)

Four percentages `a/b/c/e` that clip the eyes: eye A is kept only where
`W·a/100 < x < W − W·b/100`, eye B only where `W·c/100 < x < W − W·e/100`.
Default 4/0/0/4 therefore cuts A at x ≤ 16 and B at x ≥ 384 (`t1_pl_cm5`);
`10/0/0/10`, `0/10/10/0`, `0/0/0/0` match (`t1_crop1..3`). Three values are
rejected. *Confirmed.*

### 8.4 `bright=` (fractint001 172/174, default 80/100)

No effect on saved pixels or palette (`t1_br1`, `t2_br1`, `t2_br2`); one value
is rejected. Presumably it scales the display palette of each eye
(**hypothesis**, not observable in the GIF).

### 8.5 Palette of stereo images

Modes 1 and 2 save XFractint's **default palette** (identical to the palette
of `corpus/xfractint/complete/mandel-base.gif`), not the input palette and not
a red/blue glasses palette. If a glasses palette is used for display, it is
not saved. *Confirmed* (`st1fl`, `st2fl`, `s1_*`, `s2_*`).

### 8.6 Modes 3 and 4

- `stereo=3`: in batch mode the pixels and palette are identical to
  `stereo=0`; only fractint001 180 differs (`st3` vs `st0`, `t3_pl`). Whatever
  it does interactively is **unknown**.
- `stereo=4`: the output is 800×600 even with `viewwindows=2`. The plain
  400×300 image appears twice, moved down by 100 rows, once at x + 0 and once
  at x + 400 (`k_stereo_4`, `t4_pl`: content in columns 20..380 and 420..780,
  rows 115..385). With interocular 0 the halves are identical; which half is
  which eye was not tested. *Confirmed layout for 400×300 only.*

---

## 9. Saved blocks, reload, interrupt

### 9.1 `fractint001`

A 3D render is saved with one `fractint001` block (504 bytes, no
`fractint002`; status 4 = complete, pass mode `g`). Compared with the input:

- offset 10 (type number) becomes **7** (the number `type=plasma` also gets,
  `plasma-base`); offset 252 becomes **1** (it is 0 in every 2D save we know).
  *Confirmed* (`bm_ft0`, `pl_ft5`).
- If the input had a `fractint001` block (`mandel.gif`), every other field is
  copied from it (corners, maxiter, seed, …) apart from the calculation
  time at 214–217 (`bm_ft0` vs `inputs/mandel.gif`: only offsets 10, 214, 252
  differ). A
  plain GIF gives corners −1, 1, −1, 1 and maxiter 150 (`pl_ft5`).
- 3D fields record the command line: 126–156 (sphere, rotation, scale,
  roughness, waterline, fill type, perspective, shifts, light vector,
  smoothing), 158 (coarse), 164–170 (crop), 172/174 (bright), 176
  (converge), 178 (interocular), 180 (stereo), 254/256 (transparent), 258
  (ambient), 260 (haze), 262 (randomize). Negative interocular and converge
  are accepted and stored (`k_*`, `bm_*`, `t*`). *Confirmed.*
- The 6-bit-scaled input palette is written as the global colour table
  (stereo 1/2: the default palette).

### 9.2 Reload

Loading a 3D result interactively and pressing Enter recalculates it as a
**plasma** image (type 7); the 3D picture is lost and offset 252 returns to 0
(`reload-pl_ft5`). *Confirmed.* Whether XFractint can redo the 3D render
from the block alone is **unknown**; nothing we saw suggests it.

### 9.3 Interrupt and resume

**Unknown.** Batch renders cannot be interrupted. Interactively, `3d=yes
filename=…` first shows four or five parameter screens (mode/stereo, fill
type, palette file prompt for light fills, planar parameters, light
parameters); we stepped through them with Enter and captured the screen
(`keys.sh` action `p`). Under our qemu harness XFractint crashes
(segmentation fault) as soon as the 3D render starts, for fill 0, 2, 5 and 6
and for `stereo=1` (logs described in the corpus README), so no partial 3D
image could be saved.

---

## 10. Verification

`tools/blackbox/3d-lighting/verify.sh` (output in the corpus README):

| What | Cases | Pixel match |
|---|---|---|
| Light fills, top view 1:1, ambient 0: spikes (4 light directions), steps, ramps, roughness 3/30/60, fill 6, smoothing 1/2/3/5 | `e_sp5*`, `e_sp6`, `e_sx5`, `e_sy5`, `e_sx1`, `x_r3`, `y_r3`, `p_r3`, `n_r60`, `n_y1`, `q_f6a`, `q_f6c`, `x_sm`, `sm1`, `sm2`, `sm5`, `smy3` | **100 %** each (21 images) |
| Random heights, ambient 0 | `v_nz5`, `v_nz6`, `v_nz5s` (smoothing 2) | 99.6 %, 99.4 %, 98.6 % (misses ±1, plus 17/11/31 pixels further off) |
| Pyramid, ambient 0 | `v_py5` / `v_py6` | 99.9 % / 82.2 % (fill 6: c≈0 rounding, §4.2) |
| XFractint mandel input, roughness 10 | `v_md5` | 99.85 % (129 unexplained background pixels, §6) |
| Ambient > 0 | `v_py5d`, `v_nz5a`, `e_fl5`, `rp8` | 99.3 %, 76.8 %, 1.2 %, 1.2 % — all misses exactly ±1 (§5) |
| Flat-surface colour formula | 367 flat cases | 279 exact (ambient 0: 59/59; ambient > 0: 220/308), rest ±1 |
| Stereo 1/2 from the mono image (shifts, crops, bright, light fill) | 22 pairs | 100 % for 18; 99.7–99.9 % for the 4 ramp/pyramid cases (colour 0 and both-levels-15 edge cases, §8.1) |

---

## 11. Open questions

1. Exact arithmetic of the cosine/ambient step (§5 table): the ±1 cases.
2. The c ≈ 0 pattern of 253/254 with fill 6 (§4.2).
3. Background holes near steep pits (§6).
4. General rasteriser weights for scales other than 100 % (§6; see
   3d-transform.md).
5. `haze`, `fullcolor`, `background`, `lightname`: effect on the TGA output.
6. Perspective parallax for stereo (§8.2), `stereo=3` interactively, and the
   eye assignment of A/B.
7. Interrupt/resume of 3D renders (§9.3).
