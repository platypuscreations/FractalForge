<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# L-system fractals (`type=lsystem`, type number 107)

This spec says how XFractint reads L-system files (`.l`), which entries it
accepts or rejects (and with which message), how it rewrites the axiom, what
every turtle command does, and exactly where every line lands in the image,
down to the pixel. It is precise enough to reproduce XFractint's saved images
bit for bit: our own model written from this spec
(`tools/blackbox/lsystem/lsysmodel.py`) reproduces every image case of the
evidence corpus that it was run on (1,128 cases, plus orders up to 200) with
**zero** differing pixels (§9).

Everything here was found black-box: we wrote our own `.l` files, rendered
them with XFractint and looked at the saved pixels, the saved GIF blocks and
the messages. No Fractint, XFractint or Iterated Dynamics source code, help
text, documentation or bundled `.l` file was used (see
[CLEANROOM.md](../CLEANROOM.md)). Published turtle-graphics and L-system
ideas were only hypotheses until the pixels confirmed them. Names are our
own. Labels: **confirmed** (seen directly, cited), **hypothesis** (fits
everything we saw, not proven), **unknown**.

---

## 0. Method and evidence

- Program: Debian `xfractint 20.4.10-5`, amd64 under qemu (image
  `fractalforge-blackbox`), batch mode, via
  `tools/blackbox/lsystem/run-case.sh`, which runs
  `xfractint batch=yes <arguments> savename=ff.gif` in a fresh directory that
  holds only the input files the case names (`only=` in `cases.txt`).
- Most cases use `viewwindows=2` (a 400×300 image) and `map=uniq.map` (a
  palette with 256 different colours, so saved colour indices are exact).
  Other sizes are set with `viewwindows=1/0.75/no/W/H`.
- Evidence lives in `corpus/xfractint/lsystem/<round>/`. A case is cited as
  `round/case`, for example `fmt/f-base`; its arguments are in `cases.txt`,
  its outcome is the line of the same name in `<round>/results.txt` (pixel
  hash, bounding box of drawn pixels, colour counts, stored type, param 1,
  status, `fractint003` name, stop message, console output), and its image
  and message files sit next to it. The `.l` file a case reads is the one
  its `lfile=` names, in `tools/blackbox/lsystem/inputs/`.
- Batch messages go to `stopmsg.txt` (kept as `<case>.stopmsg.txt`). In
  every error case below XFractint still saves an image, all colour 0,
  unless the case says it crashed.
- The native arm64 build of the same package was used for quick exploration
  only. Where a coordinate lies exactly on a pixel boundary it saves
  different pixels from the amd64 build (seen on 12 of the 192 `seg` cases; runs
  not kept). **All evidence and all statements below are for the amd64
  build.** §6.2 explains why the two differ.

---

## 1. Summary

1. The file is plain text. An entry is `name { ... }`; inside, `angle N`
   gives the turning step 360/N degrees (3 ≤ N ≤ 50), `axiom S` the start
   string and lines `X=S` the rewriting rules. Text is case-insensitive
   (everything is lowercased) and `;` starts a comment.
2. `lfile=` names the file, `lname=` the entry and param 1 (`params=`) the
   order, default 2. The axiom is rewritten *order* times, all symbols at
   once.
3. The result is drawn with a turtle: `F` and `D` draw, `G` and `M` move,
   `+ - | !` turn on a table of N directions, `/ \` turn by degrees (for `D`
   and `M` only), `[ ]` push and pop, `C < >` set the colour, `@` scales the
   step. Everything else is ignored.
4. Drawing happens twice: once at unit size to find the extent, once for
   real. The drawing is fitted into the image with margins of 5 pixels left
   and right and 3 above and below, centred, with a horizontal aspect factor
   0.75·W/H. The arithmetic is x87 extended precision with a float32 scale;
   points are rounded half up and joined with a rounded straight line.
5. Background colour 0, default line colour 15. An interrupted image cannot
   be resumed.

---

## 2. The `.l` file

### 2.1 Finding the file (confirmed)

- `lfile=NAME` is opened in the working directory. Names are cut to DOS 8.3:
  `lfile=fmt-angle6.l` opened a file `fmt-angl.l` when one existed, and a
  file actually named `fmt-angle6.l` could not be opened (seen during
  exploration, not kept; the same as for `.par` files, [par.md](par.md)
  §2.7). With a long `lfile=` name the entry was still found, through the
  search below (`fmt3/f-lk-long-b`). The tool scripts use short names.
- The `.l` extension may be left out and case does not matter:
  `lfile=lkc` and `lfile=LKC.L` read `lkc.l`, not the other candidate
  `lkb.l` with a different entry of the same name (`fmt3/f-lk-noext-*`,
  `f-lk-upper-*`).
- **Search of other files.** When the named file cannot be opened, *or* it
  has no entry of that name, XFractint looks for the entry in the other
  `.l` files of the working directory: `fmt3/f-lk-noentry` (entry missing
  from `lka.l`, found in `lkb.l`), `f-lk-nofile-bc`, `f-lk-nofile-cz`,
  `f-lk-nofile-ab` (no such file; the entry came from `lkb.l` or `lkc.l`).
  With two candidates it took `lkb.l` over `lkc.l` and `lkc.l` over `lkz.l`
  (hypothesis: alphabetical or directory order). Only when no file has the
  entry does it report `'NAME' file entry item not found` (`f-lk-nofile-a`).
  FractalForge need not copy this search, but any test against XFractint
  must keep other `.l` files out of the working directory.
- Without `lfile=` the entry `e` was not found (`fmt2/f-nolfile`); the
  default file is XFractint's own (hypothesis; not examined).

### 2.2 Lines (confirmed)

- Lines end at LF; CRLF files read the same (`fmt/f-crlf`). Blank lines and
  any amount of spaces or tabs are allowed anywhere (`f-blank`, `f-tab`,
  `f-spaces-many`, `f-indent0`).
- `;` starts a comment that runs to the end of the line, anywhere: before
  the entry, after `{`, on its own line, after a value, after `}`
  (`fmt/f-cmt-*`). A `;` inside an axiom ends the axiom there
  (`chars/c059`: `F;F+F` draws one segment). A `;` before the `{` hides the
  brace (`fmt3/f-comment-in-name`, `fmt/f-cmt-brace`). `#` is **not** a
  comment (§2.4).
- A physical line longer than 255 characters is cut after its 255th
  character and the rest is read as the **next line** (with the next line
  number). Example: an axiom line of 262 characters gives
  `Syntax error line 4: xxxxf+f`, the last 7 characters (`fmt2/f-axlen255`,
  `-256`, `-300`, `f-rulelen255` …). Lines of about 1,000 characters or more
  crash XFractint after the message (`f-axlen1000`, `f-rulelen1000`,
  `deg2/deg2-e22`: "stack smashing detected").

### 2.3 Entries (confirmed)

- An entry starts on a line that contains `{`. Its **name** is the text
  before the `{`, up to the first space, tab, `(` or `[`: `e x {` is entry
  `e` (`fmt2/f-name-twowords`), `a(b) {` and `a[b {` are entry `a`
  (`fmt3/f-name-paren-a`, `f-name-brack-a`; `lname=a(b)` is not found).
  Other punctuation is part of the name (`- . _ ! / =` and a leading digit:
  `fmt3/f-name-dash` …). Leading spaces are allowed (`f-name-indent`), the
  brace may touch the name (`fmt/f-nospace-brace`, `fmt3/f-name-glued`).
- If the `{` is on the line after the name, the entry's name is empty: it
  is found only without `lname=` (`fmt3/f-brace-nextline`,
  `-nextline-empty`).
- Names are compared without regard to case and must match exactly:
  `lname=E` finds `e`, `lname=eex` finds `EeX` (`fmt/f-lname-upper`,
  `fmt3/f-name-upper`); `lname=e` does not match `ee` (`fmt/f-prefix`). With
  two entries of the same name the first one is used (`fmt/f-dupname`).
- An entry is only found if a `}` comes after its `{` (later on the `{` line
  or on a later line) before the end of the file and before the next line
  that contains a `{`. Missing `}` at the end of the file
  (`fmt/f-noclose`, `fmt3/f-noclose-indented`, `f-noclose-col0`; with the
  `}` added it is found, `f-noclose-col0-close`), a second `{` line before the `}`
  (`fmt/f-noclose2`, `fmt3/f-after-open-entry`, `f-axiom-open`,
  `f-rule-brace`) and a `{` inside an axiom (`chars/c123`) all give
  `'NAME' file entry item not found`. A `}` anywhere in a line counts for
  this test, even one glued to a value (`lhs/gl1`).
- Text after the `{` on the name line is read as the entry's first line
  (`fmt/f-after-open`, `-open2`, `fmt3/f-name-glued`). Text after a closing
  `}` is not part of the entry (`fmt/f-after-close`: `} junk`). The search
  for entries looks at every line with a `{`, so a line `}e {` both closes
  one entry and starts entry `e` (`fmt3/f-second-noblank`); the name is
  then the text before `{` with the `}` removed (hypothesis: the search
  skips `}` like a separator).
- The body is read line by line, starting with the rest of the `{` line, and
  ends at the first line whose first word starts with `}`
  (`fmt2/f-close-first`: a line `} axiom F` ends the entry, so the axiom
  given before it counts and nothing after it). A `}` glued to the end of a
  value is just a character of that value and reading continues
  (`fmt3/f-axiom-close`, `f-rule-close`, `lhs/gl2`: a later `angle 6`
  applies). `axiom F+F+F }` with a space is an error
  (`Extra text after command line 3: }`, `fmt/f-close-sameline`).

### 2.4 Body lines (confirmed)

The whole entry text is lowercased first: messages show lowercase text
(`args/a-atsp`) and upper- and lowercase forms of keywords, rule symbols and
commands behave identically (`fmt/f-kw-upper`, `f-kw-mixed`,
`f-rule-lower`, `f-rule-lower2`, `chars` pairs such as `c070`/`c102`).

A body line is split into words. The **first word** ends at a space, tab or
`=`; then any spaces, tabs and `=` signs are skipped; the **value** is the
next run of characters up to a space or tab (it may contain `=`). A third
word is an error: `Extra text after command line N: WORD`, where WORD is the
third word (`fmt/f-angle-extra` → `5`, `f-axiom-space` → `+f`,
`f-rule-sp4` → `f`, `f-rule-space-in` → `-f`, `args/a-atsp` → `2f+f`).
N counts lines from the entry's name line (= 1), including blank and
comment lines and the extra lines made by §2.2. Then the first word decides:

| First word | Meaning |
|---|---|
| `angle` | value = number of directions N (§2.5) |
| `axiom` | value = start string |
| one character X | a rule: X is replaced by the value (§4) |
| anything else | error `Syntax error line N: WORD` (WORD = the first word, lowercased) |

- Keywords must be spelt out and separated: `ang 4`, `ax F`, `angle4`,
  `axiomF+F+F` are syntax errors (`fmt/f-kw-abbrev`, `f-kw-nospace`), as
  are `bogus 7` and `hello` (`f-unknown-kw`, `-word`). `angle=F` is the
  keyword `angle` with value `f` (error, `f-rule-angle`); `axiom=F` sets the
  axiom to `f` (`f-rule-axiom`).
- `angle` and `axiom` may come in any order and before or after rules
  (`f-axiom-first`, `f-rule-first`). Given twice, the **last** one counts
  (`f-angle-twice`, `f-axiom-twice`, `fmt3/f-last-wins`).
- **Rules.** `F=F-F`, `F = F-F`, `F= F-F`, `F =F-F` and `F F-F` (no `=`)
  all define F → `f-f` (`fmt/f-rule`, `-sp`, `-sp2`, `-sp3`,
  `fmt2/f-rule-nosep`). A rule line without a value defines an empty
  replacement: `X=` (`fmt/f-rule-empty`), `F` alone (`fmt2/f-rule-nosep2`),
  and `=F` (the `=` is skipped and `F` becomes the first word,
  `fmt/f-rule-nolhs`). A two-character first word such as `FF=F-F` is a
  syntax error (`f-rule-multi`).
- **Several rules for one symbol are joined**, in file order: `F=F+F` then
  `F=F` rewrites F to `f+ff` (`misc/dup-r5-p1` equals the image of
  `F+FF`); `dup-r1` … `dup-r6`, `fmt/f-rule-twice`.
- **Reserved symbols.** A rule for one of `+ - | ! [ ] @ < > / \ C` (either
  case of C) is an error `Syntax error line N: Redefined reserved symbol X`
  (`lhs/h033` … `h124`, `fmt/f-rule-plus`, `-bracket`, `-c`). Every other
  printable character may have a rule, including `F G D M`, digits and `#`
  (`lhs/h034` …; `#` lines are rules, not comments: `fmt2/f-rule-hash`).
  `;` cannot (comment) and `{`, `}`, `=` were not tested as rule symbols.

### 2.5 Values (confirmed)

- **angle**: the value is read like C `atoi` (optional sign, then digits,
  the rest ignored): `4.5` → 4, `6.9` → 6, `06` → 6, `+6` → 6, `6x` → 6,
  `0x6` → 0 (`fmt/f-angle45`, `fmt2/f-angle69`, `-angle06`, `-anglep6`,
  `-angle6x`, `-angle-hex`). Valid values are **3 to 50**; 0, 1, 2, 51,
  360, 1000, −4, `x` and a missing angle line give
  `Error:  illegal or missing angle` (two spaces after the colon)
  (`fmt/f-angle0` … `f-anglem4`, `f-noangle`, `f-angle-text`).
- **axiom**: any characters up to the first space or tab (`}` and `=`
  included, §2.3). A missing axiom line gives `Error:  no axiom`
  (`fmt/f-noaxiom`).
- `angle` or `axiom` **without a value** crashes XFractint (segmentation
  fault, `fmt/f-angle-noval`, `f-axiom-noval`).

### 2.6 Messages (confirmed)

Errors are collected while the entry is read; then `Error:  no axiom` and
`Error:  illegal or missing angle` are added if they apply, in that order
(`fmt/f-empty`, `f-kw-abbrev`, `f-one-line`). Each message is a line of
`stopmsg.txt`. After any error nothing is drawn: the image is all colour 0
and is still saved with the normal blocks.

| Message (exact text) | Cause | Evidence |
|---|---|---|
| `'NAME' file entry item not found` | no entry NAME in any searched file (§2.1, §2.3); NAME as typed | `fmt/f-missing`, `fmt3/f-noclose-col0` |
| `'' file entry item not found` | `lname=` not given and no entry with an empty name | `fmt/f-nolname` |
| `Syntax error line N: WORD` | unknown first word | `fmt/f-unknown-kw` |
| `Syntax error line N: Redefined reserved symbol X` | rule for a reserved symbol | `lhs/h043` |
| `Extra text after command line N: WORD` | a third word on a line | `fmt/f-angle-extra` |
| `Error:  no axiom` | no axiom line | `fmt/f-noaxiom` |
| `Error:  illegal or missing angle` | angle missing or outside 3–50 | `fmt/f-angle51` |
| `Oops. I couldn't understand the argument:` / `  lname=…` / help hint | `lname=` longer than 18 characters; no image is saved | `fmt2/f-name24`, `fmt3/f-lname19` |
| `View window too large; using full screen.` | `viewwindows` size above 800×600; the image is 800×600 | `ext/x-plant-o3-1024x768` |

### 2.7 Limits (confirmed unless marked)

- Line length 255 characters (§2.2), so an axiom or one rule line holds at
  most 255 minus the characters before its value (`fmt2/f-axlen200` works).
- 300 rule lines for one symbol were accepted and joined (`fmt2/f-nrules300`,
  299 lines `X=X`), and 1,000 comment lines (`f-nlines1000`). Upper limits
  on the number of rules, the joined rule length and the number of entries
  are **unknown**.
- `lname=` up to 18 characters (`fmt3/f-lname18` works, `-lname19` does
  not). An entry whose name in the file has 24 characters was found by
  `lname=` with its first 18 (`fmt3/f-name18-file24`), so names in the file
  are compared on at most 18 characters (hypothesis, one case; a shorter
  `lname` is not a prefix match, `fmt/f-prefix`).
- Crashes (no image saved): unbalanced `[` (§5.3), `angle`/`axiom` without
  value, lines of about 1,000 characters or more.

---

## 3. Command line and stored parameters (confirmed)

- `type=lsystem lfile=FILE lname=ENTRY params=ORDER`. **Param 1 is the
  order**: default 2 (`sem/s-ko-def` equals `s-ko-2`); fractional values are
  truncated (`2.5`, `2.9` → 2, `0.5` → 0); negative values act as 0
  (`s-ko-m1`, `-m0p5`); `1e1` is 10 (`s-ko-1e1` equals `s-kobig-10`).
- No order limit was found: orders 10 and 12 of a four-segment rule
  (1M and 16M segments) and order 24 of `F=FF` (16M segments) were
  computed (the image of order 12 equals that of order 10 because the curve
  has converged at this resolution, but the stored calculation time grew
  from 3.3 s to 48.7 s; `F=FF` order 24 took 44.8 s; `sem/results.txt` and
  the time field of the GIFs). Orders 16 and above did not finish within 300 s
  (`sem/s-kobig-*`, logs). Time grows with the number of symbols drawn.
- Params 2–10 are stored but change nothing (`misc/p2-ko-*`). `corners=`
  is stored but changes nothing, also with a 3rd corner (`misc/cor*`).
  `maxiter`, `passes`, `symmetry`, `inside`, `outside` and `float` change
  nothing (`misc/mi-ko3-*`, `misc/geo-*-fl`, `misc/fl-*`).
- `fractint001` (layout in [gif-fractint-blocks.md](gif-fractint-blocks.md)
  §2.1) differs from the `mandel-base` baseline only in: type (offset 10)
  = **107**; corners (12–43, 184–199) = **−1, 1, −1, 1** and 3rd corner
  (−1, −1) unless `corners=` is given; param 1 (44) = the order **as typed**
  (f64: 2.5, −1, 10 …), params 2–4 and 5–10 as given; status (202); total
  extension length (204) = **603** (521 + 82 for `fractint003`); plus the
  seed and time fields. Pass mode stays `g`, maxiter 150.
- `fractint003` (66 bytes): the entry name **as typed in `lname=`** (case
  kept: `E`, `fmt/f-lname-upper`), NUL-padded in the first 48 bytes; byte
  48 = 0 (formulas have 1 there). The file name is not stored anywhere, so
  a reader must be told the file (§7).

---

## 4. Rewriting (confirmed)

- The current string starts as the axiom and is rewritten *order* times.
  Each step replaces **every** symbol that has a rule by its (joined)
  replacement, all at once; symbols without a rule stay. `A → B+F`,
  `B → A-F`, axiom `A`: order 1 is `B+F`, order 2 is `A-F+F`
  (`fmt/f-simul`, `f-simul2`).
- Rules apply to any symbol, including drawing commands and digits
  (`fmt/f-rule-digit`, `ext/x-ruled`, `x-rulem`).
- **Command arguments are not rewritten.** The characters that make up the
  numeric argument of `@`, `/` and `\` (§5.1) are copied unchanged:
  `F@2F+F` with rule `2=FF` draws like `F@2F+F` (`fmt/f-rule-at`,
  `ext/x-prot4`, `-prot5`; `x-prot3` for `/`).
- The arguments of `C`, `<` and `>` are **protected in the drawing pass but
  not in the extent pass** (§6.4): with rule `1=F`, `FC12F` is drawn as two
  segments, the second in colour 12, but the extent pass sees `FCF2F`
  (three segments), so the picture is scaled for three units and the third
  unit stays empty (`ext/x-prot1`; `x-prot2` with `<12` and rule `2=F`;
  `x-prot6` with `C1.5` and rule `5=FF`). This only matters when digits or
  `.` have rules.
- Hypothesis: XFractint does not build the rewritten string but walks the
  rules recursively, once per pass; the observations are the same.

---

## 5. Turtle commands

### 5.1 Command table (confirmed)

Each character of the rewritten string is one command; upper and lower case
are the same. *n* is an unsigned decimal number read from the characters
right after the command (digits and `.`; §5.2).

| Char | Action | Evidence |
|---|---|---|
| `F`, `D` | draw a line one step forward | `chars/c070`, `c068` |
| `G`, `M` | move one step forward without drawing | `chars/c071`, `c077` |
| `F`, `G` | use the **direction index** k (set by `+ - |`) | `mv/*` |
| `D`, `M` | use the **degree heading** (set by `/ \`) | `mv/mv-m01`, `deg/*` |
| `+` | k ← k − 1 (mod N): turn 360/N° counter-clockwise on screen | `misc/geo-g2` |
| `-` | k ← k + 1 (mod N): clockwise | `misc/geo-g3` |
| `|` | k ← k + ⌊N/2⌋ (mod N): half turn for even N, clockwise ⌊N/2⌋ steps for odd N | `sem/s-pipe3` … `-pipe9` |
| `!` | swap the meaning of `+` and `-` and of `/` and `\` | `args/a-bang`, `deg/deg-d10a4` |
| `/` *n* | degree heading += *n* (clockwise on screen) | `deg/deg-d00a4` |
| `\` *n* | degree heading −= *n* | `deg/deg-d01a4` |
| `[` | save position, k, degree heading, colour, `!` state, step size | `args/a-push-*` |
| `]` | restore the last saved state; with nothing saved: **stop** (§5.3) | `sem/s-pop*` |
| `C` *n* | colour ← *n* mod 256 | `args/a-c*` |
| `<` *n* | colour ← (colour + *n*) mod 256; if that is 0, colour ← 1 | `args/a-lt*`, `sem/s-lt*` |
| `>` *n* | colour ← (colour − *n*) mod 256; if that is 0, colour ← 255 | `args/a-gt*`, `sem/s-gt*` |
| `@` [`I`][`Q`] *n* | step size ← size × *n*; with `Q` use √*n*; with `I` divide by it | `args/a-at*`, `sem/s-at*` |
| anything else | nothing (digits, `.`, `=`, `}`, unknown letters…) | `chars/*` |

Details:

- The two headings are independent: `+`, `-`, `|` never affect `D`/`M`
  and `/`, `\` never affect `F`/`G` (`mv/mv-m09` `F+D` draws straight on,
  `deg/deg-d08a4` `D/90F`, `misc/sl-*` `F/45F` is straight). Both start
  pointing right (+x on screen). `D` and `M` draw and move exactly like `F`
  and `G` otherwise (same step size, same colour, `@` applies).
- `+` and `-` take no argument: digits after them are ignored
  (`args/a-plus2`, `chars/d043`).
- `!` affects `+ - / \` but not `|` (`sem/s-bangpipe`); `!!` cancels
  (`args/a-bang2`). Its state is saved by `[` (`args/a-push-bang`).
- The degree heading is **not** reduced: `/450` equals `/90`
  (`deg-d06a4`), but `/360` is not quite `/0` (`deg-d19a4` differs from
  `deg-d18a4`), because 360° goes through the trigonometric functions
  (§6.3).
- Colours: `C0`, `C256` set colour 0, which draws in the background colour
  and **does overwrite** earlier lines (`sem/s-erase`, `s-erase2`);
  `C300` → 44, `C1000` → 232, `C65538` → 2, `C2.5` → 2, `C007` → 7.
  `C` without digits (also `C+5`, `C-1`) sets colour 0, and the `+`/`-`
  then act as turns (`args/a-cm1`, `sem/s-cplus`). `<`/`>`: 15 + 241 → 1,
  15 + 300 → 59, 15 + 497 → 1, 15 + 1000 → 247, 15 − 15 → 255,
  15 − 20 → 251, 15 − 271 → 255, 15 − 1000 → 39; `C0<0` → 1, `C0>0` → 255
  (`sem/s-c0lt0`, `s-c0gt0`). `<` or `>` without digits adds or subtracts 0
  (then the zero rule applies).
- `@`: `@2`, `@0.5`, `@.5`, `@1.5`, `@10` multiply; `@Q4` and `@q4` = ×2,
  `@I2` = ÷2, `@IQ4` = `@QI4` = ÷2, `@QQ16` = ×4 and `@II2` = ÷2 (a
  repeated letter counts once), `@2@2` = ×4. Without digits, *n* = 0:
  `@`, `@I`, `@Q`, `@I0` all make the step size 0 (`sem/s-ati`, `-ati0`,
  `-atq`). `@-1` is `@` (size 0) followed by a `-` turn; `@1e1` is `@1`
  (`args/a-atm1`, `a-ate1`). A space after `@` is an error (`a-atsp`).
- Branch nesting of at least 100 levels works (`sem/s-nest100`,
  `s-nestrule-200`).

### 5.2 Numbers (confirmed)

After `C < > / \` the scan takes all following digits and dots; after `@` it
first takes any `I`/`Q` letters, then digits and dots. The value is the
longest prefix of the form `digits[.digits]` (`@2.5.5` = `@2.5`,
`sem/s-at255`; `C1.5` = colour 1). For `C`, `<`, `>` the value is
truncated to an integer. There is no sign and no exponent. All the scanned
characters belong to the command (they are never rewritten, §4, and never
executed).

### 5.3 Unbalanced brackets (confirmed)

- A `]` with nothing saved **ends** the drawing: nothing after it is drawn,
  and it also ends the extent pass, so nothing after it counts for the fit
  (`sem/s-pop3`: `F]+FFF` is one full-width segment; `s-pop0`: `]F` draws
  nothing; `s-pop1`, `s-pop2`, `s-pop4`).
- A `[` that is never closed crashes XFractint (segmentation fault, no
  image): `[F`, `[[F]`, `F[F`, `F[+F` (`sem/s-push1` … `-push3`,
  `args/a-pushend`, `chars/c091`).

---

## 6. Rendering

### 6.1 Overview (confirmed)

The string is interpreted twice with the same turtle (§5): **pass 1** from
position (0, 0) with step size 1 records the smallest and largest x and y of
every position the turtle reaches (the start and the end of every `F G D M`,
drawn or not, inside branches too: `sem/s-bbmove`, `s-bbbranch`,
`misc/dup-r4-p1`); colour commands are ignored in this pass (§4). Then the
scale and the start point are chosen (§6.4) and **pass 2** draws. Screen
coordinates are used throughout: x to the right, y **down**, so heading 0
points right and `+` (k − 1) turns towards the top of the image.

### 6.2 Arithmetic (confirmed by exact reproduction)

To reproduce XFractint's pixels, the following arithmetic must be followed
exactly; ties at half pixels are very common (any vertical line through the
centre of an even-width image, every corner of a square, …) and they are
decided by rounding errors of the order of 1e-14.

- **ext(v)**: the value v rounded to a binary floating-point number with a
  64-bit significand, round-to-nearest-even (x87 extended precision). All
  turtle positions, steps, extents, the start point and the table
  directions are kept in this precision. Every operation below that is
  written ext(…) is rounded once.
- **f32(v)**: v rounded to IEEE single precision. The **scale** and the
  **aspect factor** are float32 values.
- **dbl(v)**: IEEE double. π below is the double nearest to π
  (3.141592653589793…).

Evidence: `bisect/prec-bisect.txt` measured the rounding point of segment
ends at 20 positions to 1e-9 pixel: the error is a pure scale error of
exactly the relative size of float32(390/1.1) versus 390/1.1
(−3.9125e-8), zero at the image centre. `bisect/origin-bisect.txt` measured
the start point of an arrowhead curve to 1.5e-16 pixel; only the
extended-precision direction table of §6.3 gives that value (double
directions are 2.0e-14 off). Double-precision positions, double directions,
correctly rounded double sines and other formula orders were all tried and
each fails some corpus images; the variant described here fails none (§9).
The arm64 build differs from the amd64 build exactly at such ties
(hypothesis: it has no extended precision and contracts a·b + c into one
rounding).

### 6.3 Directions (confirmed)

- **Aspect factor** A = f32(dbl(dbl(W / H) × 0.75)), with W × H the image
  size. A = 1 exactly for 4:3 images (400×300, 640×480, 800×600). Every
  horizontal component below is multiplied by A, so a drawing keeps its
  shape on a 4:3 screen: on 301×301 a square is 0.75 times as wide as high
  (`seg/seg-301x301-q4`), on 200×400 0.375 (`seg/seg-200x400-*`).
- **Direction table** for N directions: step = ext(dbl(2π) / N); for
  k = 0 … N−1: θk = ext(step × k), and
  dir(k) = ( ext(ext(cos θk) × A), ext(sin θk) ),
  where cos and sin are evaluated accurately (to at least 64 bits) at the
  extended-precision argument θk. F and G move by size × dir(k).
- **Degree heading** h (a double; `/ n` adds, `\ n` subtracts, in double):
  r = dbl(dbl(h × π) / 180); D and M move by size × (ext(c × A), s) with
  c = dbl(cos r), s = dbl(sin r). (The order `h × (π/180)` gives the same
  results on all our cases; an extended-precision r fails two of them,
  `deg2/deg2-e40`, `-e41`.)

### 6.4 Extent, scale and start point (confirmed)

Pass 1 moves the turtle from (0, 0) with size 1: each `F G D M` sets
x ← ext(x + ext(size × dx)), y ← ext(y + ext(size × dy)), and `@` sets
size ← ext(size × v) or ext(size / v) (§5.1). Let xmin, xmax, ymin, ymax be
the extremes of all positions, bw = ext(xmax − xmin), bh = ext(ymax − ymin).

    S  = f32( min( ext((W − 10) / bw) if bw > 0,
                   ext((H − 6) / bh)  if bh > 0 ) )
    x0 = ext( (W − 1)/2 − ext( ext( ext(xmin + xmax) / 2 ) × S ) )
    y0 = ext( (H − 1)/2 − ext( ext( ext(ymin + ymax) / 2 ) × S ) )

So the drawing's extent is centred on the image centre ((W−1)/2, (H−1)/2)
and fills W − 10 pixels across or H − 6 pixels down, whichever is reached
first (single horizontal segment on 400×300: ends at x = 4.5 and 394.5,
pixels 5–395, `misc/geo-g1`; vertical: 2.5 and 296.5, `seg/seg-400x300-p4`).
Margins are fixed pixel counts, not fractions (`seg/seg-800x600-h`: 5–795).

- Several algebraically equal ways of writing x0 and y0 (for example
  ext((W−1) − ext((xmin+xmax) × S)) / 2) give the same pixels on all our
  cases; the order above is one that works.
- If bw = bh = 0 every point is the centre: `@0F` draws the single pixel
  (200, 150) on 400×300 (`sem/s-zero1`). S is then irrelevant.
- For W < 10 the first candidate is negative and wins the minimum: the
  drawing is mirrored left-right and still matches (`ext/x-*-8x8`,
  `x-*-12x9`).
- The window corners play no part (§3).

### 6.5 Drawing (confirmed)

Pass 2 starts at (x0, y0) with size S, direction index 0, degree heading 0,
colour **15**, `!` off. Every move updates the position as in pass 1. For a
drawing command the segment from the old position (xa, ya) to the new one
(xb, yb) is drawn as follows:

1. Each end is rounded to a pixel: P(v) = ⌊ext(v + 1/2)⌋ (half up in
   extended precision), giving (Xa, Ya) and (Xb, Yb).
2. Let n = max(|Xb − Xa|, |Yb − Ya|). If n = 0, the single pixel is set.
   Otherwise one pixel is set for each step along the longer axis, from one
   end to the other (n + 1 pixels, both ends included): if |ΔX| ≥ |ΔY|,
   for each column X between Xa and Xb the row is
   ⌊Ya + (X − Xa)·ΔY/ΔX + 1/2⌋ computed exactly (rationally); otherwise,
   for each row Y the column is ⌊Xa + (Y − Ya)·ΔX/ΔY + 1/2⌋. Exact halves
   therefore go to the **larger** coordinate whichever way the line runs.
   This is a Bresenham line whose ties do not depend on the direction.
   Evidence: every segment of the single-segment cases is exactly such a
   line between its two extreme pixels (`seg`). The 312 images of `seg`,
   `rich` and `arr` contain 7,016 tie pixels; with "ties to the larger
   coordinate" the model reproduces all 312 images, with "towards the
   start", "towards the end" or "to the smaller coordinate" 94, 104 and 137
   of them differ.
3. The pixels get the current colour, overwriting what is there (colour 0
   too). Later segments overwrite earlier ones (`sem/s-overw`).

Pixels outside the image are not drawn (hypothesis: nothing in the corpus
needed clipping except possibly the tiny sizes, which match).

- **Background**: colour 0 everywhere else (all cases).
- **Default line colour 15**; with the default palette 15 is white
  (255, 255, 255), 1 (0, 0, 170), 2 (0, 170, 0), 3 (0, 170, 170),
  4 (170, 0, 0) (`ext/x-plant-o3-defmap`). Colour indices are written as
  computed (the uniq palette shows every index). With the default palette
  the saver may store the lowest index of an equal colour instead
  ([types.md](types.md) §1.3).
- `inside=`, `outside=`, `fillcolor` and similar options do not affect the
  colours (§3).

### 6.6 Resolution dependence (confirmed)

The image depends on W and H only through §6.3–6.5: the margins are fixed
in pixels, the scale fills W − 10 or H − 6, and the aspect factor stretches
x for non-4:3 sizes. Verified pixel-exact on 400×300 (`viewwindows=2`),
800×600 (the batch default, `ext/x-plant-o3-defsize`), 640×480, 333×211,
301×301, 200×400, 401×301, 555×333, 777×555, 500×250, 300×200, 267×200,
533×400, 320×240, 200×150, 50×50, 20×20, 12×9 and 8×8 (`seg`, `rich`,
`arr`, `ext`, `misc/vw*`). Sizes above 800×600 fall back to 800×600 with
a message (§2.6).

### 6.7 Pseudo-code

    A    = f32(dbl(dbl(W/H) * 0.75))
    tab  = direction table for N (§6.3)
    s1   = rewrite(axiom, order, protect args of @ / \)
    ext  = run(s1, pos=(0,0), size=1, colours ignored)          # pass 1
    S, x0, y0 from ext (§6.4)
    s2   = rewrite(axiom, order, protect args of @ / \ C < >)
    run(s2, pos=(x0,y0), size=S, colour=15, plot segments)        # pass 2

    run: k=0, h=0.0, rev=false, stack=[]
      F/G: (dx,dy)=tab[k];  D/M: (dx,dy)=degree direction(h)
           new = (ext(x+ext(size*dx)), ext(y+ext(size*dy)));  F/D: line(old,new,colour)
      +: k=(k-1) mod N (k+1 if rev)   -: the opposite   |: k=(k+N div 2) mod N
      /n: h+=n (h-=n if rev)   \n: the opposite   !: rev=not rev
      @[IQ]n: v=n (sqrt if Q); size = ext(size/v) if I (0 if v=0) else ext(size*v)
      Cn, <n, >n: §5.1 (pass 2 only)
      [: push (x,y,k,h,colour,rev,size)   ]: pop, or stop if empty

`tools/blackbox/lsystem/lsysmodel.py` is this pseudo-code in full.

---

## 7. Interrupt, save and reload (confirmed)

Evidence: `interrupt/` (`run-interrupts.sh`; order 12 of `sem.l` entry
`kobig`, which takes 22–40 s interactively under qemu; `results.txt`
summarises status, time, blocks and how the pixels compare with the batch
render `sem/s-kobig-12`).

- Pressing `s` during the calculation saves the image drawn so far with
  **status 3** at `fractint001` offset 202 (a value not seen for other
  types, see gif-fractint-blocks §2.1) and **no `fractint002`** block; the
  time field holds the time so far, the extension length stays 603
  (`int-8`: nothing drawn yet, `int-15`: part of the curve). An image that
  finished before the key press is saved normally with status 4 (`int-30`).
- The partial image is a subset of the finished one (no extra pixels); it
  shows the curve from its start (hypothesis: exactly the segments of
  pass 2 drawn so far).
- **There is no resume.** Loading the save:
  - batch `filename=in.gif`: re-saved unchanged with status −1
    (`ld-batch`), as for other types;
  - interactive `filename=in.gif` without a key: unchanged, status −1
    (`ld-none`);
  - interactive `filename=in.gif lfile=sem.l` and Enter: the whole image is
    **recalculated from scratch** — marker pixels painted into the file are
    gone, the result equals the batch render and the time starts again
    (`ld-ret`, `ld-ret-mark`);
  - the same without `lfile=`: the entry is looked for in the default file,
    is not found (a message box), and after Escape a blank image is saved
    with status 4 (`ld-ret-nolf`). The `.l` file name is not stored in the
    GIF (§3), so it must be given again;
  - the in-program load command `r` shows the saved pixels (markers kept)
    and calculates nothing, status 3 kept (`ld-r-mark`).
- A writer that wants XFractint to redraw an L-system image needs only
  `fractint001` (type 107, param 1) and `fractint003` (entry name), with the
  file given on the command line.

---

## 8. Floating-point vs integer math (confirmed)

`float=yes` and `float=no` give byte-identical pixels for every case tried:
the nine shapes of `misc/geo-*` and `misc/geo-*-fl`, five multi-segment
curves at orders 3–8 (`misc/fl-*`), and all 66 rounding probes of `prec`
(compared during exploration, not kept). The flag is not stored
(gif-fractint-blocks §5.6). We found no integer (fixed-point) L-system path:
the precision probes show extended-precision floating point in both
settings.

---

## 9. Verification performed

- `tools/blackbox/lsystem/verify.py` renders every image case of the corpus
  with `lsysmodel.py` (this spec in code) and compares all pixels:
  **1,128 cases identical, 0 different** (orders ≤ 8); a second run over
  `sem` and `ext` with orders up to 200 (including order 10 of the
  four-segment Koch rule, about 1M segments): **114 identical, 0
  different** (`verify.txt`).
  Skipped: cases with a stop message or no image, the file look-up cases
  (another file was read), and orders 12 and up (too slow for the model).
- The corpus covers all angles 3–50 in single segments, 19 image sizes,
  orders 0–10, Koch, dragon, Lévy, Hilbert, Sierpinski arrowhead, bushes
  and coloured plants with branches, every command, degree turns summed
  from many small steps, colour arithmetic, and pixel ties of every kind.
- `bisect/` holds the two precision probes of §6.2.

---

## 10. Unknowns

- Which file the search of §2.1 prefers when several have the entry, and
  what the default L-system file is.
- Limits on the number of entries, rules and joined rule length (§2.7);
  whether long entry names are compared on 18 characters.
- `{`, `}` and `=` as rule symbols; `lfile=` with a directory part.
- Exactly how the line-length split interacts with comments (a `;` in the
  first 255 characters hides the rest of the physical line? not tested).
- Whether the partial drawing of an interrupted save is exactly a prefix of
  pass 2 (§7).
- Clipping of pixels outside the image (§6.5); whether the scale for an
  all-zero extent matters anywhere.
- XFractint's own L-system file and its in-program L-system parameter
  screens were not examined (clean-room rule).

---

## 11. Evidence index

| Topic | Cases |
|---|---|
| File look-up, 8.3, search | `fmt3/f-lk-*`, `fmt2/f-nofile-solo`, `f-noext`, `f-upperfile`, `f-nolfile` |
| Entry structure | `fmt/f-*`, `fmt3/f-name-*`, `f-noclose-*`, `f-after-open-entry`, `lhs/gl1`, `gl2` |
| Body syntax, rules, messages | `fmt/f-*`, `fmt2/f-*`, `lhs/h*`, `misc/dup-*` |
| Limits | `fmt2/f-axlen*`, `f-rulelen*`, `f-nrules*`, `f-nlines*`, `f-name*`, `fmt3/f-lname*` |
| Commands | `chars/*`, `args/*`, `sem/*`, `mv/*`, `deg/*`, `deg2/*`, `misc/sl-*` |
| Rewriting | `fmt/f-simul*`, `f-rule-*`, `ext/x-prot*`, `ext/x-rule*` |
| Geometry and rounding | `seg/*`, `rich/*`, `arr/*`, `prec/*`, `bisect/*` |
| Sizes and aspect | `seg/*`, `ext/x-*-<size>`, `misc/vw*` |
| Stored blocks | every `results.txt` line (`t=`, `p1=`, `corners=`, `st=`, `003=`) |
| Interrupt/reload | `interrupt/*` |
| float | `misc/geo-*`, `misc/fl-*` |
