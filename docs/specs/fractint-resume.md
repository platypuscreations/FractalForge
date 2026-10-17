<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Fractint-format resume data (`fractint002`)

Status: **Phase 2 draft, derived from black-box analysis only.** Every rule
below comes from pixels and bytes of files that XFractint saved, or from what
XFractint did with files we made. No Fractint, XFractint or Iterated Dynamics
source code, headers, help text or documentation was consulted (see
[CLEANROOM.md](../CLEANROOM.md)). Names are our own.

This spec refines §4 and §5.4 of
[gif-fractint-blocks.md](gif-fractint-blocks.md) (block framing, `fractint001`
fields, pixel ↔ coordinate mapping and colours are described there) and uses
the symmetry rule of [types.md](types.md) §1.6. Where this spec and §4 of
gif-fractint-blocks.md disagree, this spec has the newer evidence (§9 lists
the corrections).

Labels: **Confirmed** = shown directly by many files or by a load test.
**Hypothesis** = fits all evidence but not proven. **Unknown** = not
determined.

Evidence lives in `corpus/xfractint/resume/` (file names below are relative
to it; command lines are in its `cases.txt`). The scripts that made and
checked it are in `tools/blackbox/resume/`. XFractint is Debian
`xfractint 20.4.10-5` (amd64) in `fractalforge-blackbox`.

---

## 0. Summary

- `fractint002` is a small **work list**: a header of two i32 values (format
  2, number of pending items) and room for 12 items of 8 i32 values each. An
  item is a rectangle of the image still to be worked on, with its symmetry,
  the pass it is in and the next pixel (row, column) to do (§1).
- Every pixel outside all pending items is final. Inside an item the pass
  mode decides which pixels are final, provisional (coarse blocks or
  guesses) or empty. §4 gives an exact rule; it reproduced every pixel of 73
  of 74 interrupted saves, and all but one pixel of the last (1-pass, 2-pass
  and guessing; centred, off-centre and absent axis on mandel, julia,
  barnsleym1; 200×150 to 800×600).
- Symmetric items compute only their upper half and copy each pixel to its
  mirror at once (§2).
- **XFractint resumes only when the file is loaded with its in-program load
  command (`r`)** (the only way we found). Loading with `filename=` on the
  command line and pressing Enter recalculates the whole image from scratch
  (§5.1). With `r`, XFractint continues from the work list and finishes with
  pixels identical to an uninterrupted render, for every mode tried (§5.3).
- On resume XFractint never recomputes pixels the work list says are done,
  including the origins of provisional blocks, and it trusts the item's
  symmetry code (§2.4, §5.3).
- A third-party writer can hand XFractint a partial image: for passes=1 any
  (row, column); for 2-pass and guessing at pass and band boundaries, if the
  coarse pixels are supplied as XFractint would have made them (§6, all
  load-tested with marker pixels). §7 lists what happens with bad data.

---

## 1. Block layout (confirmed)

`fractint002` has a 392-byte payload (framing in gif-fractint-blocks §1.2).
Read it as 98 little-endian i32 values. No value seen needed more than 16
bits; write i32.

| Byte offset | Meaning (our name) | Observed |
|---|---|---|
| 0 | format | always 2 |
| 4 | **item count** `n` (0–12) | 1 in almost every save; 2 in `i-MO-2-12` |
| 8 + 32·k | item k (k = 0…11): 8 values, below | |

Item fields (byte offset inside the item; slot 0 is at 8–39):

| Item offset | Block offset (slot 0) | Meaning | Notes |
|---|---|---|---|
| +0 | 8 | `left`: first column | 0 everywhere |
| +4 | 12 | `right`: last column | w − 1 everywhere |
| +8 | 16 | `top`: first row of the item | moves down as rows become final (§3) |
| +12 | 20 | `bottom`: last row of the item | moves up with symmetry (§2) |
| +16 | 24 | `row`: current row (1, 2-pass), or top row of the current band (guessing) | |
| +20 | 28 | `sym`: symmetry code | 16 none, 17 x-axis, 51 origin (§2) |
| +24 | 32 | `pass`: pass index inside the mode, from 0 | §3 |
| +28 | 36 | `col`: next column in `row` (1, 2-pass); 0 for guessing | |

- Slots at index ≥ `n` are **left over from earlier** and must be ignored:
  `i-MO-2-20` has `n` = 1 but slot 1 still holds `0, 399, 100, 299, 100, 17,
  1, 0` from the earlier two-item state (compare `i-MO-2-12`). A writer should
  zero unused slots.
- When the image is finished there is no `fractint002` block at all
  (gif-fractint-blocks §7).

Evidence: all `i-*.gif`; multi-item: `i-MO-2-12` (`n` = 2), `i-MO-2-20`,
`i-MO-2-30` (stale slot 1).

## 2. Symmetry

### 2.1 Codes (values confirmed, meaning partly hypothesis)

| `sym` | Seen with | Mirroring XFractint did |
|---|---|---|
| 0 | `symmetry=none` on the default mandel view (`i-MSN-1-12`); `outside=real` (`i-ORL-1-12`); also written back for `verify/crafted-r1` (`outside=real`, gif-fractint-blocks §8) | none |
| 16 | no axis in view (`MN`, `BOF` with `inside=bof60`) | none |
| 17 | mandel default view (`MS`, `MSG`), off-centre view (`MO`) | x-axis: rows |
| 51 | julia `-0.123/0.745` (`J`, `JG`), julia `-1/0` (`JB`) | origin: rows and columns together |
| 51 | barnsleym1 default view (`BX`) | **xy**: rows and columns independently |

- **Confirmed:** 0 and 16 both mean "no mirroring" for reading and for
  loading (`loaded/q-Y13`, §7).
- **Hypothesis:** the 0x10 bit means "symmetry was allowed" (it is clear
  when the user or the colouring turned symmetry off) and the low bits say
  which directions are mirrored (0x01 rows, 0x02 columns, repeated in the
  high nibble for 51 = 0x33).
- **Confirmed:** the code does **not** distinguish origin from xy symmetry:
  both julia and barnsleym1 store 51. A reader has to take the kind from the
  fractal type (types.md §1.6 and its per-type table) when it sees 51:
  `i-J-*`, `i-JB-*` fit origin mirroring exactly, `i-BX-*` fit xy mirroring
  exactly (`model-check.txt`).

### 2.2 Mirror rule inside an item (confirmed)

For an item with rows `top…bottom` and columns `left…right`:

- `last = top + (bottom − top) div 2` is the last row computed directly.
  Rows `top…last` are computed; rows `last+1…bottom` are copies.
- x-axis (17): pixel (r, c) with r > last is a copy of
  (`top + bottom − r`, c).
- origin (51, julia): pixel (r, c) with r > last is a copy of
  (`top + bottom − r`, `left + right − c`).
- xy (51, barnsleym1): additionally, inside each computed row only columns
  `left … left + (right − left) div 2` are computed and column c >
  that is a copy of `left + right − c`; then rows mirror as for x-axis
  (`i-BX-1-12`: row 43 has columns 0–174 and 225–399, `col` = 175).
- The copy is made **as each pixel is computed**: in an interrupted save the
  partly done row and its mirror row hold the same pixels (`i-MS-1-12`:
  rows 102 and 197 both have columns 0–254; `i-J-1-12`: row 85 has columns
  0–107 and row 214 columns 292–399).
- `top + bottom` stays constant while an item shrinks (§3.1), and equals
  `S` from types.md §1.6 (the row sum of a mirrored pair: 299 for the
  default 400×300 view, 399 for the off-centre view below).

### 2.3 Off-centre axis: the image is split into items (confirmed)

`MO` = `type=mandel corners=-2.5/1.5/-1.0/2.0` (400×300): the real axis is
at row 199.33, so rows 200–299 mirror rows 199–100 and rows 0–99 have no
partner (types.md §1.6, `S = 399`). XFractint keeps two items:

| File | Items (left, right, top, bottom, row, sym, pass, col) |
|---|---|
| `i-MO-2-12` (2-pass, first pass) | `0,399,100,299,192,17,0,174` and `0,399,0,99,0,16,1,0` |
| `i-MO-2-20` (second pass) | `0,399,147,252,147,17,1,223` (count 1) |
| `i-MO-1-12` (1-pass) | `0,399,153,246,153,17,0,249` (count 1) |

So the rows without a partner form their own item with sym 16, the mirrored
band `100…299` is a sym-17 item with `top + bottom = 399`, and XFractint
finishes each pass on the unmirrored item (rows 0–99) **before** it starts
that pass on the mirrored item: in `i-MO-2-12` the 0–99 item already waits
for pass 1 while the other is still in pass 0. A finished item is removed and
the count drops (`i-MO-2-20`, `i-MO-1-*`).

### 2.4 XFractint trusts the stored item when it resumes (confirmed)

On load (§5.2) XFractint mirrors exactly as the item says: with code 17 or
51 it mirrors about `top + bottom`, whatever the parameters are.

- `loaded/q-Y14`: non-symmetric `MN` view written with code 17 and rows
  120…299: rows 210–299 came out as copies of rows 209–120 (7,277 wrong
  pixels).
- `loaded/q-C4m`, `q-C6m`, `q-C7m`: symmetric views written with the item
  covering all remaining rows to the bottom (`bottom` = 299) and code 17/51:
  mirroring about the wrong sum, wrong pixels below the item's middle.
- The same files with code 16 (`q-C7n`), or with the item in XFractint's own
  shrunken form (`q-C5m`: rows 100…199, `top + bottom` = 299), finish
  correctly.

### 2.5 Other symmetries

**Unknown:** codes for y-axis, pi-periodic and the other symmetry kinds of
types.md were not obtained. Only views with the axis centred, off-centre
below the middle (`MO`) and absent were run; rotated views were not.

---

## 3. Visiting order per pass mode (confirmed)

`w`, `h` = image size. Blocks always sit on the **image-wide grid**: a k×k
block has its top-left pixel ("origin") at a row and column that are
multiples of k. The origin is computed; the rest of the block is filled with
the origin's colour. (Coarse passes were only seen on items whose top is 0 or
100; later passes on items with odd tops, e.g. `i-MO-2-20`, still use the
image-wide grid.)

### 3.1 passes=1

- One pass (`pass` = 0). Rows top to bottom, each row left to right, inside
  the computed half of the item.
- `row` = current row R, `col` = next column C. Pixels of row R with column
  < C are done; column C and later are not. `top` = R as well, so the item
  covers only unfinished rows.
- With symmetry `bottom` = `top + bottom` (constant) − R, so the item shrinks
  from both ends: `i-MS-1-12`: 102 … 197; `i-MS-1-60`: 145 … 154.
- Without symmetry `bottom` = h − 1.

Evidence: 26 saves `i-*-1-*` (views MS, MN, J, MO, JB, BX, BOF, ORL, MSN;
§4 reproduces them exactly).

### 3.2 passes=2

- **Pass 0** (`pass` = 0): a coarse image of 2×2 blocks. Block rows go top to
  bottom (`row` = the even row of the block row), blocks left to right
  (`col` = the even column of the next block). `top`, `bottom` keep the
  item's full range during this pass (`i-MS-2-12`: `top` 0, `bottom` 299,
  `row` 140). With symmetry the pass covers the computed half and each block
  is copied to its mirror (block rows 148–149 → rows 151–150).
- **Pass 1** (`pass` = 1): full resolution, row by row like passes=1,
  starting again at the item top. Pixels not reached yet still show the
  pass-0 block colour. `top` = `row` = R, `bottom` as in §3.1.
- Inside the partly done row R, columns < C are final and the rest are still
  block colour; row R + 1 is still entirely block colour even when R is the
  even row of a block row (`i-J-2-30`, R = 104; `i-MS-2-60`, R = 142), so
  rows are not refined in pairs.

Evidence: 21 saves `i-*-2-*` (views MS, MN, J, MO, JB).

### 3.3 passes=g (solid guessing)

The image is refined through `N` passes. `N` depends on the image size
(table). In pass p (`pass` = p, 0 ≤ p < N) the block size produced is
`b = 2^(N−1−p)` and the pass works through **bands** of `2b` rows; `row` is
the top row of the current band, a multiple of `2b`, and `col` is 0.
`top` = 0 and `bottom` = h − 1 throughout (or the item's range).

| Image | N | pass 0: band, blocks | pass 1 | pass 2 | pass 3 | Evidence |
|---|---|---|---|---|---|---|
| 200×150 | 2 | 4 rows, 2×2 | 2 rows, 1×1 (final) | | | `s4-MSG-g-*` |
| 267×200 | 2 | 4 rows, 2×2 | (not caught) | | | `s3-MSG-g-*` |
| 400×300 | 3 | 8 rows, 4×4 | 4 rows, 2×2 | 2 rows, 1×1 | | 17 saves `i-{MSG,MNG,JG}-g-*` |
| 800×600 | 4 | 16 rows, 8×8 | 8 rows, 4×4 | 4 rows, 2×2 | 2 rows, 1×1 | `int-passg*` (gif-fractint-blocks corpus) |

**Hypothesis** for other sizes: `N` = ⌊log2(w / 50)⌋ (equivalently h / 37.5
for these 4:3 sizes). It fits all four sizes; the boundaries between sizes
were not probed and the file does not store `N`. A reader that meets a size
it can't place should treat all pixels inside the item as provisional.

State of an interrupted guessing save, pass p, band top R, item rows
`top…bottom` (computed half if symmetric):

- Rows < R: every pixel has the colour of its b×b block origin. With
  b = 1 (last pass) these are final.
- Rows R … R + 2b − 1 (the band): **in progress.** Origins of the 2b grid
  on row R are exact. Other pixels can be exact, block colour from the
  previous pass, half-refined values, or 0 (pass 0).
- Rows ≥ R + 2b: pass p > 0: 2b×2b blocks from the previous pass; pass 0:
  empty (0), except that grid points (column multiple of 2b) on row R + 2b
  may already be computed (exact). Seen as 50 exact points on row 96 while
  band 88 was in progress (`i-JG-g-10`).
- Every block origin that is present holds the value the finished guessing
  render has at that pixel. Provisional block colours are therefore the
  finished colour of the block's origin, which is what a reader needs.

"Final" for guessing means equal to XFractint's finished **guessing** render,
which itself differs from passes=1 in a few pixels (13 of 120,000 for `MN`,
`ref-MN-g` vs `ref-MN-1`).

Evidence: 27 saves: 17 `i-*G-g-*`, 8 `s?-MSG-g-*`, and `int-passg`,
`int-passg-early` from the Phase 1 corpus.

---

## 4. Reader rule (confirmed: reproduces every pixel)

`tools/blackbox/resume/model.py` implements this. Given an interrupted save
and a finished batch render of the same parameters and pass mode, it predicts
every pixel of the save. Over 74 interrupted saves (72 in `int/` plus
`interrupted/int-passg` and `int-passg-early` from the Phase 1 corpus) it
reproduced every pixel outside the guessing band, except one: in
`i-BOF-1-12` the pixel at (`row`, `col`) itself already holds its final value
(the save probably came between writing the pixel and advancing `col`).
Treat (`row`, `col`) as to-do. The 13 runs that finished before the key
press equal their batch render.

### 4.1 Classification

    state(r, c) = FINAL                    # default
    for each item k < n:
        if (r, c) outside the item rectangle: continue
        (r', c') = (r, c), or its mirror (§2.2) when r > last
        state(r, c) = inner_state(mode, item, r', c')
        # mirrored pixels have the same state and colour as their partner

inner_state for (r, c) in the computed half (R = `row`, C = `col`,
p = `pass`):

| Mode | Condition | State | Colour in the file |
|---|---|---|---|
| 1 | r < R, or r = R and c < C | final | exact |
| 1 | otherwise | empty | 0 |
| 2, p = 0 | block row r−r%2 < R, or = R and c−c%2 < C | provisional | origin of its 2×2 block |
| 2, p = 0 | otherwise | empty | 0 |
| 2, p = 1 | r < R, or r = R and c < C | final | exact |
| 2, p = 1 | otherwise | provisional | origin of its 2×2 block |
| g | r < R | final if b = 1, else provisional | origin of its b×b block |
| g | R ≤ r < R + 2b | in progress | see §3.3 |
| g | r ≥ R + 2b, p > 0 | provisional | origin of its 2b×2b block |
| g | r ≥ R + 2b, p = 0 | empty (grid points on row R + 2b may be exact) | 0 |

"Origin of its k×k block" means: the pixel holds the colour that the block's
top-left pixel (multiples of k, §3) has in the finished render. Origins of
provisional blocks are themselves final (their own pixel is exact), and
XFractint never recomputes them (§5.3). A reader that only wants certainly-correct pixels can keep
`final` pixels and block origins, and treat everything else as to-do.

### 4.2 Empty is 0

Pixels not yet reached are colour 0 in the saved image. Colour 0 can also be
a real result (default inside colour, or an escape count that is a multiple
of 256), so "colour 0" alone does not mean "empty". Use the work list.

### 4.3 Unsupported pass modes

Modes `3`, `b`, `t`, `d` and `o` were not analysed here. `t` and `d` store
counters that aren't rows (gif-fractint-blocks §4). A reader should treat all
pixels inside pending items as provisional for these modes.

### 4.4 Evidence for §4

`model-check.txt` in the corpus folder has one line per save: mode, items,
counts per state and mismatches (all empty except the one pixel of
`i-BOF-1-12`).

---

## 5. Resuming in XFractint

### 5.1 `filename=` on the command line + Enter recalculates (confirmed)

This is how `run-load` and gif-fractint-blocks §8 loaded files. It does
**not** use the resume data:

- `loaded/k1-ret`: own 1-pass save `i-MN-1-30` (`row` 155) with row 10
  repainted colour 7 (`crafted/K1`). Enter after 10 s, `s` after 40 s more.
  Row 10 was recomputed (marker gone) and the save is at row 204, about where
  a fresh render is after the same time (`i-MN-1-45`: row 221 at 45 s).
- `loaded/k2-ret`: 800×600 own save `int-pass1-late` (`row` 93) with a marker
  row (`crafted/K2`). 20 s after Enter the save is at row **84** and the
  marker is gone.
- `loaded/k1-none`: no key at all: nothing is calculated; the save has status
  −1 and no `fractint002`.
- Space, Escape, `c`, Tab and a second Enter did not resume either (the space
  key switches to another image, the others do nothing; runs not kept).

So every earlier "resume" check that used `filename=` + Enter (Phase 1 R1–R4,
`interactive-resume-int-pass1-early`) only showed that the parameters load
correctly: XFractint computed those images from the beginning.

### 5.2 The in-program load command `r` resumes (confirmed)

Start XFractint with any quick image (we use `type=mandel maxiter=10
viewwindows=2`), press `r`, type the file name, press Enter. XFractint loads
the image and **continues the calculation from the work list at once**; no
further key is needed (`loaded/k1-rload`: marker row 10 kept, rows 155 → 296
computed in 30 s). An Enter after that does no harm (`loaded/k1-rload-ret`
finished with only the marker differing). `tools/blackbox/resume/run-keys.sh`
drives this with `s10 kr s3 tin.gif s1 kReturn sD ks`.

### 5.3 Own saves resume to the uninterrupted image (confirmed)

21 own interrupted saves, unmodified, loaded with `r` and left to finish
(`resumed/o-*`): every result is complete (status 4, no `fractint002`) and
**pixel-identical** to the batch render of the same parameters:

| Mode | Saves (`int/` name → `resumed/o-…`) |
|---|---|
| passes=1 | `MS-1-30`, `MN-1-30`, `J-1-30`, `MO-1-20` |
| passes=2, pass 0 | `MS-2-12`, `MN-2-12`, `J-2-12`, `JB-2-12`, `MO-2-12` (two items) |
| passes=2, pass 1 | `MS-2-45`, `MN-2-45`, `J-2-45` |
| guessing pass 0 | `MSG-g-10`, `MNG-g-10`, `JG-g-10` |
| guessing pass 1 | `MSG-g-30`, `MNG-g-15`, `JG-g-45` |
| guessing pass 2 | `MSG-g-60`, `MNG-g-20`, `JG-g-90` |

To show that these were real resumes, the same kind of saves were loaded
with marker pixels (colour 7) painted over pixels that are already done
(`crafted/K-*` → `loaded/q-K-*`). In every case all markers survived and
every other pixel matches the batch render:

| File | Marker on | Survived | Shows |
|---|---|---|---|
| `K-MS-1-30` | row 10 (final) | yes | 1-pass resumes at (`row`, `col`) |
| `K-MN-2-45` | row 10 (final); even columns of row 200 (2×2 origins, provisional area) | yes, both | pass 1 never recomputes 2×2 origins |
| `K-J-2-45` | row 10; even columns of row 140 | yes | same with origin symmetry |
| `K-MN-2-12` | even columns of row 100 (origins made in pass 0) | yes; the odd pixels of rows 100–101 were refined | pass 0 → pass 1 keeps origins |
| `K-MSG-g-60` | row 10 (final); even columns of row 140 | yes | last guessing pass keeps 2×2 origins |
| `K-MSG-g-30` | columns ≡ 0 mod 4 of row 140 (4×4 origins) | yes | guessing pass 1 keeps 4×4 origins |
| `K-MNG-g-10` | columns ≡ 0 mod 4 of row 100 | yes | guessing pass 0 → 1 keeps origins |

Rule (confirmed): **XFractint never recomputes a pixel that an earlier pass
or an earlier stretch of the current pass has computed.** It trusts the
image data for block origins and for everything before (`row`, `col`).

---

## 6. Writer recipe

All recipes were load-tested with `r` (§5.2) from `crafted/` files whose
pixels came from a finished batch render (`ref/`), blanked or reduced to
blocks as stated; results are `loaded/q-<name>`. Where a marker row is
mentioned, it survived, so the file was resumed and not recalculated.

### 6.1 Common part

1. Write the GIF and `fractint001` as in gif-fractint-blocks §5.1–5.2, with
   the pass-mode letter at 001[200], status 2 at 001[202] and 930 at
   001[204] (521 for `fractint001` + 409 for `fractint002`).
2. Put `fractint002` before `fractint001`. Payload: 98 i32 values, all 0
   except `[0]` = 2, `[1]` = item count `n`, and the items in slots 0…n−1
   (§1).
3. Pixels outside every item are taken as final and never touched
   (`q-C5m`: rows 240–299 left as given; `q-Y20`: columns outside the item
   left as given).
4. Zero unused slots. Use `sym` = 16 unless you reproduce XFractint's
   symmetric item exactly (§6.5).

### 6.2 passes=1, any row and column (confirmed)

- Item: `left` 0, `right` w−1, `top` R, `bottom` h−1, `row` R, `sym` 16,
  `pass` 0, `col` C.
- Pixels before (R, C) in row order: final colours. Pixels from (R, C) on:
  anything; they are recomputed (`q-Y10` had them all set to 7).
- Tested: (120, 200) `q-C1m`, (120, 0) `q-C2m`, (0, 137) `q-C3m`, julia with
  origin symmetry written as sym 16 (60, 300) `q-C7n`. Each finished
  identical to the batch render apart from the marker row.
- Several items work: rows 120…199 from column 200 plus rows 200…299 as a
  second item (`q-C18m`).

### 6.3 passes=2 (confirmed at and between pass boundaries)

The non-origin pixels of each 2×2 block are recomputed in pass 1, but the
origins (even row, even column) never are, so **every origin that pass 0 has
reached must hold its final colour**.

| State to write | Item (`top`, `row`, `pass`, `col`) | Pixels | Test |
|---|---|---|---|
| pass 0 finished | 0, 0, 1, 0 | every pixel = colour of its 2×2 origin | `q-C8m` ✓ |
| inside pass 1 at (R, C) | R, R, 1, C | before (R, C) final; after: 2×2 block colours with exact origins | `q-C9m` ✓ |
| inside pass 0 at block row R (even), column 0 | 0, R, 0, 0 | rows < R: 2×2 blocks with exact origins; rest anything | `q-C11m` ✓ |
| inside pass 1, rest **zero** | R, R, 1, C | after (R, C) all 0 | `q-C10m` ✗: every even row after R keeps 0 at even columns |

A writer that only has full-resolution pixels should write passes=1 (§6.2)
or, keeping letter `2`, put the item in pass 0 at (R, C) (`q-Y15`: letter
`2`, `pass` 0, 1-pass pixels before (120, 200) → correct image; XFractint
does pass 0 and pass 1 over the item only). Writing `2` with `pass` 1 and
zeros after (R, C) fails like `q-C10m` (`q-C17m`).

Mid-row pass 0 states (C ≠ 0) were not tested.

### 6.4 Guessing (confirmed at pass boundaries and band boundaries)

Use `N` and block sizes from §3.3. Entering pass p the image must be the
previous pass's coarse image: every pixel has the colour of its block
origin, and origins are exact. This matters for all pixels, not only
origins: XFractint does not rewrite pixels it decides to guess, so a guessed
area keeps whatever colour the file had (`q-C14m`: 74 of 100 marker pixels
on positions refined in pass 1 survived; the other 26 were computed).

| State to write (400×300, N = 3) | Item (`row`, `pass`) | Pixels | Test |
|---|---|---|---|
| pass 0 finished | 0, 1 | 4×4 blocks from exact origins | `q-C12m` ✓ (markers on origins kept) |
| inside last pass at band R = 100 | 100, 2 | rows < 100 final; rest 2×2 blocks | `q-C13m` ✓ |
| same, rest zero | 100, 2 | rows ≥ 100 all 0 | `q-C13z` ✗ |
| inside pass 0 at band R = 96 | 96, 0 | rows < 96 4×4 blocks; rest 0 | `q-C14m`: correct except the 50 grid points (every 8th column) of row 96, left at 0 |
| symmetric view, pass 1 finished, `sym` 17 | 0, 2 | 2×2 blocks taken from the finished image's own grid in both halves | `q-C15m` ✗: 1,162 wrong pixels in rows 151–281 |

So, for guessing:

- Write `top` 0, `bottom` h−1, `col` 0, `row` = a band top (multiple of
  2b), `pass` = p.
- In pass 0 also supply the grid points (columns multiple of 2b) of row R.
- With symmetry, the mirrored half must be the exact mirror of the computed
  half (including provisional blocks), as XFractint makes it (§2.2); simpler:
  write `sym` 16 and blocks for the whole image (not load-tested for
  guessing).
- Mid-band states were not tested.

### 6.5 Symmetric views

- Safe: `sym` 16 with an item covering everything still to do. XFractint then
  computes all of it directly (`q-C7n`). The result equals the symmetric
  render whenever the view is mirror-exact (types.md §1.6); for an off-centre
  view mirroring changes some pixels, so compare with the parameters' own
  render.
- Exact XFractint form: `sym` 17/51, `top` = R, `bottom` = S − R, mirrored
  rows already copied (`q-C5m`; S from types.md §1.6). Off-centre views also
  need the extra unmirrored item (§2.3) (not load-tested).
- Wrong: `sym` 17/51 with `bottom` = h−1 (`q-C4m`, `q-C6m`, `q-C7m`, §2.4).

---

## 7. Inconsistent and unsupported resume data

All loaded with `r` (§5.2). Base file: `MN` passes=1, item at (120, 200),
marker row 10. "Resumed" means the marker survived.

| File | Change | What XFractint did |
|---|---|---|
| `q-Y1`, `q-Y23` | format `[0]` = 1, or 0 | resumed normally: `[0]` is not checked |
| `q-Y2` | item count 0 | treated as finished: saved as complete (status 4), rows 120–299 stay empty |
| `q-Y22` | item count 13 (slots 1–12 zero) | resumed normally, correct image |
| `q-Y3` | `row` 400 (below the image) | treated as finished (status 4); nothing computed |
| `q-Y4` | `col` 999 | row 120 counted as done; continued at row 121 |
| `q-Y12` | `row` −5 | computed from row 0: whole image recomputed (marker replaced) |
| `q-Y21` | `top` 50, `row` 120 | started at (120, 200); rows 50–119 untouched (second marker row 80 survived) |
| `q-Y20` | `right` 199 | only columns 0–199 computed; 200–399 of rows 120–299 stay empty; saved as complete |
| `q-Y13` | `sym` 0 | resumed normally, no mirroring |
| `q-Y14` | `sym` 17 on a non-symmetric view | mirrored anyway (§2.4): 7,277 wrong pixels |
| `q-Y15` | letter `2`, `pass` 0 | resumed: pass 0 then pass 1 over the item; correct |
| `q-Y16` | letter `t` (tesseral) with this item | resumed with tesseral over the item: correct except 1 pixel |
| `q-Y18` | letter `2`, `pass` 2 (no such pass) | resumed; correct image |
| `q-Y7` | guessing, `pass` 7 (no such pass) | treated as finished (status 4); provisional blocks left as they were |
| `q-Y5` | status 2 but no `fractint002` | computed the whole image from scratch |
| `q-Y6` | status 4 with a `fractint002` | not resumed: shown as finished, empty rows stay; saved as status 4 without 002 |
| `q-Y19` | status 1 | not resumed; saved with status 0, empty rows stay |

So XFractint resumes only when 001[202] = 2 **and** `fractint002` is
present, does not validate the items, clamps or skips out-of-range values in
the ways above, and never rejects a file. A writer must get the items right;
a reader should clamp items to the image and treat an empty or unusable work
list as "status unknown" rather than "finished".

---

## 8. Unknowns

- How `N` (guessing pass count) is chosen for sizes other than 200×150,
  267×200, 400×300 and 800×600 (§3.3 hypothesis).
- Symmetry codes and mirror rules for symmetry kinds other than none, x-axis,
  origin and xy; how the 0x10 bit is decided (§2.1).
- Exact meaning of the low bits of `sym` and how XFractint tells origin from
  xy on load (presumably from the type).
- Pass modes `3`, `b`, `t`, `d`, `o` (only `t` was loaded, §7).
- 2-pass states in the middle of a pass-0 block row; guessing states in the
  middle of a band; guessing with off-centre symmetry (more than one item).
- Whether item `left`/`right` ever differ from 0 and w−1 in XFractint's own
  saves (never seen), and how items are split for other off-centre layouts
  (axis above the middle, both axes off-centre).
- Rotated or skewed views.

## 9. Corrections to gif-fractint-blocks.md

- §4 table: offset 4 is the **item count** (not a constant 1); offsets 8–39
  are item 0 (left, right, top, bottom, row, sym, pass, col); offsets 40–391
  are slots 1–11, not padding. Offset 20 is the item's bottom row, which is
  smaller than h−1 for symmetric views. Offset 28 is the symmetry code
  (§2.1): 16 was constant in the Phase 1 corpus because those views have no
  axis.
- §4 and §8, "[28] became 0 after a resumed calculation": the 0 comes from
  `outside=real` in `crafted-r1` (code 0 = symmetry off, §2.1);
  `interactive-resume-int-pass1-early` still holds 16.
- §4 "A 1-pass render picks up there when resumed", the Observations bullet
  "Interactive mode resumes an interrupted image when Enter is pressed after
  loading", §5.4 and the R1–R4 / "interactive resume" rows of §8: loading with
  `filename=` and pressing Enter **recalculates from scratch** (§5.1). Those
  tests confirm the parameter fields, not resume semantics. Resuming needs
  the `r` command (§5.2); §6 has load-tested recipes.
- §4 "[16] is where the final pass begins": [16] is the item's top row; it
  equals `row` in the last pass of 1-pass/2-pass and stays at the item's top
  during coarse passes and guessing.
- §4 guessing "[32] = 2 then 3, direction not proven": [32] is the pass index
  0…N−1 (§3.3); N = 4 at 800×600.

## 10. Evidence index

All in `corpus/xfractint/resume/` (command lines in `cases.txt`; checks in
`model-check.txt`). Views (all `viewwindows=2 periodicity=0 inside=100`
unless noted; `inside=100` makes "empty" (0) distinct from inside):

| Tag | Parameters | Symmetry |
|---|---|---|
| `MS` | `type=mandel maxiter=100000` | x-axis, centred |
| `MN` | `type=mandel corners=-0.75/-0.70/0.10/0.14 maxiter=8000` | none |
| `J` | `type=julia params=-0.123/0.745 maxiter=100000` | origin |
| `MSG`, `MNG`, `JG` | as `MS`, `MN`, `J` with maxiter ×10 (guessing is fast) | |
| `MO` | `type=mandel corners=-2.5/1.5/-1.0/2.0 maxiter=100000` | x-axis, off-centre |
| `JB` | `type=julia params=-1/0 maxiter=100000` | origin |
| `BX` | `type=barnsleym1 maxiter=100000` | xy |
| `BOF`, `ORL` | `MN` corners, maxiter 8000, `inside=bof60` / `outside=real` (no `inside=100`) | none |
| `MSN` | `MS` with `symmetry=none` | off |
| `s3-`, `s4-` | `MSG` at `viewwindows=3` (267×200) / `4` (200×150) | x-axis |
| `C800` | `type=mandel maxiter=20000 corners=-0.75/-0.74/0.10/0.11 passes=g`, 800×600 | none |

| Folder | Content |
|---|---|
| `ref/` | finished batch renders `ref-<view>-<pass>` |
| `int/` | interrupted interactive saves `i-<view>-<pass>-<seconds>`, `s3-…`, `s4-…` |
| `resumed/` | own saves resumed with `r` (`o-<int name>`, §5.3) |
| `crafted/` | files we wrote (`C*`, `Y*`: from `ref/` with `craft.py`; `K*`: XFractint's own saves with marker pixels) |
| `loaded/` | what XFractint saved after loading a crafted file: `q-<name>` (with `r`), `k1-*`, `k2-*` (the key experiments of §5.1–5.2) |

Tools (`tools/blackbox/resume/`): `gif.py` (GIF reader/writer),
`model.py` (§4 rule), `craft.py` + `craft-r.sh` (crafted files),
`marker.py` / `evalq.sh` (marker reports), `run-keys.sh` + `keys.sh`
(scripted key presses, including the `r` load), `queue.sh` + `jobs-*.txt`
(the runs). The first exploratory runs (`jobs-craft.txt`, `jobs-extra.txt`,
`jobs-x.txt`, `craft-all.sh`, `craft-extra.sh`, `craft-x.sh`) used
`filename=` + Enter and therefore only showed the recalculation of §5.1;
their outputs were not kept.
