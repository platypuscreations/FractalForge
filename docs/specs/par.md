<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Fractint-format `.par` parameter files

Status: **Phase 2 draft, derived from black-box analysis only.** Every rule
below comes from `.par` files we wrote ourselves and loaded into XFractint
(then reading the GIF it saved, or the error it reported), or from `.par`
files XFractint wrote through its interactive "save current parameters"
screen. No Fractint, XFractint or Iterated Dynamics source code, help text or
documentation was consulted, and none of the `.par` files bundled with
XFractint were used. Key names are the program's (they are the file format);
all other names and wording are ours. See [CLEANROOM.md](../CLEANROOM.md).

Conventions:

- **Confirmed** = a test shows it directly. **Hypothesis** = fits the
  evidence, not proven. **Unknown** = untested or undeterminable.
- Evidence names refer to `corpus/xfractint/par-map/` (README there). A
  bare case name (`s-tabs`, `c-cdigits`, …) is an amd64 batch render: its
  command line is in `cases.txt` (from `tools/blackbox/par-map/cases-*.txt`;
  input files in `tools/blackbox/par-map/inputs/`), its outcome in
  `results-read.txt`, and for some cases the GIF is kept as
  `read/<case>.gif`. `written/<name>.out.par` = files XFractint wrote
  (`cases-writer.txt`); `rt-<name>` = such a file loaded back
  (`results-roundtrip.txt`); `sp-*` = other interactive runs
  (`cases-special.txt`, files in `special/`).
- "Same as reference" means the saved GIF is identical to the one from the
  same settings given on the command line, apart from the random seed and
  calculation time (offsets 98–101 and 214–217 of `fractint001`, see
  [gif-fractint-blocks.md](gif-fractint-blocks.md) §3). The structure tests
  use the reference `s-ref`: `type=julia params=-0.8/0.156 maxiter=200
  corners=-1.5/1.5/-1.125/1.125 viewwindows=2`.
- XFractint version: Debian `xfractint 20.4.10-5`. The amd64 build (under
  qemu) was used for all reading tests. The writer tests ran the **arm64**
  build of the same Debian package natively (`tools/blackbox/par-map/docker`),
  because the `makepar=` command-line option segfaults in both builds. The two
  builds give identical pixels, palette and blocks except `fractint001` offset
  122–123 (an unknown field) for the same command line.

## 1. Summary for implementers

```
; anything after a semicolon is a comment
myentry            { ; optional comment
                     ; more comment lines
  reset=2004 type=julia corners=-1.5/1.5/-1.125/1.125
  params=-0.8/0.156 float=y maxiter=200 inside=0
  colors=000<3>zzz...\
  ...rest of the colour string
  }
```

- A file holds any number of **entries**. An entry is `name {`, then
  whitespace-separated `key=value` tokens, then `}`.
- The tokens are exactly the program's command-line arguments: an entry is a
  stored command line. XFractint reports the same "couldn't understand the
  argument" error for a bad token in a file and on the command line.
- Keys and keyword values are **lowercase only**. No spaces around `=`.
- `\` at the end of a line joins the next line (its leading whitespace
  removed). Lines are processed in pieces of at most 512 characters.
- Applied left to right; a later value wins. `reset` (or `reset=2004`)
  first returns everything except the palette to the defaults.
- An unknown key, a bad value or a malformed `colors=` makes XFractint **refuse
  the whole entry** (batch mode: no image; start-up: exits). FractalForge
  should warn and skip instead (§9).
- `colors=` holds the palette: 3 characters per entry from a 64-symbol
  alphabet (6-bit values), with `<n>` for n interpolated entries (§6).

## 2. File structure

### 2.1 Entries (confirmed)

| Rule | Evidence |
|---|---|
| The entry name and `{` must be on the **same line**. A name alone on a line followed by `{` on the next line is never found (error `'nextlinebrace' file entry item not found`). Entries after it are still found. | `s-nextlinebrace`, `s2-nextl-nextline`, `s2-nextl-after` |
| Space before `{` is optional (`nospace{`). The name line may be indented. | `s2-nospace`, `s2-indented` |
| The body may start on the `{` line and the whole entry may be one line: `name { type=julia ... }` | `s-samelinebrace` |
| The entry ends at the first `}` outside a comment, wherever it is on a line (`  maxiter=200 }`, a line `}` or `  }`). Text after it on that line is ignored (`... } maxiter=100` → maxiter stays 200). | `s-afterclose`, `s2-closesame`, `s-plain` |
| A `{` in the body (not in a comment) makes the entry **unfindable** (`'nested' file entry item not found`, also for a value `comment={x}`). A `name {` line inside another entry's body is found as an entry of its own. | `s-nested`, `s2-nest-nested`, `s3-bracevalue`, `s-nested-inner` |
| Braces inside comments are harmless: `name { ; comment } with a brace`, `... ; a { in a comment` | `s-comments`, `s3-bracecomment` |
| A `;` before the `{` on the name line hides the brace: `name ; comment {` is not found. | `s2-cbefore` |
| Blank lines and full-line comments may appear anywhere in a body. | `s-blanklines`, `s-comments` |
| Text outside entries (between them, before the first) is ignored when an entry is requested. | `s-plain` (file `r1.par` has a stray text line before the first entry) |
| CRLF line ends work. | `s-crlf` |
| Two entries with the same name: the **first** one is used. | `s-dup` |

### 2.2 Entry names (confirmed)

- The name is the first whitespace-delimited token on the line (`name with
  space {` is found as `name`; `s-namespace`, `s2-spname-name`).
- Matching is **case-insensitive** (`CaseName` requested as `casename` or the
  file's `plain` as `PLAIN`; `s-case-lower`, `s-plain-upper`) and exact, not a
  prefix (`plai` doesn't find `plain`; `s-prefix`).
- Punctuation in names works (`punct-name.x_1`; `s2-punct`).
- **Only the first 18 characters count**: requesting the first 18 characters
  of a 25-character name finds it (`s4-req-18-prefix`), and entries with long
  names don't disturb other entries (`s3-after-longname`).
- **Requesting a name of 19 or more characters crashes XFractint** (`***
  buffer overflow detected ***`), whether or not the file has it (`s2-len19`
  … `s2-len40`, `s4-req-long-*`). 18 works (`s2-len18`).
- Writer rule: names of at most 18 characters, no whitespace, no `{ } ;`.
  XFractint itself pads the name to 19 columns (§7).

### 2.3 Comments (confirmed)

`;` starts a comment that runs to the end of the line, anywhere: at the start
of a line, after the `{`, after tokens, and **without a space**
(`maxiter=200;maxiter=998` gives maxiter 200). `s-comments`.
Comment text following `{` on the name line is what XFractint writes as the
entry's description (§7).

### 2.4 Tokens and whitespace (confirmed)

- Tokens are separated by spaces or tabs (`s-tabs`) and line breaks.
- `key = value` with spaces is read as three tokens, and the bare `maxiter`
  is rejected: `Oops. I couldn't understand the argument: maxiter`
  (`s-spaceeq`).
- Keys are lowercase: `MaxIter=200` and `TYPE=JULIA` are
  rejected (`s-upper`, `s2-keycase`). Keyword values too: `type=JULIA`
  (`s2-valcase`), `passes=G` (`v-passesG`).
- Keyword values are not abbreviated: `inside=bof6` and `inside=max` are
  rejected (`v-insideabbr`, `v-insidemax`). The yes/no values accept `y`/`n`
  (§4).

### 2.5 Line continuation (confirmed)

A line ending in `\` is joined to the next line; the next line's leading
whitespace is dropped and **no separator is inserted**:

| Lines | Read as | Evidence |
|---|---|---|
| `params=-0.8/\` + `    0.156 maxiter=200 …` | `params=-0.8/0.156 maxiter=200` | `s-contin` |
| `params=-0.8/0.156 \` + `  maxiter=200` | the space before `\` separates | `s-continspace` |
| `… max\` + `iter=200` | `maxiter=200` (a key split across lines) | `s-continkey` |

XFractint uses this only inside `colors=` (§7), including splitting inside a
`<n>` run (`<3\` + `>G00`; `rt-wjulia` loads it correctly). Whether trailing
spaces after the `\` still count as a continuation is **unknown**: writers
must end the line with the `\`.

### 2.6 Line length (confirmed)

Physical lines are processed in pieces of **512 characters**: a token that
crosses character 512 is cut in two. Test `s3-split5NN`: a line of NN
characters of valid tokens and spaces, then ` maxiter=100`.

| NN | Result |
|---|---|
| 510 | error on the token `m` (characters 511–512 = ` m`) |
| 511, 512, 513 | `maxiter=100` applied: the piece boundary fell on a space |

A 2,400-character line failed on `maxite` (`s2-longline`, cut at 512). Lines
of ~250 characters work (`s2-long2`). Continuation joins logical lines but the
limit applies to each physical line. Writers: keep physical lines short
(XFractint writes at most 75 characters).

### 2.7 Files and loading (confirmed unless marked)

- Command line: `@file/entry`. Several may be given and mixed with other
  arguments; they are applied in order (`s3-two-entries`,
  `s3-two-entries-rev`, `s-before-par`, `s-after-par`).
- `@file` without `/entry` reads the file as a list of arguments (no braces);
  comments are allowed (`s-indirect` with `r3.txt`). Given a real `.par` file
  this way, the first stray word is rejected (`s-noentry`).
- A missing entry: `'<name>' file entry item not found`, no image
  (`s-missing`).
- **DOS 8.3 file names:** files whose base name is longer than 8 characters
  could not be opened (they reported `'<entry>' file entry item not found`;
  seen during development with `cmag-xmag.par`, so the round-trip files are
  named ≤ 8 characters).
- A **missing file** did not stop the search: `@nosuch.par/crlf` found the
  entry `crlf` in `r2.par` in the working directory (`misc` cases
  `x-nofile-crlf`, also `c-nofile`, `pa-nofile`), and `@nosuch.par/zzzz` reported `'zzzz' file entry item
  not found`. **Hypothesis:** XFractint falls back to other `.par` files.
  FractalForge need not copy this.
- Inside an entry, `map=` and `formulafile=` names are not looked up in the
  current directory (§6.5, §5.3).

## 3. Order, precedence and `reset` (confirmed)

- Arguments and entries are applied **left to right**; a later value
  replaces an earlier one, within an entry (`maxiter=200 … maxiter=100` → 100,
  `s-dupkey`) and across the command line (`maxiter=100 @r1.par/plain` → the
  entry's 200; `@r1.par/plain maxiter=100` → 100).
- An entry **only changes what it names**. Without `reset`, settings made
  earlier on the command line survive: `maxiter=77 inside=5 @…/noreset`
  keeps maxiter 77 and inside 5 (`s-inherit-noreset`), and an entry holding
  only `maxiter=200` keeps the earlier type, params and corners
  (`s-partial`).
- `reset` and `reset=<version>` restore the defaults at that point,
  including values set earlier **in the same entry** (`maxiter=77 inside=5
  reset type=julia …` → maxiter 150, inside 1; `r-resetmid`,
  `s-inherit-reset`, `s-inherit-reset2004`, `s2-reset-cmdline`).
- Defaults after `reset` differ from the plain start-up state in at least one
  value: **inside becomes 1** (start-up batch default is 0). XFractint's
  writer therefore always writes `inside=0` when inside is 0 and leaves it
  out when it is 1 (`written/wins1.out.par`).
- `reset` does **not** reset the palette (`c3-reset-palette`,
  `c3-reset-cmdline`: the `map=` palette survives).
- The number after `reset=` is a program version (XFractint writes 2004).
  `reset=2004` gives the same image as plain `reset` (`r-reset2004`).
  `reset=1700` (with `map=ff.map` before it) gave exactly the same file as
  the same settings without `reset=1700` (`r-reset1700` vs
  `c-map-cmdline-before`), so no effect of the number was seen for this
  image. `reset=abc` is rejected (`r-resetbad`). Writers should write
  `reset=2004`.
- `type=` does not reset parameters given before it in the same entry:
  `params=… corners=… type=julia` is the same as the usual order
  (`v-paramsfirst`, `v-cornersfirst`).
- `corners=` and `center-mag=` set the same view; whichever comes last wins
  (`v-bothcorncm`, `v-bothcmcorn`).

## 4. Value syntax (confirmed)

- Lists use `/` between values: `corners=-1.5/1.5/-1.125/1.125`.
- Real numbers are read as decimal floating point: `-8e-1`, `1.56E-1`,
  `15e-1`, `+1.125`, `-.8`, `.156` all work (`v-numexp`, `v-numdot`). 17
  significant digits (as XFractint writes) are read exactly
  (`-0.80000000000000004` → the double nearest −0.8; `rt-wjulia`).
- Integer keys accept a sign, a fraction (dropped: `maxiter=200.7` → 200) and
  an exponent (`maxiter=2e2` → 200) (`v-maxfrac`, `v-maxplus`, `v-numexp`).
- Non-numbers are rejected: `maxiter=abc` (`s-badvalue`).
- Missing trailing list values: `params=-0.8` sets param 2 to **0** (not the
  type's default 0.6) (`v-paramsshort`). An **empty** list value
  (`params=/0.156`) stores garbage (3.4028234663852886e+38 for param 1;
  `v-paramsempty`): never write empty values.
- Too few values for `corners` (3) or `center-mag` (2) are rejected
  (`v-corners3`, `v-cmag2`).
- Yes/no keys accept `yes`, `y`, `no`, `n` (`v-floaty`, `v-floatyes`,
  `v-floatn`, same for `finattract`). XFractint writes `y`.

## 5. Keys

### 5.1 Keys relevant to escape-time images

"Default" is the value after `reset`; XFractint's writer leaves a key out
when it has this value ("omitted"). "GIF" points at the `fractint001` field
in [gif-fractint-blocks.md](gif-fractint-blocks.md) §2.1 that the key sets,
which defines the value precisely. `k-<key>` vs `kc-<key>`: the
same settings from a `.par` entry and from the command line gave identical
GIFs for every key marked ✓.

| Key | Value syntax | Default / when XFractint writes it | GIF | Evidence |
|---|---|---|---|---|
| `reset` | none or `=version` | always written first, as `reset=2004` | — | §3 |
| `type` | lowercase type name (`mandel`, `julia`, `fn*fn`, `julfn+zsqrd`, `formula`, `mandel4`, `popcorn`, `lambda`, `newton`, …) | always written, second | [10], names → numbers in §6.1 there | all |
| `function` | 1–4 function names joined by `/` (`cos/sinh`) | written for types that use functions, all of the type's slots (`popcorn` → `function=sin/sqr/sinh/cosh`) | [218–221] | ✓ `k-function`; `wfnfn`, `wjulfn`, `wpopcorn` |
| `formulafile`, `formulaname` | file name; entry name | written for `type=formula` | [10], `fractint003` | `wfrm`; reading differs from the command line (`k-formula` ≠ `kc-formula`), see §5.3 |
| `passes` | `1` `2` `3` `g` `b` `t` `s` `d` `o` (lowercase) | `g`; written right after `type`/`function` | [200] | ✓ `k-passes`; `wg1`, `wg7`, `wg8`, `wg9` |
| `corners` | xmin/xmax/ymin/ymax, or + x3/y3 (6 values) | one of `corners`/`center-mag` is always written (§5.2) | [12–43], [184–199] | `s-*`, `wzoom`, `wcorn6` |
| `center-mag` | cx/cy/mag[/xmag[/rotation[/skew]]] (3–6 values) | as above | same | §5.2 |
| `params` | up to 10 reals | written with as many values as the type uses (mandel 2, newton 1, popcorn 4); none for `fn*fn` and for a formula that doesn't use them, even when non-zero (`wfrmpar`, `wpar10` drops params 3–10 for mandel) | [44], [52], [270], [278], [354–401] | ✓ via all julia cases |
| `float` | yes/no | XFractint always writes `float=y`; it made no difference in this build | none found | `v-float*`, gif spec §5.6 |
| `maxiter` | integer | 150; omitted when 150 | [8], [408] | `s-*`, `wg1`, `wg9` |
| `bailout` | integer | 0 (= type default); written when set explicitly, even `bailout=4` | [212], [402] | ✓ `k-bailout`; `wdefs` |
| `bailoutest` | `real` `imag` `or` `and` `manh` `manr` (default `mod`) | omitted when default | [406] | ✓ `k-bailout`; `wg1`, `wg7`–`wg10` |
| `inside` | colour number, negative number, or `maxiter` `zmag` `bof60` `bof61` `epscross` `startrail` `period` `atan` `fmod` | **1**; so `inside=0` is written for the start-up default (§3) | [104] | ✓ `k-inout`; `wg1`, `wg7`–`wg10`, `wins1` |
| `outside` | colour number or `iter` `real` `imag` `mult` `summ` `atan` `fmod` `tdis` | `iter`; omitted when default | [182] | ✓ `k-inout`; `wg1`, `wg7`–`wg10` |
| `fillcolor` | integer, -1 = off | -1 | [286] | ✓ `k-fillcolor` (tested with `passes=b`; both warn "Boundary tracing cannot be used with inside=0 or outside=0" and save the same image); `wg2` |
| `logmap` | `yes`/`y` (1), `no` (0), `old` (−1), integer | 0 | [106], [420] | ✓ `k-logmap`; `v-logmap*`; `wg2`, `wg7`, `wg8` |
| `potential` | maxcolor/slope/bailout[/`16bit`] | off | [86–97], [242] | ✓ `k-potential`; `wg4`, `wg10` |
| `invert` | radius/x/y | off | [428–451] | ✓ `k-invert`; `wg5` |
| `decomp` | n[/bailout] | 0 | [120], [402] | ✓ `k-decomp`; `wg3`, `wg8` |
| `distest` | n[/width[/xres/yres]] | 0; XFractint writes 4 values: `distest=1/71/400/300` (the last two are the image size) | [208], [268], [424] | `wg6` |
| `biomorph` | colour, -1 = off | -1 | [102] | ✓ `k-biomorph`; `wg2`, `wg10` |
| `symmetry` | `none` `xaxis` `origin` … (lowercase) | automatic; omitted then | [124] (none 0, xaxis 1, origin 4) | ✓ `k-symmetry`; `v-sym*`; `wg2`, `wg7` |
| `periodicity` | integer (0 off, negative allowed) | 1 | [240] | ✓ `k-periodicity`; `wg2`, `wg8` |
| `finattract` | yes/no | no | [222] | `v-float*`, `wg6` |
| `rseed` | integer | random | [98–100] | `wg6` |
| `cyclerange` | lo/hi | 1/255 | [264–267] | `wg6` |
| `viewwindows` | n/aspect/yes/x/y | off; XFractint writes `viewwindows=2/0.75/yes/0/0` when on | image size | all `w*` |
| `colors` | §6 | written last, when asked for | GIF palette | §6 |
| `map` | file name | never written by XFractint (it writes `colors=`) | GIF palette | §6.5, [map.md](map.md) |

### 5.2 `corners` and `center-mag` (confirmed)

`corners=xmin/xmax/ymin/ymax` gives an unrotated view. The 6-value form adds
the bottom-left corner: the stored points are top-left `(xmin, ymax)`,
bottom-right `(xmax, ymin)` and bottom-left `(x3, y3)`; with 4 values,
`(x3, y3) = (xmin, ymin)` (gif spec §2.1, `wcorn6` writes them back
unchanged).

`center-mag=cx/cy/mag[/xmag[/rot[/skew]]]` (defaults xmag 1, rot 0, skew 0)
gives the same corners by this rule, which reproduced every stored value to
within 2.2e-16 (`v-cmag3`…`v-cmag6`, `k-cmagrs`, `k-cmagxr`, `k-cmagneg`):

```
h = 1 / mag                 # half the height
w = h * (4/3) / xmag        # half the width
t = tan(skew degrees)
for each corner offset (dx, dy) in TL (-w, +h), BR (+w, -h), BL (-w, -h):
    dx = dx + dy * t                      # skew
    (dx, dy) = rotate (dx, dy) by -rot degrees   # i.e. clockwise
             = (dx*cos r + dy*sin r, -dx*sin r + dy*cos r), r = rot in radians
    corner = (cx + dx, cy + dy)
xmin, ymax = TL;  xmax, ymin = BR;  x3, y3 = BL
```

- The 4/3 factor is the image aspect for every image in these tests (800×600
  and 400×300). `viewwindows=2/1` did not change the image shape, so the
  factor's dependence on the window aspect is **unknown** (hypothesis: it is
  the screen's height/width ratio, 0.75).
- Inverse (for writers): `cx = (xmin+xmax)/2`, `cy = (ymin+ymax)/2`;
  `2w = |BR − BL|`, `rot = −atan2(BR.y − BL.y, BR.x − BL.x)`; rotate `TL − BL`
  by `+rot` to get `(2h·t, 2h)`; `mag = 1/h`, `xmag = h·(4/3)/w`,
  `skew = atan(t)`. XFractint's own output agrees: corners from
  `center-mag=-0.5/0/1/1.5/30/10` were written back as
  `center-mag=-0.5/0/1/1.5/29.9999999999999964/10.0000000000000018`
  (`wcmrot`).
- XFractint writes `mag` with **7 significant digits** (`0.6666667`), so its
  own center-mag output loses precision: loading `wmandel` gives xmin
  −2.499999900000005 instead of −2.5, and 60–700 pixels differ from the
  original image (`rt-*`). It reads more digits exactly
  (`center-mag=-0.5/0/0.66666666666666667` → xmin = −2.5 exactly;
  `v-cmagmandel`). FractalForge should write `corners=` with 17 significant
  digits (exact), or `center-mag=` with 17 digits.
- **Hypothesis** (writer's choice): XFractint writes `corners=` when the view
  was given with `corners=` and `center-mag=` otherwise, e.g. the default
  mandel view is written as `center-mag` (`wmandel`) but the same view given
  as corners is written as `corners` (`wcorndef`).

### 5.3 File-name keys inside entries (confirmed)

`formulafile=ff.frm` and even `formulafile=/work/ff.frm` failed inside an
entry (`'ffcube' file entry item not found`, `Could not open file where
formula located`; `pa-par-frmabs`), but worked on the command line
(`pa-cmd-formula`), and an entry with only `formulaname=` worked after
`formulafile=ff.frm` on the command line (`pa-par-frmnameonly`). `map=` in an
entry behaves the same way (§6.5). **Hypothesis:** in entries, file names are
looked up in the program's own data directories only. FractalForge should
look next to the `.par` file first, then in its own directories.

### 5.4 Unknown and invalid keys (confirmed)

| Input | XFractint's reaction | Evidence |
|---|---|---|
| `frobnicate=7` | message `Oops. I couldn't understand the argument:` + the token; batch: no image, exit status 1 | `s-unknownkey` |
| `frobnicate` (no `=`) | same | `s-unknownbare` |
| `maxiter=abc`, `inside=max`, `passes=G`, `corners` with 3 values | same | §4 |
| malformed `colors=` | same | §6.3 |
| unknown key at interactive start-up (`@r1.par/unknownkey` without batch) | message printed to the console, program exits | `sp-unknown` |

The whole entry is refused; nothing in it is applied. What the interactive
`@` menu does with a bad entry is **unknown**.

## 6. `colors=` palette strings

### 6.1 Alphabet (confirmed)

Each palette entry is three characters, red, green, blue, each one a 6-bit
value 0–63:

| Characters | Values |
|---|---|
| `0`–`9` | 0–9 |
| `A`–`Z` | 10–35 |
| `_` | 36 |
| `` ` `` (backquote) | 37 |
| `a`–`z` | 38–63 |

That is: `0`–`9` → c − 48; `A`–`Z` → c − 55; `_`…`z` (ASCII 95–122) → c − 59.
Case matters (`ABC` = 10, 11, 12; `abc` = 38, 39, 40). Evidence: `c-cdigits`
(every symbol in every channel, 256 explicit entries) matches exactly;
`c3-cupper`, `c3-clower`. The other printable characters `[ \ ] ^ | ~ ! #`
are rejected (`c3-cbad0`…`c3-cbad7`); `{` makes the entry unfindable
(`c-cbadchar`).

The GIF colour table then holds `(v << 2) | (v >> 4)` (gif spec §1.4).

### 6.2 Runs `<n>` (confirmed)

`<n>` between two entries A and B stands for **n entries interpolated
linearly** from A to B (A and B are written explicitly):

```
entry k (k = 1..n) = round_half_up(A + (B − A) · k / (n + 1))   per channel
round_half_up(x) = floor(x + 0.5)
```

Exact for every interpolated entry in `c-cinterp2` (`000<254>zzz`),
`c-cinterp3` (`zzz<254>000`), `c-cinterp4` (mixed rising/falling channels),
`c3-crun2/3/9/200`, and for all XFractint-written strings loaded back
(`rt-wjulia`, `rt-wmandel`, `rt-wmapyes`). Floor, truncation and round-half-
even each mismatch somewhere.

- `<1>` is **rejected** (`c3-crun1`), so is `<0>` (`c-crun0`). Use n ≥ 2.
- A run at the very start (`<3>zzz…`) is rejected (`c-cstartrun`).
- A run must be followed by an explicit entry (XFractint always does this).
  A run at the end is **unknown**; don't write one.

### 6.3 Length and errors (confirmed)

- At most **256** entries in total, runs included: 257 explicit entries
  (`k-c257`) or `000<254>zzz` + one more (`k-c256run`) are rejected; 256
  (`k-c256`) and `000<254>zzz` work.
- An incomplete last triple (`z00z00…z0`) is rejected (`c-cpartial`).
- Whitespace ends the token: `colors=z00z00 z00` leaves `z00` as a separate,
  rejected argument (`c3-cspace`).
- 256 entries take 768 characters, more than one physical line (§2.6), so
  use `\` continuation lines (`k-c256`, 60 characters per line, worked; the
  same string on one line, `c3-c256`, was rejected).
- Any rejection refuses the whole entry (§5.4).

### 6.4 Fewer than 256 entries (confirmed)

Entries not given are set to 6-bit **(40, 40, 40)**, GIF `a2 a2 a2`, not to
the previous palette (`c-cshort16`, `c-cshort1`, `c-short16-after-map`:
`map=ff.map` first, the short string still leaves grey). Same filler as a
short `.map` file ([map.md](map.md) §2.4). XFractint writes fewer entries
when the "# of colors" field is lowered (16: `colors=00z<14>3Qw`, `wmap16`;
40: `wnc40`), and such files load with grey for the rest (`rt-wmap16`,
`rt-wnc40`).

### 6.5 `colors=@file.map` and `map=` in entries (confirmed)

`colors=@name.map` loads a map file (map.md) from XFractint's **maps
directory** only (`/usr/share/xfractint/maps/` here), on the command line
too: `colors=@ff.map`, `@./ff.map`, `@/work/ff.map` all failed with `Could
not load color map /usr/share/xfractint/maps/ff.map` (`c-cmapref`,
`c3-cmapdot`, `c3-cmapabs`, `pa-cmd-colorsat`, `pa-cmd-colorsatabs`).
`map=` inside an entry behaves the same (`c3-mapdot`, `c3-mapabs`,
`c3-mapbare`, `c-mapthencolors`), while `map=` on the command line opens
the given path (`pa-cmd-mapabs`). A map copied into that directory is found
from an entry (`special/maps-dir-check.gif`, made by
`tools/blackbox/par-map/maps-dir-check.sh`). XFractint writes `colors=@<name>`
when its "record colors" answer is left at its suggestion `@<current map
name>`: `colors=@default.map` for the start-up palette (`wmkeep`, loads
fine: `rt-wmkeep`), `colors=@ff.map` after `map=ff.map` (`wmapkeep`, fails
to load: `rt-wmapkeep`).

### 6.6 What XFractint writes (confirmed: lossy)

The "record colors" answer `yes` writes the whole palette, compressed with
runs. The runs are **approximate**: loading the string back gives a palette
that differs from the original by at most **1** (6-bit) in some channels
(`rt-wjulia`: 29 entries differ; `rt-wmapyes`: 77 entries). The rule the
writer uses to choose runs was not determined (it is not greedy: the default
palette's grey ramp `5, 8, 11, 14, 17, 20` is written `555<3>HHHKKK`).
FractalForge should not copy this loss (§8).

Example (default start-up palette, `written/wmandel.out.par`):

```
  colors=00000e0e00eee00e0eeL0eeeLLLLLzLzLLzzzLLzLzzzLzzz000555<3>HHHKKKOO\
  O<3>ccchhhmmmssszzz00z<3>z0z<3>z00<3>zz0<3>0z0<3>0zz<2>0GzVVz<3>zVz<3>zV\
```

### 6.7 Decoder

```text
decode(s):                         # s = value after "colors=", continuations joined
    if s starts with "@": load map file s[1:]; return
    out = []; pending = 0
    i = 0
    while i < len(s):
        if s[i] == '<':
            j = index of '>' after i (error if none)
            n = integer s[i+1:j] (error unless n >= 2; error if out is empty)
            pending = n; i = j + 1; continue
        need 3 characters, each in the alphabet (error otherwise)
        B = the three values; i += 3
        if pending:
            A = out.last
            for k in 1..pending: out.push(round_half_up(A + (B−A)·k/(pending+1)))
            pending = 0
        out.push(B)
        error if len(out) > 256
    error if pending (run without an end, untested in XFractint)
    pad out to 256 with (40, 40, 40)
```

## 7. What XFractint writes (confirmed)

From `written/*.out.par` (interactive `b` screen, native build) and
`sp-multi`, `sp-into`:

```
wjulia             { ; our comment one
  reset=2004 type=julia corners=-1.5/1.5/-1.125/1.125
  params=-0.80000000000000004/0.156 float=y maxiter=200 inside=0
  viewwindows=2/0.75/yes/0/0
  colors=00000e0e00eee00e0eeL0eeeLLLLLzLzLLzzzLLzLzzzLzzz000555<3>HHHKKKOO\
  …
  GBFGBDGBCG000<6>000
  }

```

- **Name line:** the name left-aligned and padded with spaces to 19
  characters, then `{`. With a comment: `{ ; ` + comment 1. Without comment:
  `{` alone (`sp-into`). Comments 2–4 follow as separate lines of 21 spaces,
  `; `, the text (`sp-multi`).
- **Body:** lines start with 2 spaces; tokens separated by one space; a new
  line starts when the next token would pass column 72 (lines of up to 70
  characters observed for ordinary tokens). The screen's "maximum line
  length" field (default 72) set to 40 gave body lines of ≤ 40 characters
  and colour lines of 2 + 40 + 1 (`wll40`); 200 was ignored and 72 used
  (`wll200`). Tokens are never split, except `colors=`.
- **colors:** starts on a new line; split every 72 characters with `\`
  (lines of 2 + 72 + 1 = 75 characters), continuation lines indented 2.
- **End:** a line `  }`, then one empty line. LF line ends.
- **Key order** (each key only when written, see §5.1):
  `reset`, `type`, `function`, `formulafile`, `formulaname`, `passes`,
  `corners` | `center-mag`, `params`, `float`, `maxiter`, `bailout`,
  `bailoutest`, `fillcolor`, `inside`, `outside`, `logmap`, `potential`,
  `invert`, `decomp`, `distest`, `finattract`, `rseed`, `biomorph`,
  `symmetry`, `periodicity`, `viewwindows`, `cyclerange`, `colors`.
  (Every written file is consistent with this order. Keys that never
  appeared in the same file, e.g. `biomorph` and `rseed`, have a guessed
  relative order.)
- **Numbers:** the shortest form that reads back exactly for some values
  (`-0.7`, `0.156`, `1e+13`) and 17 significant digits for others
  (`-0.80000000000000004`, `9.9999999999999995e-21`); trailing zeros dropped
  (`0.131825904205330` → `0.13182590420533`); exponents as `e+13`, `e-21`.
  Mag always 7 significant digits. Hypothesis: `%.17g`, shortened when a
  shorter form (≤ 15 digits?) prints the same value.
- **Saving into an existing file:** a new name is appended at the end
  (after a blank line); an existing name (case-insensitive?) is replaced in
  place after a "File already has an entry named …" confirmation; the rest
  of the file (other entries, comments, stray text) is kept byte for byte
  (`sp-into`: diff of `r1.par` before/after shows only those changes).
- "Record colors" answers: `no` → no `colors=`; `yes` → full string;
  `@name` (the suggestion) → `colors=@name`; `only for full info` (`only`)
  → an entry holding **only** `colors=` (`wmaponly`).

## 8. Writer recipe (FractalForge → XFractint)

1. Entry name: 1–18 characters, letters, digits and `_-.`; no whitespace.
   Write `name` padded to 19 columns, `{`, and ` ; comment` if there is one.
   Extra comment lines: 21 spaces + `; text`.
2. Body lines: two spaces indent, tokens separated by single spaces, wrap
   before column 72. Lowercase keys and keywords, no spaces around `=`.
3. Always start with `reset=2004 type=<name>`. Then the view and settings in
   the §7 order. Write every non-default setting, and write `inside=` always
   (the post-`reset` default is 1, which surprises everyone).
4. View: `corners=xmin/xmax/ymin/ymax` (add `/x3/y3` only when rotated or
   skewed), each value with 17 significant digits or the shortest
   round-trip form (Rust's `{}` / ryu formatting of an `f64` is exact).
   Prefer this to `center-mag`, whose mag XFractint would print with 7
   digits.
5. `params=`: all values the type uses, never an empty value; for
   formulas write all non-zero params up to the last non-zero one.
6. Palette: write `colors=` with **256 explicit entries** (no runs), or with
   runs only where the §6.2 formula reproduces every entry exactly and
   n ≥ 2. Start it on a new line, split with `\` every ≤ 72 characters, and
   never write a `colors=@file` or `map=` reference (XFractint resolves those
   in its own directory only).
7. Close with `  }` and an empty line. Use LF line ends.
8. Don't write keys XFractint wouldn't understand (FractalForge-only settings
   must go elsewhere, or XFractint refuses the whole entry).
9. File name: base name ≤ 8 characters if the file is meant for XFractint
   under DOS-style naming (this build could not open longer ones).

Verified: every XFractint-written file loads back to the same blocks
(except the documented center-mag precision loss) and, with `colors=yes`,
a palette within ±1 (`rt-*`); our own 256-entry explicit strings load
exactly (`c-cdigits`, `k-c256`).

## 9. Reader recipe (tolerant)

1. Read the file; split into lines on LF, strip a trailing CR.
2. Find entries: a line whose first token (after leading whitespace) is the
   name, directly followed (with or without spaces) by `{`, before any `;`.
   Match names case-insensitively on the first 18 characters; take the first
   match. To be more forgiving than XFractint, FractalForge may also accept a
   `{` alone on the next line and ignore `{` inside the body; warn when it
   does so, because XFractint would not find such an entry.
3. Body: from after `{` up to the first `}` not inside a comment. For each
   line: drop everything from `;`; if the line (before the comment) ends with
   `\`, remove it and append the next line with its leading whitespace
   removed. (XFractint also cuts physical lines at 512 characters; a reader
   need not imitate this, but should warn.)
4. Split on spaces/tabs into tokens; split each at the first `=` into key and
   value (value may be absent).
5. Apply tokens left to right on top of the current state; `reset[=n]`
   restores defaults except the palette. Later tokens win.
6. Unknown keys, keyword values in the wrong case, and malformed values:
   **XFractint rejects the whole entry** (§5.4). FractalForge should instead
   record a warning per bad token, skip it, and apply the rest; in a
   "strict / XFractint-compatible" check mode, report the entry as one
   XFractint would refuse. Keys XFractint writes that FractalForge doesn't
   implement (e.g. `viewwindows`, `float`, `cyclerange`, `rseed`, 3D keys)
   should be kept (round-tripped) or ignored silently.
7. Numbers: parse as `f64` decimal (accept `+`, leading `.`, exponents);
   for integer keys truncate toward zero. Missing list values → 0; empty list
   values → treat as invalid (XFractint stores garbage).
8. `colors=`: §6.7. `map=`/`colors=@`: look next to the `.par` file, then
   in FractalForge's palette directory; the palette stays unchanged on
   failure (warn).

## 10. Unknowns

- Behaviour of the interactive `@` selection screen with a bad entry.
- Trailing whitespace after `\`.
- Whether a `<n>` run may end the string.
- The writer's run-selection rule (§6.6) and exact number formatting rule.
- `center-mag` for windows that are not 4:3.
- `reset=` versions other than 1700 and 2004.
- Every key not listed in §5.1 (3D, stereo, orbit, L-system/IFS keys): the
  gif spec shows their storage; their `.par` syntax was not tested beyond
  "same as the command line".

## 11. Verification performed

| Area | Cases (`tools/blackbox/par-map/cases-*.txt`) | Result |
|---|---|---|
| structure, names, comments, continuation, length | `cases-structure*.txt` (`s-`, `s2-`, `s3-`, `s4-`) | §2 |
| precedence, reset | `s-*before/after/inherit*`, `cases-reset.txt` | §3 |
| value syntax, center-mag | `cases-values.txt` (`v-`), `cases-keys.txt` (`k-cmag*`) | §4, §5.2 (fit error ≤ 2.2e-16) |
| par vs command line, 15 keys | `cases-keys.txt` (`k-` vs `kc-`) | identical GIFs |
| colors decoding | `cases-colors.txt`, `c3-*` in `cases-values.txt`, `k-c25*` | §6 |
| file names | `cases-paths.txt`, `maps-dir-check.sh` | §5.3, §6.5 |
| XFractint as writer | `cases-writer.txt` (42 files), `cases-special.txt` | §7 |
| round trip | `cases-roundtrip.txt` | §8 |
