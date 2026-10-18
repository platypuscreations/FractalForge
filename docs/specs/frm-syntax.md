<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Formula files (`.frm`): syntax, file structure and storage

This spec says how XFractint reads a formula file, which texts it accepts or
rejects (and with which error), and how a `type=formula` image is stored in
GIF and `.par` files. It is precise enough to write a tokenizer and parser
from. What the operators, functions and variables **compute**, how
iterations are counted and coloured, and what symmetry qualifiers copy is
the subject of [frm-semantics.md](frm-semantics.md). Where the two overlap,
this spec owns the syntax.

Everything here was found black-box: we wrote our own formula files, rendered
them with XFractint and looked at the saved pixels, the saved GIF blocks and
the messages. No XFractint source, documentation or bundled formula file was
used. Labels: **confirmed** (seen directly, cited), **hypothesis** (fits
everything we saw, not proven), **unknown**.

---

## 0. Method and evidence

- Program: Debian `xfractint 20.4.10-5` (amd64 under qemu), batch mode, via
  `tools/blackbox/frm-syntax/run-case.sh`, which runs
  `xfractint batch=yes type=formula passes=1 periodicity=0 viewwindows=2
  map=uniq.map ARGS savename=ff.gif` in a fresh directory holding all files of
  `tools/blackbox/frm-syntax/inputs/` (our own test files).
- Evidence lives in `corpus/xfractint/frm-syntax/<round>/`. A case named
  `p-neg2` is in folder `p`, `r3-store-s1` in folder `r3`, and so on (the
  prefix before the first `-` is the folder). Its arguments are in
  `cases.txt`, its outcome is the line of the same name in
  `<round>/results.txt` (pixel hash, colour counts, `fractint003` contents,
  the stop message and the console output); the image and message files sit
  next to it. The entry `X` used by a case is in the `.frm` file that the
  case's `formulafile=` names.
- **The probe entry** (`gen-probe.py`). Most expression tests use

  ```
  NAME {
    z = 0, <statements that set k>:
    z = z + 1
    real(z) <= real(k)
  }
  ```

  with `maxiter=1000`. Every pixel then gets the same colour index
  **floor(k) + 1** for 0 ≤ k < 254 (`cal-k0` … `cal-k200`, `cal-k2p5` → 3),
  **1** for k < 1 (`cal-km1`), wrapping modulo 255 above that (`cal-k300` →
  46) and 0 when k ≥ maxiter (`p-inside`). So "colour 106" below means the
  expression gave k = 105. NaN also gives colour 1 (`e-er22`).
- **Batch messages** go to the file `stopmsg.txt` in the working directory;
  XFractint goes on and still saves an image (§7).
- An image whose every pixel has colour 1 (pixel hash `740c5da2`) is what a
  formula error produces; such cases are told apart by their `stopmsg.txt`.

---

## 1. File structure

### 1.1 Bytes and lines (confirmed)

- Line ends: LF and CR LF both work (`s-crlf`, `s-crlf2`: a CR LF file).
- A Ctrl-Z byte (0x1A) ends the file: an entry after it is not found
  (`t-ctrlz-after`), the one before it is (`t-ctrlz-before`).
- Bytes 0x80–0xFF in comments and in text outside entries are harmless
  (`t-latin1-comment`). Non-ASCII entry names: **unknown** (`t-latin1-name`
  could not pass the byte through the command line).
- Tabs behave like spaces (`s-comments` has a tab-indented statement;
  `t-misc-c10a` has a tab before `{`).
- Line length: no limit found. A 5,000-character statement line
  (`r3-longent-lg3`, mostly spaces), a 3,000-character comment
  (`t-longcomment3000`) and a 3,000-character line of free text
  (`t-longtext`) all work. Entry length in lines: 2,000 comment lines
  (`r3-longent-lg1`) and 3,000 blank lines (`r3-longent-lg2`) before the
  statements work. Limits on the amount of *code* are in §1.7.

### 1.2 Entries (confirmed)

An entry is

    NAME [QUALIFIER] [other text] {  BODY  }

- **The header must be on one line**: the `{` has to be on the same line as
  the name. With `{` on the next line (`s-brace-c3`) or after a blank line
  (`s-brace-c3c`) the entry is not found. Blanks before the name
  (`t-leadspace`) and between name and `{` (`s-brace-c3d`) are fine; none is
  needed (`s-brace-c3b`, `c3b{`).
- **The name** is the first word of the line. It ends at a blank, `(`, `[`
  or `{`: `my name {` is found as `my` and not as `my name`
  (`s-hdr2-my`, `s-hdr2-myname`); `x[y {` and `w[1] {` are found as `x`
  and `w` only (`r6-brk-*`, `r6-brk2-*`). Every other printable character
  we tried is part of the name: `- _ . # ! + @ / $ , : " } ) \ | =` and a
  leading digit (`s-char0`…`s-char9`, `t-char0`, `t-char2`…`t-char7`,
  `r6-eq-full`, `r6-eq-short`). A `;` starts a comment (§1.4), so `a;b {`
  defines nothing usable (`t-char1`).
- **Qualifier**: an optional `( … )` right after the name (blanks allowed
  before it, `s-hdr-c4b`). §1.5 lists the recognised words. A `(` without
  `)` on the header line is an error: stop message `Didn't find matching ')'
  in symmetry declaration`, no line number (`s-hdr-c4n`, `r6-paren`).
- **Other text** between the name or qualifier and `{` is ignored:
  `c4f junk words {`, `c4g[x=1] {`, `c4h(XAXIS)[x] {`, `t(XAXIS) junk {`,
  `s junk(XAXIS) {` are all found by their name (`s-hdr-c4f`, `-c4g`,
  `-c4h`, `r6-symjunk`, `r6-junksym`). Whether a qualifier after other text
  (`s junk(XAXIS)`) still takes effect is **unknown** (only uniform probes).
- **Body**: everything after the `{` up to the **first** `}` that is not in a
  comment. A `}` in the middle ends the entry; the statements after it are
  ignored (`t-brace-c5e`: body `z = 0, k = 51:` only, so every pixel is
  inside). `}` may follow the last statement on the same line
  (`s-line-c5b`), the whole entry may be on one line (`s-line-c5`,
  `t-misc-c10e`), statements may follow `{` on the header line
  (`s-line-c5c`), and another entry may start on the line of the `}`
  (`s-line-c5d` / `s-line-c5e`: `} c5e {`).
- A `{` inside the body (outside comments) breaks the lookup:
  `c15c { … { z = z + 1 } … }` is not found (`s-brace-in-c15c`), and in
  `t-brace-c5d` the entry `c5d` (with a stray `{` line) was rendered with the
  body of a later entry, `c5f` (colour 54 = `c5f`'s k). A `{` inside the
  parentheses of the qualifier also makes the entry unfindable
  (`t-brace-c5g`). Braces inside comments are harmless (`s-brace-in-c15`:
  `; }`, `s-brace-in-c15b`: `; {`). FractalForge should **reject** a `{` in
  a body.
- An entry without a closing `}` is not found, whether it is the last one in
  the file (`s-noclose-eof`) or followed by others (`s-noclose-a`); the
  entries after it are found normally (`s-noclose-b`, `s-noclose-c`), the
  ones before too (`s-noclose-eof-first`).
- **Text outside entries** is ignored, before the first entry, between
  entries and after the last (`s-text-before`, `s-text-between`, s02.frm),
  as long as it does not look like a header (a word followed by `{` on the
  same line starts an entry — **hypothesis**, consistent with all the
  above). A name line without `{` is ordinary text (`s-hdr2-c28c` not found,
  the next entry `c28d` is fine). A line `{` with no name before it does not
  disturb the next entry (`s-hdr2-c28`).

### 1.3 Entry lookup (confirmed)

- By `formulaname=` on the command line or in a `.par` entry (§8.2).
- **Case-insensitive**: `Mixed`, `UPPER` are found as `mixed`, `upper`,
  `UPPER` (`s-case-mixed`, `s-case-upper`, `s-case-upper2`).
- **Duplicates**: the first entry of that name in the file wins (`s-dup`,
  and `s-case-dupU` with `DUP`: colour 71 = the first `dup`).
- **No prefix matching**: the first 15 characters of a 16–60 character name
  do not find it (`s-len16-cut15` … `s-len60-cut15`).
- **Length**: `formulaname=` accepts at most **18 characters**. 15–18 work
  (`s-len15` … `s-len18`, `r3-store-len18`); 20 or more are rejected as a bad
  argument (`Oops. I couldn't understand the argument:`, no image, exit
  status 1; `s-len20` … `s-len60`), also inside a `.par` entry
  (`r4-par-img7`). Longer names are legal in a file but cannot be selected.
- **Missing entry**: stop message `'NAME' file entry item not found` and
  `Could not open file where formula located`; an image is still saved
  (all colour 1) with NAME in `fractint003` (`a-missing`).
- **Missing file**: `formulafile=zz.frm` (no such file) still found entries
  that exist in other `.frm` files of the working directory (`a-nofile`,
  `t-nofile-c1`, `r5-fallback-frm`) but not an entry that exists only in a
  `.par` file there (`r5-fallback-par`). **Hypothesis:** when the named file
  is missing, XFractint searches the `*.frm` files of the current directory.
  Without any `formulafile=` the entry `c1` of `s01.frm` was not found
  (`t-nofileparam-c1`; hypothesis: the default is a file of XFractint's own).
  `formulafile=s01` (no extension) and `formulafile=S01.FRM` found `c1`
  (`t-noext`, `t-upperfile`), but that may be the same fallback, so whether
  `.frm` is added or file names are case-folded is **unknown**. FractalForge
  need not copy the fallback.
- `formulafile=` without `formulaname=` renders an all-colour-1 image with an
  empty name in `fractint003` and no message (`t-noname`).

### 1.4 Comments (confirmed)

- `;` starts a comment that runs to the end of the line, anywhere: on its
  own line, after a statement, after the `:`, after `{` on the header line,
  after `}` (`s-comments`), after a continuation backslash (`t-cont-c6a`).
- A `;` before the `{` of a header hides the brace (`t-char1`).
- Nothing else is a comment: `#`, `'` (error 28), `//` (error 0), `!`
  (error 4) at the start of a body line are errors (`s-cmt-h`, `s-cmt-ap`,
  `s-cmt-sl`, `s-cmt-bang`), and `rem remark` is just the expression
  `remremark`, a variable (§1.6; `s-cmt-rem` renders normally).
- Blank lines anywhere are ignored (`s-comments`, `r3-longent-lg2`).

### 1.5 Symmetry qualifiers: syntax (confirmed)

The word in the parentheses, compared without case and after removing all
blanks (§1.6), is one of

    XAXIS  YAXIS  XYAXIS  ORIGIN  PI_SYM
    XAXIS_NOPARM  YAXIS_NOPARM  XYAXIS_NOPARM  ORIGIN_NOPARM  PI_SYM_NOPARM
    XAXIS_NOREAL  XAXIS_NOIMAG

(`y-01-xaxis` … `y-12-pi_sym_noparm` all change the image; `y-15-xaxis`
lowercase and `y-17-x-axis` written `(X AXIS)` give exactly the `XAXIS`
image). Anything else — `NOSYM`, `NONE`, `BOGUS`, `()`, `(xaxis junk)` —
is accepted silently and copies nothing (`y-13-nosym`, `y-16-none`,
`y-14-bogus`, `s-hdr-c4m`, `y-18-xaxis-junk`, all pixel-identical to the
plain entry `y-00-plain`). What each word copies, and the parameter
conditions of the `_NO…` forms, are in frm-semantics.md §6; our
`symcheck.py` counts (`y` folder) agree: XAXIS mirrors rows, YAXIS columns,
XYAXIS and ORIGIN as named; `_NOPARM` copies only with params 0, `NOREAL`
only when P1's real part is 0, `NOIMAG` only when its imaginary part is 0.
The qualifier is **not stored** (§8.1), and `symmetry=` on the command line
overrides it (`y-01-xaxis-symnone`, `y-01-xaxis-symy`).

### 1.6 Blanks are removed before tokenizing (confirmed)

Within a statement every space and tab is deleted before the statement is
split into tokens. Consequences:

- `1 0` is the number 10 (`p-lit13`), `< =` is `<=`, `& &` is `&&`, `= =` is
  `==` (`p-ws1`, `p-ws2`, `p-ws6`).
- `else if (1)` is `elseif(1)` (`i-if16`), `end if` is `endif` (`i-if18`).
- `if 1` becomes the variable name `if1` (`i-if6`: the following `endif` has
  no matching `if`).
- `z = 0 k = 189` becomes `z=0k=189`, and `0k` is an invalid constant
  (`s-sep-space`, error 27). Blanks therefore never separate statements.
- Error messages echo the statement with its blanks removed (every
  `stopmsg.txt`).

### 1.7 Size limits (confirmed, with crashes)

- Identifiers and number literals: at most **32 characters** (§3.1, §3.2).
- Jumps: an entry with 200 or more `if`/`elseif`/`else`/`endif` keywords is
  rejected with error 23 `Formula has too many jump commands`, reported at
  the entry's last statement line (`t-nest100`: 100 nested `if` + 100
  `endif`; `r3-jumps-if100`: 100 `if`/`endif` pairs; `r3-jumps-ife67`: 201
  keywords). 198 keywords passed this check (`r3-jumps-ife66`, which then
  crashed, see below); 100 keywords, 50 levels deep, worked (`t-nest50`).
  **Hypothesis:** the limit is 199 keywords.
- **Amount of code: no error, but crashes.** XFractint does not check it:
  - 50 statements `k = k + 1` work (`t-stmts50`); 100 render correctly and
    then abort on exit with `munmap_chunk(): invalid pointer`
    (`t-stmts100`); 150–300 crash (segmentation fault, no image:
    `t-stmts150` … `t-stmts250`, `s-long-m300`).
  - One statement `k = 0 + 1 + … + 1`: 200 terms work (`t-terms200`), 250
    and 300 give the right image and then abort (`t-terms250`,
    `t-terms300`), 400 crash (`t-terms400`); 495 `+ 0` terms crash
    (`s-long-l2000`), 145 work (`s-long-l600`).
  - `if` blocks: 49–51 `if (1) k = k + 1 endif` groups abort on exit
    (`r3-jumps-if49` … `-if51`), 99 crash (`r3-jumps-if99`).
  - 2,000 statements `k = k + 0.025` silently gave an all-colour-1 image with
    no message (`s-long-m2000`).

  **Hypothesis:** internal tables of a few hundred operations overflow.
  FractalForge should not crash: either accept large formulas or reject
  them with an error. Formulas below about 50 statements / 200 operators are
  safe in XFractint.

---

## 2. Entry body

### 2.1 Statements and separators (confirmed)

- Statements are separated by **newlines** and by **commas** (outside a
  complex literal, §3.3). Both are equivalent: `if (1), k = 105, endif`
  works like the three-line form (`i-if5`), and whole bodies can be written
  on one line with commas (`s-line-c5`).
- **Empty statements are allowed**: `z = 0,, k = 170,:`, a leading comma,
  a trailing comma, a line holding only `,` (`s-empty-c17`, `-c17b`,
  `-c17c`).
- A **newline always ends a statement**, even inside parentheses
  (`s-split-c10d`: `k = (100` / `+ 4)` gives errors 3 and 0) or after an
  operator (`s-split-c10b`: `k = 100 +` / `2` gives error 0). Use a
  continuation (§2.2).
- There is no other separator. In particular a blank is not one (§1.6), and
  `;` starts a comment.

### 2.2 Line continuation (confirmed)

- A `\` that is the last thing on a line (only blanks or a comment may
  follow) joins the next line to this one: `k = 100 + \` / `1` (`s-split-c10`),
  `k = 100 \` / `+ 3` (`s-split-c10c`), trailing blanks (`t-cont-c6c`), a
  comment after it (`t-cont-c6a`, `r3-bslash-bs2`), several in a row
  (`t-cont-c6d`). The join happens before blanks are removed, so it may
  split an identifier or a number: `k\` / `k` is `kk` (`t-cont-c6e`), `1\` /
  `08` is `108` (`t-cont-c6h`), and the `:` may be on the next line
  (`t-cont-c6f`).
- A `\` elsewhere in a line is simply **dropped**: `k = 100 \ + 2` is
  `k=100+2` (`t-cont-c6b`), `10\0` is `100` (`r3-bslash-bs3`), and such a
  backslash does not join lines (`r3-bslash-bs1`: the next line `+ 3` is a
  separate, erroneous statement).

### 2.3 Initialisation and iteration sections (confirmed)

- A single `:` divides the body into the **initialisation** statements (run
  once per pixel, before iterating) and the **iteration** statements (run
  once per iteration). See frm-semantics.md §1.1 for the run-time model.
- The `:` is a statement separator itself: it may end a line, sit in the
  middle of a line with iteration statements after it (`s-sep-sameln`,
  `s-line-c5`), follow an empty statement (`k = 170,:`, `s-empty-c17`), or
  start the body (`s-sep-emptyin`: empty initialisation).
- **At most one `:`**: a second one is error 34 `Only one ":" permitted in a
  formula` (`s-sep-two`, `s-sep-colon2`).
- **No `:`**: every statement is an iteration statement and nothing is
  initialised. `z = z + 1` / `real(z) <= 9` without `:` gave colour 10 at a
  single pixel and colour 1 everywhere else, because `z` carries over from
  pixel to pixel (`s-sep-none`, `s-sep-none2`, `s-sep-onlytest`; carrying is
  frm-semantics.md §1.5). It is not an error.
- **Empty iteration section** (nothing after `:`): no error; the test is
  never false, so every pixel reaches maxiter (`s-sep-emptyit`; also
  `t-brace-c5e`).
- **No statements at all** (`{}`, only comments, only `:` or only `,`):
  stop message `Formula has no executable instructions` (`s-sep-empty`,
  `s-hdr2-c28b`, `t-misc-c10b`, `-c10c`, `-c10d`).
- An `if` block must lie entirely in one section: `if (1)` / `:` / `endif`
  gives error 20 and error 18 (`t-ifit-it4`); `if`/`endif` wholly before the
  `:` is fine (`r4-it-ie6`).

### 2.4 The continue/bailout test (confirmed)

The value that decides whether iterating continues is the value of the
**last statement executed** in the iteration section on that iteration
(true = continue, by frm-semantics.md §1.2):

- A comparison earlier in the section is ignored when another statement
  follows: `real(z) <= real(k)` / `z = z` iterates forever (`s-sep-mid`), and
  `real(z) <= 3` before `z = z + 1` / `real(z) <= real(k)` has no effect
  (`s-sep-mid2`).
- An assignment's value is the assigned value: `z = z + 1` last iterates
  forever (`s-sep-asg`), `w = 0` last stops at once (`s-sep-asg0`),
  `k = real(z) <= 9` last stops after 9 iterations (`s-sep-asgk`).
- **`if` conditions count as executed statements; `else` and `endif` do
  not.** With `if (cond)` / `t = 0` / `else` / `t = 1` / `endif` last, the
  value is that of the branch's `t = …` (`t-ifit-it1`). With
  `if (cond)` / `t = 5` / `endif` last (no `else`), a false condition is
  itself the last value, so the pixel stops (`t-ifit-it5`, `t-ifit-it7`,
  `t-ifit-it3`: a false `if (0)` after the real test stops every pixel at
  once; `r4-it-ie5`). A test inside a taken branch works as the last value
  (`t-ifit-it2`). **Hypothesis:** `elseif` conditions count the same way
  (not tested as last statements).
- Comparisons may also appear in the initialisation section; they do
  nothing there (`s-sep-testinit`).

### 2.5 Statement kinds

- **Expression** (confirmed): any expression is a statement (`s-cmt-rem`,
  `s-sep-testinit`, all tests).
- **Assignment** (confirmed): `variable = expression`, right-associative
  chains allowed: `a = b = 7` sets both (`p-asg2`). The left side must be a
  single variable name; anything else is error 25 `Only variables are
  allowed to left of assignment` (`e-er2` `3 = k`, `e-er9` `k = 5 = 6`,
  `q-q3` `k = 100 + a = 5`). An assignment cannot be nested in an
  expression: `k = 100 + (a = 5)` (`p-asg1`), `a = 5 + (b = 2)` (`q-q4`)
  and `if (j = 1)` (`i-if20`) are error 25. A statement that is only `= 5`
  crashed XFractint (`e-er1`, segmentation fault).
- **`if`, `elseif`, `else`, `endif`** (§5).

---

## 3. Tokens

After comments, continuations and blanks are removed (§1.4, §2.2, §1.6),
a statement is a string of the tokens below. Characters outside these
tokens are error 28 `This character not supported by parser`: `%`, `#`,
`'`, `$`, `~`, `?`, `[`, `]`, `` ` ``, `"`, `@` (`p-pct`, `s-cmt-h`,
`s-cmt-ap`, `i-id9`, `e-er15` … `e-er20`).

### 3.1 Identifiers (confirmed)

- Letters, digits and `_`, not starting with a digit: `abc_1`, `_a`,
  `a1b2` (`i-id1`, `i-id2`, `i-id3`). A word starting with a digit is read as
  a number and is error 27 (`s-sep-space`: `0k`).
- **Case-insensitive**: `Ab`/`aB`, `a`/`A`, `K`/`k` are the same variable
  (`i-id4`, `i-id14`, `i-id13`); built-in names too (`v-v6` `PIXEL`, `v-v8`
  `P1`, `v-v17` `Pi`, `v-v18` `E`, `v-v7` `FN1`, `b-call6` `SQR`).
- **Length ≤ 32**: two 32-character names that differ only in the last
  character are different variables (`i-id6`); 33 or more characters is
  error 33 `Variable or constant exceeds 32 character limit` (`w-w3` 33,
  `i-id5` 34, `i-id15` 78).
- A `.` in a name is error 26 `Illegal variable name` (`i-id8` `a.b`).
- **Reserved**: `if`, `elseif`, `else`, `endif` (§5; as variables they give
  errors 16 or 15: `b-aif`, `b-aelseif`, `b-aelse`, `b-aendif`,
  `b-rif` …), and every function name (§6.2; error 31 `Can't use function
  name as variable`: `b-asin`, `b-areal`, `b-afn1`, `b-rsin`, `e-er23`).
  Every other name is a variable; names XFractint does not predefine start
  at 0 (`i-id10`, `i-id11`, `a-undef`).

### 3.2 Real number literals (confirmed)

- Digits with an optional decimal point: `5.`, `.5`, `1.`, `007`, `3.75`,
  `12345678901234567890` (`p-lit5`, `p-lit4`, `p-lit15`, `p-lit14`,
  `p-lit17`, `p-lit16`).
- Optional exponent `e` or `E` followed by digits: `1e2`, `1E2`, `1.5e1`,
  `1e02`, `1.e1`, `.5e1` (`p-lit1`, `p-lit2`, `p-lit3`, `u-e9`, `u-e7`,
  `u-e8`).
- Error 27 `Invalid constant expression`: `0x10`, `1d2`, `2.5.5`, `1e`,
  `1e2.5`, `1e2e3`, `.`, `1..2` (`p-lit8`, `p-lit9`, `p-lit10`, `p-lit11`,
  `u-e5`, `p-lit12`, `u-e6`, `u-e16`, `u-e17`).
- Length ≤ 32 characters (error 33 for 33: `w-w4`, `w-w8`; 32 works:
  `w-w7`).
- Overflow gives an infinite value without error (`p-lit19` `1e400` →
  inside).
- **Signed exponents are misread (quirk).** In `1e+2` the number token ends
  before the sign, but its value is taken from the whole text including the
  signed exponent; then the sign and digits are parsed again as an ordinary
  `+ 2`. So `1e+2` is 100 + 2 = 102 (`p-lit6`, `u-e10`), `2e+1` is 21
  (`u-e2`), `3e+1 + 0` is 31 (`u-e11`), `1e+1 * 2` is 10 + 1·2 = 12 (`u-e13`),
  `1e+2 * 0 + 50` is 150 (`u-e1`), `2.5e+1` is 26 (`u-e15`),
  `1e+2 + 1e+2 - 100` is 104 (`u-e19`); `5e-1 * 100` is 0.5 − 100
  (`u-e3`), `100 + 5e-1` is 99.5 (`u-e4`), `1e-1 * 2 + 50` is 48.1
  (`u-e14`), `1e-400` is 0 − 400 (`p-lit18`); also inside complex literals,
  `(1e+1, 0)` has real part 11 (`q-q8`). `1e-` at the end of a statement
  gave colour 1 without a message (`u-e18`; **unknown**).
  FractalForge must reproduce this to render such formulas the same way;
  a warning is advisable.

### 3.3 Complex literals (confirmed)

- `(a,b)` where `a` and `b` are real literals, each optionally preceded by
  `-`, is a single constant token a + bi: `imag((3,4))` = 4 (`p-cplx1`),
  `(-3,4)` and `(3,-4)` (`p-cplx2`), `(1e1,0)` (`p-cplx3`), blanks allowed
  (`p-cplx6`, `e-er26`), usable in arithmetic (`p-cplx7`).
- Only literals: `(a, 0)` with a variable and `(1 + 2, 0)` are not complex
  literals. The comma then acts as a statement separator, giving error 3
  `Need more ')'` for the part before it and error 2 `')' needs a matching
  '('` for the part after (`p-cplx4`, `p-cplx5`); `(1,2,3)` likewise
  (`e-er7`). **Hypothesis:** the tokenizer recognises `(`, signed literal,
  `,`, signed literal, `)` as one token before statements are split at
  commas.
- A function name followed directly by a complex literal is a call with that
  argument: `sqr(1, 2)` is `sqr((1,2))` = (−3, 4) (`b-call2`, `b-call3`).

### 3.4 Operators and punctuation (confirmed)

| Token | Use | Evidence |
|---|---|---|
| `+` `-` `*` `/` | binary arithmetic | `p-mul`, `p-sub3`, `p-div3`, `q-q2` |
| `^` | binary power | `p-pow2b`, `p-powmul` |
| `-` | unary minus (also repeated: `- - 5`) | `p-mulneg`, `p-uneg2`, `p-uneg3` |
| `\|` … `\|` | modulus bars around an expression | `p-modsq`, `p-modsum`, `w-w2` |
| `<` `<=` `>` `>=` `==` `!=` | comparisons | `p-cmp1`, `p-cmp5`, `p-cmp6`, `p-cmp7` |
| `&&` `\|\|` | logical and / or | `p-and1`, `p-and4` |
| `=` | assignment (§2.5) | `p-asg2` |
| `(` `)` | grouping, function call, complex literal | |
| `,` | statement separator; inside a complex literal | §2.1, §3.3 |
| `:` | section separator | §2.3 |

Rejected:

| Text | Error | Evidence |
|---|---|---|
| unary `+` (`100 + +5`) | 0 Should be an Argument | `p-uplus` |
| `**`, `^^`, `<<`, `>>`, `<>`, `1 + + 2`, `+*` | 0 | `p-star2`, `e-er21`, `e-er24`, `e-er25`, `p-ws5`, `e-er11`, `e-er14` |
| `=<`, `=>` (read as assignments) | 25 | `p-ws3`, `p-ws4` |
| `!` (also `!0`), `&` | 4 Undefined Operator | `p-not1`, `p-and7` |
| single `\|` between operands (`1 \| 1`) | 1 Should be an Operator | `p-or7` |
| `%` and the characters listed above | 28 | `p-pct`, `e-er15` … |
| `x^-1`, `x^-(1)`, `x ^ - 1` | 32 Negative exponent must be enclosed in parens | `p-powneg`, `e-er28`, `e-er29`, `e-er37` |
| juxtaposition: `2(3)`, `(1)(2)`, `sqr(1)(2)` | 1 | `p-imul`, `e-er32`, `e-er33`, `e-er31` |

Modulus bars: inside an open `|…|`, the next `|` at the same parenthesis
depth always closes it, so `|3 + |(0,2)||` is error 0 (`p-modnest`), while
a bar pair inside parentheses is fine: `|3 + (|(0,2)|)|` = 49
(`w-w1`), `|(|2| - 1)|` = 9 (`w-w10`), `sqr(|2|)` (`w-w6`). Two bars in a
row are always the `||` operator (`p-modnest2`, `w-w9`: `||2|-1|` is
error 0). An unclosed bar is error 30 `Unmatched modulus operator "|" in
this expression` (`e-er5`); `3|` is error 1 (`e-er6`).

### 3.5 Function calls (confirmed)

`NAME(expression)` with exactly one argument, `NAME` a function name (§6.2):
nested calls work (`b-call5`), `sin()` is error 0 (`b-call1`), a variable or
unknown name called as a function is error 5 `Undefined Function` (`a-unkfn`
`foo`, `e-er10` `fn9`, `e-er34` `pi`, `b-fp1`, `b-fz`, `b-fpixel`), a function
name without parentheses is error 31 (`e-er23`). Calls can only be followed
by an operator (`e-er31`).

---

## 4. Precedence and associativity (confirmed)

From highest to lowest. Each row cites a probe whose result differs between
the possible parses.

| Level | Operators | Assoc. | Evidence (value obtained; alternative parse) |
|---|---|---|---|
| 1 | literal, complex literal, variable, call, `( … )`, `\| … \|` | — | `q-q15` (20 nested parentheses) |
| 2 | unary `-` | prefix | `p-neg2` `100 + -2^2` = 104 (not 96); `u-u22` `-a^2` = +4; `q-q7` `-a^2*2` = 18; `p-neg3` `-2*3^2` = −18 |
| 3 | `^` | **left** | `p-pow2b` `(2^3^2)/8 + 0.5` = 8.5 (not 64.5); `q-q16` `3^2^0.5` = 3 (not 4.7) |
| 4 | `*` `/` | left | `p-div3` `64/4/2` = 8; `q-q1` `8/2*4` = 16; `q-q17`; `p-powmul` `2*3^2` = 18; `q-q5` `2^3*2` = 16 |
| 5 | `+` `-` | left | `p-sub3` `10-4-3` = 3; `q-q2` `10-2+3` = 11; `p-mul` `2+3*4` = 14 |
| 6 | `<` `<=` `>` `>=` `==` `!=` | left, one level | `p-cmp2` `100 + 3 < 4` = 0; `p-cmp3` `3>2>1` = 0; `p-cmp4` `0<3==1` = 1 (not 0); `p-cmp8` `5==1<2` = 1 (not 0); `q-q19`; `q-q10` |
| 7 | `&&` `\|\|` | left, **one level** | `p-and2` `1\|\|1&&0` = 0 and `p-and6` `1\|\|0&&0` = 0 (`&&` is not tighter); `p-and3` `0&&0\|\|1` = 1 (`\|\|` is not tighter); `p-and4`, `p-and5`, `q-q11`, `q-q12` (comparisons and `+` bind tighter) |
| 8 | `=` | right, statement level only | `p-asg2`; §2.5 |

- The right operand of `^` may not begin with `-` (error 32); `2^(-1)` is
  fine (`p-pow3`).
- **Unary minus before `(` and a literal (quirk).** When `-` is followed by
  `(` and the first token inside is a numeric or complex literal (possibly
  after more `(`), the minus is lost and that first literal counts as 0:
  `-(2)` = 0, `-(2.5)`, `-((2))`, `-(3,0)`, `-((1,2))` = 0
  (`n-n1`…`n-n3`, `n-n11`, `r3-uminus-um9`), `-(1+1)` = 1, `-(1+2)` = 2,
  `-(5+1)` = 1, `-(7-1)` = −1, `-(1+1+1)` = 2, `-(2*3)` = 0, `-(8/2)` = 0,
  `-(.5+1)` = 1, `-(1e1+1)` = 1, `-(2+a)` = a (`n-n6`…`n-n10`, `n-n12`,
  `n-n13`, `n-n23`, `n-n24`, `r3-uminus-um7`, `-um8`, `-um1`). The same holds
  in the iteration section (`r3-uminus-um11`) and after another operator
  (`n-n18` `3 * -(2)` = 0, `n-n16`, `p-negp1`, `u-u3`…`u-u17`). It does
  **not** happen when the parenthesis starts with a variable or a call:
  `-(a)`, `-(a+2)`, `-(sqr(3))`, `-(pi)`, `-(a)*3` are correct (`u-u1`,
  `r3-uminus-um2`, `n-n14`, `n-n15`, `r3-uminus-um10`). `-(2)^2` is 0^2,
  which XFractint evaluates as an invalid value (colour 1: `p-negpow2`,
  `n-n20`, and `0^2` itself, `r3-uminus-um4`…`-um6`). For pixel
  compatibility FractalForge must reproduce this; a warning is advisable.

---

## 5. Control flow (confirmed)

- Keywords `if`, `elseif`, `else`, `endif`, case-insensitive (`i-if7`). Each
  is a statement of its own, separated like any statement (newline or comma:
  `i-if1`, `i-if5`).
- `if (expr)` and `elseif (expr)`: the condition must be in parentheses
  right after the keyword, and nothing may follow on that statement:
  `if (1) k = 105` is error 1 (`i-if8`), `if (1) then` error 1 (`i-if22`),
  `if ()` error 0 (`i-if14`), `if = 5` / `if` used as a value: error 16 `Jump
  instruction missing required (boolean argument)` (`i-if27`).
- `else` and `endif` take nothing: `else (1)`, `endif (1)`, `endif = 5` are
  error 15 `No characters may follow this jump instruction` (`i-if25`,
  `i-if24`, `i-if26`).
- Structure: `if` … { `elseif` … } [ `else` … ] `endif`; branches may be
  empty (`i-if13`, `i-if19`); only the first true branch runs (`i-if33`);
  blocks nest (`i-if12`; 50 levels deep work, `t-nest50`). Errors:

| Situation | Error | Evidence |
|---|---|---|
| `endif` without `if` | 18 `"endif" has no matching "if"` | `i-if10`, `r4-it-ie2` |
| `else`/`elseif` without `if` | 19 `Misplaced "else" or "elseif()"` (then 18 for the `endif`) | `i-if23`, `i-if34` |
| second `else`, or `elseif` after `else` | 17 `Next jump after "else" must be "endif"` | `i-if11`, `e-er30`, `r4-it-ie3` |
| `if` without `endif`, initialisation section | 20 `"if()" in initialization has no matching "endif"` | `i-if9`, `t-ifit-it4` |
| `if` without `endif`, iteration section | 21 `"if()" has no matching "endif"` | `r4-it-ie1` |
| 200 or more jump keywords | 23 (§1.7) | `t-nest100` |

- `if` blocks are allowed in both sections (`i-if*` in the initialisation
  section, `t-ifit-*` in the iteration section) but must not contain the
  `:` (§2.3). Their value as the last statement: §2.4.
- There are no loops or other control statements: `while (1)` is a call of
  an unknown function (`i-if29`, error 5).

---

## 6. Predefined names (syntactic list)

Values: frm-semantics.md §2 and §4. Here only which names exist.

### 6.1 Variables (confirmed)

Names with a value of their own (reading them gives something other than 0
in at least one probe): `pixel`, `p1`, `p2`, `p3`, `p4`, `p5`, `pi`, `e`,
`rand`, `ismand`, `maxit`, `scrnmax`, `scrnpix`, `whitesq`, `center`,
`magxmag`, `rotskew` (`b-rpixel`, `b-rp1`…`b-rp5`, `b-rpi`, `b-re`, `v-v5`,
`b-rismand`, `b-rmaxit`, `b-rscrnmax`, `b-rscrnpix`, `b-rwhitesq`, `v-v1`,
`v-v2`, `b-rmagxmag`, `v-v9`, `v-v3`, `v-v4`). `lastsqr` is accepted and read
as 0 everywhere (`b-rlastsqr`, `r5-lastsqr1`), so whether it is predefined is
**unknown** from syntax alone. `z` is an ordinary variable whose final value
XFractint also uses for colouring (frm-semantics.md §1.4).

All of them can be assigned (`b-apixel`, `b-ap1`, `b-api`, `b-ae`, `b-arand`,
… all colour 131). `p6`, `x`, `y`, `c`, `i`, `j`, `t`, `w`, `q`, `zero`,
`one` are not predefined (read as 0: `b-rp6`, `b-rx`, …). `params=` takes at
most 10 values (`v-v19`: 12 is a bad argument).

### 6.2 Functions (confirmed)

Accepted (no error 5): `abs acos acosh asin asinh atan atanh cabs ceil conj
cos cosh cosxx cotan cotanh exp flip floor fn1 fn2 fn3 fn4 imag log real round
sin sinh sqr sqrt srand tan tanh trunc` (`b-f*`, `v-v10`, `v-v11`, `v-v20`).
`fn1`…`fn4` take their meaning from `function=` (frm-semantics.md §4.2).

Not functions (error 5): `acot arg asec atan2 cot coth csc csch cube fn0 fn5
ident ln log10 max min mod neg norm one pow rand recip sec sech sign zero`
(`b-f*`, `v-v12`, `v-v13`). Other names are untested.

### 6.3 Keywords

`if`, `elseif`, `else`, `endif` (§5).

---

## 7. Errors

### 7.1 How XFractint reports formula errors (confirmed)

- The whole entry is checked when it is loaded, before any pixel is
  computed: one run reports errors from both sections (`s-lineno-err3`,
  `r4-it-ie4`).
- In batch mode the messages go to `stopmsg.txt`, the run continues, an
  image is saved in which **every pixel has colour 1**, the type is 73 and
  `fractint003` holds the entry name (`a-bad1`, all error cases), and the
  exit status is 0 (no status line in the logs). Interactive behaviour was
  not tested.
- Message layout, one block per error:

  ```
  Error(N) at line L:  MESSAGE
    STATEMENT-WITHOUT-BLANKS
    ^^^^
  ```

  The caret line marks the offending token (one `^` per character,
  `i-id15`, `p-lit10`). `L` counts lines inside the entry, the header line
  being 1 and blank and comment lines included (`s-lineno-err`: header on
  file line 4, error reported at line 5; `s-lineno-err2`: one-line entry,
  line 1).
- **At most three errors** are reported (`r4-it-ie4`: 12 bad statements, 3
  reported; `s-lineno-err3`: 3 of 3), and at most one per statement
  (`e-er14`: 11 errors in one statement, 1 reported).

### 7.2 Error classes seen

| N | Message | Cause (evidence) |
|---|---|---|
| 0 | Should be an Argument | operand missing: `z*+`, `**`, `++5`, `k =`, `if ()`, `sin()`, `\|\|`, `<<` (`a-bad1`, `p-star2`, `p-uplus`, `e-er3`, `i-if14`, `b-call1`, `p-modnest2`, `e-er24`) |
| 1 | Should be an Operator | operand where an operator is needed: `2(3)`, `if (1) k=…`, `3\|`, `1\|1` (`p-imul`, `i-if8`, `e-er6`, `p-or7`) |
| 2 | ')' needs a matching '(' | `k = )` (`e-er12`) |
| 3 | Need more ')' | `k = (`, `sin(1` (`e-er13`, `e-er8`) |
| 4 | Undefined Operator | `!`, `&` (`p-not1`, `p-and7`) |
| 5 | Undefined Function | unknown name called (`a-unkfn`) |
| 15 | No characters may follow this jump instruction | `else`/`endif` with text (`i-if24`) |
| 16 | Jump instruction missing required (boolean argument) | `if`/`elseif` without `( … )` (`i-if27`) |
| 17 | Next jump after "else" must be "endif" | §5 |
| 18 | "endif" has no matching "if" | §5 |
| 19 | Misplaced "else" or "elseif()" | §5 |
| 20 | "if()" in initialization has no matching "endif" | §5 |
| 21 | "if()" has no matching "endif" | §5 |
| 23 | Formula has too many jump commands | §1.7 |
| 25 | Only variables are allowed to left of assignment | §2.5 |
| 26 | Illegal variable name | `a.b` (`i-id8`) |
| 27 | Invalid constant expression | §3.2 |
| 28 | This character not supported by parser | §3 |
| 30 | Unmatched modulus operator "\|" in this expression | `\|3` (`e-er5`) |
| 31 | Can't use function name as variable | §3.1 |
| 32 | Negative exponent must be enclosed in parens | §3.4 |
| 33 | Variable or constant exceeds 32 character limit | §3.1, §3.2 |
| 34 | Only one ":" permitted in a formula | §2.3 |

Codes 6–14, 22, 24 and 29 were never produced; their meaning is **unknown**.
Other load-time messages (no code): `'NAME' file entry item not found` +
`Could not open file where formula located` (§1.3), `Didn't find matching
')' in symmetry declaration` (§1.2), `Formula has no executable
instructions` (§2.3).

### 7.3 Not errors (confirmed)

Unknown variables (value 0, §3.1), variables read before they are assigned
(`a-undef`, `i-id10`), unknown qualifier words and junk in the header
(§1.2, §1.5), text outside entries (§1.2), a missing `:` (§2.3), an empty
iteration section (§2.3), NaN or infinite constants (`e-er22`, `p-lit19`).

### 7.4 Recommendations for FractalForge

Report every error with its line and column instead of rendering the
colour-1 image; keep XFractint's classes (the table above) so messages are
recognisable. Never crash on large formulas (§1.7). Accept but warn about
the two quirks that change values (§3.2 signed exponents, §4 `-(` literal).

---

## 8. Storage

### 8.1 GIF (confirmed)

- `fractint001` offset 10 holds type **73** (every case). The function
  slots 218–221 are stored as for other types: default `0, 6, 2, 3`,
  `function=cos/tan/exp/log` → `9, 10, 4, 5` (`r3-store-sf1`,
  `r3-store-fnset`; index table in gif-fractint-blocks.md §6.2). Params as in
  gif-fractint-blocks.md §2.1.
- The symmetry qualifier is **not stored**: offset 124 stays 999 for every
  qualifier (all `y-*` cases); only `symmetry=` changes it (1 for xaxis, 2
  for yaxis, 0 for none: `y-00-plain-symx`, `y-01-xaxis-symy`,
  `y-01-xaxis-symnone`).
- The formula **file name is not stored** anywhere in the GIF (a byte search
  for the file names in `s-comments`, `r3-store-s1`, `a-quad` finds nothing).
- `fractint003` (66 bytes) for formulas. This refines
  gif-fractint-blocks.md §2.3, whose 48-byte name field and "unknown i16 at
  48" overlap these flags:

| Offset | Type | Meaning | Evidence |
|---|---|---|---|
| 0 | ASCII, NUL-padded | entry name **as typed** in `formulaname=` (not as written in the file), without any `frm:` prefix found in a `.par` (§8.2); 18 characters at most were possible, so the field is at least 18 and at most 40 bytes | `s-case-mixed` (`mixed` for entry `Mixed`), `s-case-dupU` (`DUP`), `r3-store-len18`, `r3-par-img1` (`pq`), `r4-par-img8` (`PQ`) |
| 40 | i16 | 1 if the formula mentions `p1` (read or assigned, either section), else 0 | `r3-store-s1`, `r3-store-sab` (`p1 = 3`), `r3-store-s13`, `b-rp1` |
| 42 | i16 | the same for `p2` | `r3-store-s2`, `r3-store-p2iter` (only in the iteration section) |
| 44 | i16 | the same for `p3` | `r3-store-s3`, `r3-store-s13` |
| 46 | i16 | 1 if the formula mentions `ismand` | `r3-store-sis`, `v-v14` |
| 48 | i16 | the `ismand=` setting: 1 by default, 0 with `ismand=no` | `v-v14` (0), all other cases (1) |
| 50 | i16 | the same as 40 for `p4` | `r3-store-s4`, `b-rp4` |
| 52 | i16 | the same as 40 for `p5` | `r3-store-s5`, `b-rp5` |
| 54–65 | | 0 in every case | all |

  No flag is set for `fn1`–`fn4`, `pixel`, `p6` or any other name
  (`r3-store-sf1`…`-sf4`, `-spx`, `-sp6`, …). A formula with an error stores
  only the name and offset 48 (`a-bad1`); a missing entry likewise
  (`a-missing`). **Hypothesis:** these flags tell the `.par` writer which
  params to write (par.md: params are written only for formulas that use
  them, `wfrmpar`).
- Loading a formula GIF needs the formula file again, found through the
  current `formulafile` (hypothesis; the batch loader re-saves without
  recalculating, gif-fractint-blocks.md "Observations").

### 8.2 `.par` files (confirmed)

- **Written by XFractint** (par.md §4): `type=formula formulafile=ff.frm
  formulaname=ffcube`, the file name as given on the command line, and no
  formula text (`corpus/xfractint/par-map/written/wfrm.out.par`; one entry
  only). Params are written only if the formula uses them
  (`wfrmpar.out.par`, par.md).
- **Read**, file names: inside an entry, `formulafile=` is not looked up in
  the current directory (par.md §5.3).
- **Formulas embedded in a `.par` file work**: an entry named `frm:NAME` in
  the same `.par` file is used when an entry of that file says
  `formulaname=NAME` (`r3-par-img1`, without `formulafile=`;
  `r3-par-img2`, with `formulafile=p1.par`). Details:
  - the `frm:` prefix is case-insensitive (`FRM:upper`, `r4-par-img9`) and so
    is NAME (`formulaname=PQ`, `r4-par-img8`);
  - the `frm:` entry may come after the image entry (`r4-par-img6`);
  - `formulaname=frm:pq` inside the entry does **not** find it
    (`r3-par-img3`), and a plain entry `pr` of the `.par` (no prefix) is not
    used as a formula (`r4-par-img4`), nor are `.frm` files of the current
    directory (`r4-par-img5`);
  - the embedded formula wins over a same-named entry in a `.frm` file of the
    current directory (`r4-par-img10`: `frm:quad` used, not `a.frm`'s
    `quad`), but a `formulafile=` given on the command line **before** the
    `@file/entry` wins over it (`r4-par-img10f`, `r4-par-img5f`,
    `r3-par-cmdfile`);
  - the body syntax is that of a `.frm` entry (the probe body parsed
    normally); the name is stored without the prefix (`r3-par-img1`: `pq`).
- A `.par` file given as `formulafile=` on the command line is read as a
  formula file whose entry names include the prefix: `formulaname=frm:pq`
  works, `formulaname=pq` does not, and plain entries work
  (`r3-par-frmfile2`, `r3-par-frmfile`, `r3-par-frmfile3`).
- **Recommendation for FractalForge's writer:** write `formulafile=` and
  `formulaname=` as XFractint does, and optionally also copy the formula
  into the same `.par` as an `frm:NAME { … }` entry, which makes the file
  self-contained and which XFractint reads.

---

## 9. Grammar

### 9.1 Processing order

1. Read the file up to the first 0x1A byte. Lines end at LF; a CR before LF
   is ignored.
2. Find entries (§1.2): a line whose first word is NAME, optionally
   followed by `(QUALIFIER)` and other text, then `{`, on that line. The body
   runs from after `{` to the first `}` outside a comment (the search may
   continue across lines). A `{` before that `}` invalidates the entry.
   Lookup is case-insensitive; the first match wins.
3. In the body: delete comments (`;` to end of line); join a line ending in
   `\` (blanks or a comment may follow it) with the next line; delete every
   other `\`.
4. Split into statements at newlines, `:` and commas, except for commas
   inside a complex-literal token (§3.3). Remember each statement's line
   number (header line = 1).
5. Delete blanks (spaces, tabs) inside each statement.
6. Tokenize (§3) and parse each statement with the grammar below; check the
   block structure (§5), the single `:` (§2.3) and the limits (§1.7).

### 9.2 EBNF (after step 5; tokens are case-insensitive)

```ebnf
body          = init-section ":" iter-section
              | iter-section ;                      (* no ":" : §2.3 *)
init-section  = block ;
iter-section  = block ;
block         = { statement-or-if } ;               (* separators omitted; empty statements allowed *)
statement-or-if = statement | if-block ;
if-block      = "if" condition block
                { "elseif" condition block }
                [ "else" block ]
                "endif" ;                            (* each keyword is its own statement *)
condition     = "(" expression ")" ;                 (* nothing may follow ")" *)
statement     = assignment | expression ;
assignment    = variable "=" ( assignment | expression ) ;

expression    = comparison { ( "&&" | "||" ) comparison } ;
comparison    = sum { relop sum } ;
relop         = "<" | "<=" | ">" | ">=" | "==" | "!=" ;
sum           = product { ( "+" | "-" ) product } ;
product       = power { ( "*" | "/" ) power } ;
power         = unary { "^" primary } ;             (* left-assoc; operand may not start with "-" (error 32) *)
unary         = "-" unary | primary ;                (* see the "-(" literal quirk, §4 *)
primary       = number | complex | variable | call
              | "(" expression ")"
              | "|" bar-expression "|" ;
bar-expression = expression ;                        (* may contain "|" only inside parentheses, §3.4 *)
call          = function "(" expression ")"
              | function complex ;

variable      = identifier ;                         (* not a keyword or function name *)
function      = "abs" | "acos" | "acosh" | "asin" | "asinh" | "atan" | "atanh"
              | "cabs" | "ceil" | "conj" | "cos" | "cosh" | "cosxx" | "cotan"
              | "cotanh" | "exp" | "flip" | "floor" | "fn1" | "fn2" | "fn3"
              | "fn4" | "imag" | "log" | "real" | "round" | "sin" | "sinh"
              | "sqr" | "sqrt" | "srand" | "tan" | "tanh" | "trunc" ;
identifier    = ( letter | "_" ) { letter | digit | "_" } ;   (* <= 32 characters *)
number        = ( digits [ "." { digit } ] | "." digits )
                [ ( "e" | "E" ) exponent ] ;                  (* <= 32 characters *)
exponent      = digits
              | ( "+" | "-" ) digits ;               (* signed form: value quirk, §3.2 *)
complex       = "(" [ "-" ] number "," [ "-" ] number ")" ;  (* one token, §3.3 *)
digits        = digit { digit } ;
```

### 9.3 Entry header (before blank removal)

```ebnf
entry         = { blank } name { blank } [ qualifier ] { other-char } "{" body-text "}" ;
name          = name-char { name-char } ;     (* ends at blank, "(", "[", "{"; ";" starts a comment *)
qualifier     = "(" { any-char-but-")" } ")" ;  (* compared without case and blanks, §1.5 *)
```
