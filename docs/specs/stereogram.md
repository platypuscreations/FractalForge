<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Random-dot stereograms (autostereograms)

Status: **Phase 5 draft, derived from black-box analysis only.** Every rule
below comes from pixels and parameter blocks that XFractint (Debian
`xfractint 20.4.10`, amd64, in the `fractalforge-blackbox` container) saved,
from screen captures of its own windows, and from the messages it printed.
No Fractint, XFractint or Iterated Dynamics source code, headers, help text
or documentation were consulted (see [CLEANROOM.md](../CLEANROOM.md)); the
one help screen opened by accident (F1 on the stereogram screen) was deleted
unviewed. The pseudo-random generator in §5 is the published glibc `rand()`
algorithm, used as a hypothesis and confirmed against the pixels. The
linking scheme in §4 has the shape of the published symmetric SIRDS
algorithm (Thimbleby, Inglis and Witten, 1994), also used as a hypothesis
and confirmed. Names are our own.

Labels: **Confirmed** = shown directly by the evidence, usually by
reproducing every pixel of every case. **Hypothesis** = fits all evidence
but other explanations remain. **Unknown** = not determined.

Evidence is in `corpus/xfractint/stereogram/`. A case name like `l-mix`
means `out/l-mix.gif` there; `cases.txt` gives its depth map, parameters,
recovered seed, XFractint arguments and key sequence. Depth maps are in
`depth/`. `tools/blackbox/stereogram/rdsmodel.py` is our own implementation
of this spec and `verify.py` checks every case against it (§9). Block
offsets refer to [gif-fractint-blocks.md](gif-fractint-blocks.md) §2.1.
Out of scope here: the 3D transform and red/blue ("funny glasses") stereo,
which have their own specs.

---

## 0. Summary

- XFractint turns the image on screen into a random-dot stereogram when the
  user presses **Ctrl+S**. It is interactive only; the only related
  command-line key we found is `monitorwidth=` (the assumed width of the
  image in inches) (§1).
- The image's **colour indices are the depth map**. They are stretched to
  the image's own colour range, quantised to a number of depth steps that
  grows with the depth setting and the image width, and each step gives an
  integer pixel separation (§3).
- Each row is linked independently, centre-out style: for every column `x`
  the pixels `left = x − ⌊(s + y mod 2)/2⌋` and `left + s` are constrained to
  be equal, merging with earlier constraints (§4).
- Unconstrained pixels take `rand() & 255` from glibc `rand()` seeded with
  the current Unix time; the generator is called once for **every** pixel in
  row-major order, used or not. Given the seed, every pixel is reproducible
  (§5). With the image-map option they take a tiled pattern image instead
  (§7).
- Two calibration bars, 3×16 pixels at 400×300, are drawn over the result in
  the brightest palette colour, spaced by the average separation near the
  bars (§6).
- The saved GIF has the original size, palette and parameter blocks; only
  the pixels change. Nothing records that it is a stereogram, and XFractint
  does not put the original image back on screen (§8).

## 1. How to make one

### 1.1 Interactive key (confirmed)

With an image on screen (calculated or loaded with `filename=`), **Ctrl+S**
opens a parameter screen titled for random-dot stereograms. The main menu
lists the same function under Ctrl+S. The screen has five fields, top to
bottom (our names and wording):

| Field | Default | Kind | Meaning (black-box) |
|---|---|---|---|
| depth effect | 100 | integer, may be negative | §3; negative swaps near and far |
| image width (inches) | 10 | number | §3.3; preset by `monitorwidth=` |
| use grayscale | no | yes/no | depth from palette brightness instead of colour index (§3.5) |
| calibration bars | middle | middle / top / none | §6 |
| use image map | no | yes/no | pattern image instead of random dots (§7) |

Keys: the cursor starts on the depth field. Typing replaces the value
(`t50`, or `kminus t100` for −100 in our xdotool notation), Down moves to the
next field, Right toggles yes/no and cycles the bar field
middle → top → none → middle. Enter starts the stereogram. Captures of the
screen and the cycle are reproducible with `tools/blackbox/stereogram/run.sh`
(`c` actions save the Xvfb frame buffer).

The fields kept their defaults in every fresh session; whether a changed
value persists within a session is **unknown**.

### 1.2 Command-line keys (confirmed accepted/rejected)

Probed with `tools/blackbox/stereogram/probe-keys.sh` (an unknown key makes
XFractint print that it could not understand the argument and save nothing):

- **Accepted:** `monitorwidth=N` (sets the width field: `monitorwidth=7`
  showed 7 on the screen; outputs `r-w3`, `r-w5`, `r-w13`, `l-big-w7.5`
  match the model with that width). `7.5`, `0` and `-3` are accepted;
  `abc` is rejected. The effect of 0 or negative widths is **unknown**.
- `stereo=1` and `stereo=4` are accepted, `stereo=5` rejected; they belong
  to red/blue stereo (other spec).
- **Rejected:** `rds`, `stereogram`, `autostereo`, `autostereogram`, `sirds`,
  `randomdot`, `randomdots`, `rdsdepth`, `depth`, `stereodepth`,
  `stereowidth`, `rdswidth`, `screenwidth`, `period`, `calibrate`,
  `calibration`, `gray`, `grayscale`, `rdsgray`, `rdscolor`, `imagemap`,
  `imagefile`, `depthmap`, `map3d`, `reduce`.

So depth effect, grayscale, bars and image map can only be set on the
screen. Batch mode (`batch=yes`) cannot make a stereogram (no key input).

## 2. Inputs and notation

- `W`, `H`: image width and height in pixels (the screen image; the
  stereogram has the same size).
- `c(x, y)`: the colour index (0–255) of the pixel on screen. Row 0 is the
  top.
- `D`: depth effect; `width`: image width in inches.
- `⌊ ⌋` is floor; `//` is integer division rounding down.

The depth maps are ordinary images: a calculated fractal, or a loaded GIF.
Loading a GIF and rendering the same picture give the same result (`r-*`
cases are rendered, `l-*` cases loaded; `r-d100` and `l-rows` use the same
depth rows and the same separations).

## 3. Depth → separation

### 3.1 Stretch to the image's colour range (confirmed)

```
cmin = min over the image of c,  cmax = max over the image of c
cn   = (c − cmin) · 255 // (cmax − cmin)      if cmax > cmin
cn   = 0                                       if the image has one colour
if D < 0:  cn = 256 − cn
```

Evidence: `l-r64` (colours 64–191) and an equivalent run with colours
10–109 (not kept) put the smallest colour at the nearest step and the
largest at the farthest, with step boundaries exactly where this integer
formula puts them (`(c−min)·256/(max−min+1)` and real-valued variants each
fail on at least one boundary). Every one-colour image (`l-flat128`; also
colours 0, 1 and 255, not kept) gives the same result. The range is taken
over the image's pixels, not the palette (`l-r64` uses a full 256-entry
palette). `cn = 256 − cn` for negative `D` is confirmed by `r-dm100` and
`l-rows-neg`, where colour 0 (`cn` = 256) and colour 1 (`cn` = 255) both land
on the farthest step; a one-colour image with negative `D` is untested
(**hypothesis**: `cn` = 256).

### 3.2 Depth steps (confirmed)

```
Lmax  = |D| · W // 4000          # number of steps above the nearest regular one
level = (cn · (Lmax + 1) − 1) // 256
```

`level` runs from 0 to `Lmax`; `cn = 0` (the image's smallest colour, or
the largest one when `D < 0`) gives **−1**, one step nearer than any other
colour. Evidence at `W` = 400: `D` = 0, 20, 37, 50, 100, 200, 500 give 1, 3,
4, 6, 11, 21, 51 steps (+ the −1 step) with boundaries at the predicted
colours (`r-d0`, `r-d20`, `r-d37` (not kept), `r-d50`, `r-d100`, `r-d200`,
`l-d500`); `D` = 10 (not kept) gives 2. The `−1` in the formula is visible
where `cn·(Lmax+1)` is a multiple of 256: with `D` = 37, colour 64 is still
step 0. At `W` = 800, `D` = 100 gives 21 steps (`l-big-w7.5`), so the count
scales with `W`. The exact grouping `|D|·W // 4000` (rather than
`(|D|//10)·(W//400)`) is a **hypothesis**: the two agree on every tested
pair.

### 3.3 Separation in pixels (confirmed, one constant fitted)

```
s = ⌊ (W / width) · A · (1 − (Lmax − level) · 8 / W) ⌋
```

which is the same as `⌊ A · (W − 8·(Lmax − level)) / width ⌋`. `A` is the
farthest separation in inches. Every row period in every case is
reproduced with any `A` in **[1.865625, 1.865853]** (1,450 flat rows from
15 runs with widths 3, 5, 7.5, 10, 13 and `W` = 400 and 800; fit in
`verify.py`'s model). `rdsmodel.py` uses `A = 125/67 = 1.8656716…` and
evaluates the expression in IEEE double precision in the order written
first above. The exact constant and the rounding at exact integers are
**unknown**; within the fitted interval no tested case is affected.

Consequences (confirmed): the farthest step has separation `⌊A·W/width⌋`
(74 px for 400 px at 10 inches); each step nearer is `8A/width` pixels
(≈ 1.49 px at 10 inches) closer; `D` = 100 spans 20 % of the farthest
separation at any `W`, `D` = 500 spans 100 %. Larger colours (after
stretching) are farther away when `D` > 0.

### 3.4 Separations of 0 and below (partly confirmed)

- `s = 0` (`l-d500`, colours 1–5): every pixel of the row is free
  (unconstrained). Confirmed; it follows from §4 with `s = 0`.
- `s = 1` (`l-d500`, colours 6–10): the whole row takes one value, the free
  value of column `W − 1`. Confirmed; follows from §4.
- `s < 0` (`l-d500`, colour 0, `s = −2`): the rows hold values that rise by
  one per pixel (42, 43, 44, …), unlike anything in §4/§5. Not modelled
  (**unknown**); `rdsmodel.py` skips such links and `verify.py` reports
  these 2 rows.

### 3.5 Grayscale option (partly confirmed)

With grayscale on, the depth value of a pixel is a brightness of its
palette colour instead of the index, then §3.1–3.3 apply to it.

- Confirmed: the order is that of `0.30·R + 0.59·G + 0.11·B` on the 8-bit
  GIF palette values. Over 480 rows (two palettes, `l-gray`,
  `l-gray-randpal`) no two colours are out of order, while equal weights,
  the green channel alone, the maximum, Rec. 709 weights and several
  others are.
- **Unknown:** the exact quantisation. Stretching the real-valued
  brightness with §3.1–3.2 reproduces 278 of 284 and 276 of 284 rows; the
  misses are colours within about 1 unit (on the 0–255 stretched scale) of
  a step boundary. The observed boundaries on the stretched scale are, for
  steps 1–10: (21.9, 22.8], (46.77, 46.82], (70.3, 70.5], (93.3, 93.8],
  (116.7, 117.6], (139.9, 140.1], (163.4, 164.4], (187.0, 187.5],
  (210.6, 211.7], (233.5, 234.5], compared with 23.4, 46.6, 69.9, 93.2,
  116.5, 139.7, 162.9, 186.2, 209.5, 232.7 for the index rule. Integer and
  rounded variants, 6-bit DAC values, other scales and Rec. 601 weights were
  tried and all miss 6 or more rows.

## 4. Linking pixels (confirmed)

Rows are independent. For row `y`, with `sep(c)` from §3:

```
same[x] = x for all x
for x = 0 … W−1:                          # left to right
    s     = sep(c(x, y))
    left  = x − (s + (y mod 2)) // 2      # even rows: x − s//2; odd rows: x − (s+1)//2
    right = left + s
    if left < 0 or right ≥ W: continue
    k = same[left]
    while k ≠ left and k ≠ right:         # merge with an existing link
        if k < right: left = k
        else:         left, right = right, k
        k = same[left]
    same[left] = right
for x = W−1 … 0:                          # right to left
    out[x] = free[x]          if same[x] = x
             out[same[x]]     otherwise
```

`free[x]` is the random value of §5 (or the image-map value of §7).
So the depth at the **midpoint** of each pair decides its separation, links
always point to the right, and a pixel takes the value of the end of its
chain; the rightmost pixels of each row are free. There is no other
hidden-surface step.

Evidence: `l-mix` (runs of random length and random colour, all 256
colours) is reproduced in every pixel. Before the odd-row rule was found,
`x − s//2` matched all even rows and no odd row; `x − (s+1)//2` then matched
all odd rows. Linking at `x` with the depth of `x` (one-sided), the depth of
the partner, or without the merge loop leaves 25,000–112,000 mismatched
pixels on `l-mix`. Every other case confirms it as well (§9).

## 5. Random values (confirmed)

- Generator: glibc `rand()` (the published additive-feedback generator; see
  [ifs.md](ifs.md) §4.1 for a description in our words).
- Call `n = y·W + x` (counting from 0 after seeding) gives the free value of
  pixel `(x, y)`: `free = rand() & 255`. One call per pixel, row-major,
  whether or not the pixel ends up free; no calls between rows.
- Seed: the Unix time in seconds, apparently taken when the stereogram is
  started (**hypothesis** on the exact moment). In `l-mix` the script
  recorded host time 1792327596 just before starting the container and pressed Enter about
  27 s later; the seed is 1792327624. Two runs started in the same second
  (`r-d100` and the not-kept duplicate) are identical; the seed stored at
  `fractint001`[98] is a different, older time and does not seed this
  generator.
- Every case's seed was found by searching the 5–7 minutes before the
  file's timestamp for a seed whose outputs at `n = W − 6 … W − 1` equal the last six
  pixels of row 0 (which are always free); the rest of the image then
  verifies. The seeds are in `cases.txt`.
- Whether it is `& 255`, `% 256` or `% (number of colours)` cannot be told
  apart with 256 colours (**unknown** for other colour counts).

An implementation that wants reproducible output should take the seed as a
parameter and default to the current time.

## 6. Calibration bars (confirmed at 400×300)

Bars are painted over the finished stereogram (the pixels under them follow
§4–§5 and are put back when the bars are removed, §8).

- **Rows:** `Ybar … Ybar + H//20` (16 rows at `H` = 300, 31 at 600), with
  `Ybar = H/2` for "middle" and `Ybar = 8` for "top". "none" draws nothing
  (`l-nobars` matches the model with no bars). Top at other heights is
  **untested**.
- **Columns:** two bars, `W//200 + 1` wide (3 at 400, 5 at 800), starting at
  `W/2 − h` and `W/2 + h`.
- **Spacing:** `h = m // 2`, where `m` is the integer part of the mean of
  `sep(c)` (§3) over the window
  `x = W/2 − W/16 … W/2 + W/16`, `y = Ybar − 8 … Ybar + 8` (inclusive; 51×17
  pixels at 400×300). So the bars are `2h` apart, an even number.
- **Colour:** the palette index with the largest R+G+B, the lowest such
  index on ties. Default palette → 15 (white); `uniq.map` → 192
  (ties with 193–195…); `l-tiepal` (entries 10, 20, 30 tie, 20 has the
  highest luminance) → 10; `l-randpal` → 3 (sum 179; a luminance rule would
  pick 87).

Evidence for the window: 40 runs at 400×300. 33 of them (flat, ramp,
two-level and box images, different `D`) left 241 candidate rectangles
among all with `x0` in 100…200, `x1` in 201…300, `y0` in 60…150, `y1` in
151…240 (mean rounded down or to nearest); seven images designed to split
those candidates (`l-des0`…`l-des6`) left exactly one: `x` 175…225,
`y` 142…158, rounded down. For "top", `l-top-yge9` and two more runs (not kept) fit `y` 0…16.
The bar size at 800×600 is confirmed (`l-big-w7.5`); the window there is
consistent but not pinned down (that image is flat along rows), so the
`W/16` and `±8` scaling is a **hypothesis**.

## 7. Image-map option (confirmed for one pattern)

Turning "use image map" on and pressing Enter opens a file selector listing
`*.gif` in the current directory. After a pattern is chosen, free values come
from it instead of `rand()`:

```
free[x] for row y = pattern[(y mod ph) · pw + (x mod pw)]
```

with `pw × ph` the pattern size, tiled from the top-left corner. Linking and
bars are unchanged. Evidence: `l-imagemap` (depth `rows`, pattern
`depth/pat.gif`, 50×40, same palette) is reproduced in every pixel.
**Unknown:** what happens when the pattern's palette differs, whether a
pattern larger than the image is cropped the same way, and whether `rand()`
is still called.

## 8. Output

- **Screen and file size:** unchanged. **Palette:** unchanged; the pixels
  are indices into it.
- **Saved blocks (confirmed):** the same blocks as the image had before
  (`fractint001`, plus `fractint003` for formula images), with the same
  parameters, so the file still describes the original fractal. For a
  loaded image the save has `fractint001`[202] = −1, as for any re-saved
  loaded image (gif-fractint-blocks.md §2.1); for a rendered image 4. In
  `r-d100` Ctrl+S was pressed before the render had fully finished; that
  save carries `fractint002` and status 2 (**hypothesis**: the calculation
  was interrupted and is still marked resumable). Nothing in the file marks
  it as a stereogram or stores its settings.
- **Bars in the file:** pressing `s` straight after the stereogram is drawn
  saves the bars. Pressing Enter first removes the bars and restores the
  stereogram pixels under them (`l-keypress` matches the model with no
  bars). Escape does not remove them; a second Escape opens the main menu.
- **Original image:** not restored on screen by Enter or Escape. Because the
  saved file keeps the original parameters, the original can presumably be
  recalculated from it (**hypothesis**, not tested). The dot pattern itself
  cannot be inverted.
- With the default palette (first probe, a Mandelbrot image, not kept) the
  saved file lacked indices 16, 31 and 248–255. This looks like the
  duplicate-palette effect of [types.md](types.md) §1.3 (**hypothesis**); use
  a palette without duplicates (`uniq.map`) when comparing pixels.
- Interrupting or resuming a stereogram while it is drawn: **untested**
  (generation of 400×300 finished within the 40–60 s we waited under
  emulation).

## 9. Verification

`python3 -I tools/blackbox/stereogram/verify.py` runs the model on every
line of `cases.txt` (seed, depth map, width, depth effect, bar mode) and
compares all pixels:

| Cases | Result |
|---|---|
| `r-d0`, `r-d20`, `r-d50`, `r-d100`, `r-d200`, `r-dm100`, `r-w3`, `r-w5`, `r-w13` (rendered depth, widths 3–13, `D` −100…200) | 0 mismatched pixels |
| `l-mix`, `l-rows`, `l-rows-neg`, `l-r64`, `l-flat128`, `l-box`, `l-lefthalf`, `l-tophalf`, `l-row150`, `l-xge180`, `l-xge220`, `l-yge130`, `l-yge170`, `l-des0`…`l-des6` | 0 |
| `l-top`, `l-top-yge9`, `l-nobars`, `l-keypress`, `l-tiepal`, `l-randpal`, `l-imagemap` | 0 |
| `l-big-w7.5` (800×600, width 7.5) | 0 |
| `l-d500` | 0 except rows 0 and 256 (`s < 0`, §3.4) |
| `l-gray`, `l-gray-randpal` | 6 and 8 of 300 rows differ (§3.5) |

Runs made during the analysis but not kept (to stay within the evidence
budget) agreed as well: `D` = 10 and 37, colour range 10–109, one-colour
images 0, 1 and 255, `x ≥` 100/150/250/280/320, `y ≥` 50/100/200/240, top
bars with `y ≥` 1 and 17, a duplicate of `r-d100`, and an 800×600 run at
10 inches.

## 10. Unknowns

- Exact constant `A` and the evaluation order of §3.3 (only an interval).
- Grayscale quantisation (§3.5).
- `s < 0` behaviour (§3.4); widths ≤ 0; very large `D`.
- Calibration window and "top" row at sizes other than 400×300 (§6).
- How the random value is reduced when fewer than 256 colours are
  available (§5); the exact moment the time seed is read.
- Image map with a different palette or size; whether it consumes `rand()`.
- Whether settings persist between Ctrl+S uses in one session, and whether
  PAR files or other entry points can carry them.

## 11. Evidence and tools

- `corpus/xfractint/stereogram/` — `cases.txt`, `depth/`, `out/`,
  `README.md`, `SHA256SUMS`.
- `tools/blackbox/stereogram/`:
  - `run-keys-fb.sh` (in container) / `run.sh` (host): key and text input
    plus frame-buffer captures (`xwd2png.py` converts them).
  - `rds-run.sh` (render, Ctrl+S, save) and `rds-load.sh` (load a depth GIF,
    Ctrl+S, save).
  - `probe-keys.sh`: command-line key acceptance.
  - `craft-depth.py`: writes the crafted depth maps; `inputs/rds.frm`: the
    depth formulas.
  - `rdsmodel.py`: the model of §3–§7; `verify.py`: §9; `gifio.py`,
    `ana.py`: GIF reading and quick period analysis.
