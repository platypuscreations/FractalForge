<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Fractint-format `.map` palette files

Status: **Phase 2 draft, derived from black-box analysis only.** Every rule
below comes from palette files we wrote ourselves, loaded into XFractint, and
the colour table of the GIF it then saved, or from a `.map` file XFractint
wrote. No Fractint, XFractint or Iterated Dynamics source code, help text or
documentation was consulted, and none of the `.map` files bundled with
XFractint were opened. See [CLEANROOM.md](../CLEANROOM.md).

Conventions: **Confirmed** = shown directly by a test. **Hypothesis** = fits
the evidence, not proven. **Unknown** = not tested or not determinable.
Evidence names refer to `corpus/xfractint/par-map/` (see its README). A
case name such as `m-mfull` is a batch render with `map=mfull.map`, made from
the input file `tools/blackbox/par-map/inputs/mfull.map` (command lines in
`cases.txt`); its outcome is in `results-read.txt` and its GIF in
`read/m-mfull.gif`. Unless stated otherwise every test
used XFractint 20.4.10 (Debian `xfractint 20.4.10-5`, amd64 under qemu).

How a palette is observed: XFractint stores colours internally with 6 bits per
channel and writes them to the GIF global colour table expanded to 8 bits by
bit replication, `gif = (v6 << 2) | (v6 >> 4)` (see
[gif-fractint-blocks.md](gif-fractint-blocks.md) §1.4). So the 6-bit value
XFractint used for an entry is `gif >> 2`.

## 1. Summary for implementers

- Plain text, one palette entry per line, entries 0, 1, 2, … in file order.
- Each line starts with three decimal integers, red green blue, **0–255**,
  separated by spaces or tabs. Anything after the third number is ignored.
- XFractint keeps only the top 6 bits: `v6 = (v mod 256) >> 2`.
- At most 256 lines are used. Missing entries become grey `(40, 40, 40)` in
  6-bit units (`a2 a2 a2` in a GIF).
- **Every line uses up one entry**, including blank lines and comment lines.
  A line (or part of a line) that doesn't parse leaves the unparsed channels
  at the previous entry's values.
- Writer: 256 lines, `"%3d %3d %3d\n"` of `v6 * 4` (XFractint's own
  format), or any three 0–255 integers per line. No comments, no blank lines.

## 2. Reading

### 2.1 Line format (confirmed)

| Input line(s) | Result | Evidence |
|---|---|---|
| `12 243 36` | entry = (12, 243, 36) >> 2 | `m-mfull` (256 lines `k 255-k 3k mod 256`): all 256 entries match |
| leading spaces, `%3d %3d %3d` padding, tabs between numbers | same as plain | `m-mlead`, `m-mpad`, `m-mtabs` |
| text after the three numbers (`0 255 0   entry 0 some text 9 9 9`) | ignored | `m-mtrail` |
| CRLF line ends; no newline after the last line | no difference | `m-mcrlf`, `m-mnoeol` |
| 0–63 values ("6-bit map") | **not** auto-detected: read as 8-bit, so they come out four times darker (`63` → 6-bit 15) | `m-m6bit` |

### 2.2 Range and scaling (confirmed)

Each value is reduced modulo 256 (only its low 8 bits are kept) and then
shifted right by 2. From `m-mover`:

| Line | 6-bit result |
|---|---|
| `300 0 0` | (11, 0, 0): 300 mod 256 = 44, 44 >> 2 = 11 |
| `1000 256 511` | (58, 0, 63) |
| `-5 0 0` | (62, 0, 0): -5 → 251 |
| `0 -1 255` | (0, 63, 63) |
| `256 0 0` | (0, 0, 0) |

Two 8-bit values that differ only in the low two bits give the same colour.
Since the GIF stores `(v6 << 2) | (v6 >> 4)`, an 8-bit value read from a map
and written to a GIF changes, e.g. 255 → 255, 254 → 255, 7 → 4.

### 2.3 Lines that don't hold three numbers (confirmed)

The file is read as a sequence of lines, and line *k* (counting from 0) is
always palette entry *k*. A line that is blank, a comment, or otherwise can't
be parsed still takes up its entry. Channels that can't be read keep the
values of the **previous** entry. The numbers are read like C-style decimal
integers: an optional sign and digits, stopping at the first other character.

| Line(s) at entry k | Entry k becomes | Evidence |
|---|---|---|
| blank line | copy of entry k−1 | `m-mblank` (a blank line after every 4th entry: entry 2 = entry 1, then the file is one line "behind" for the rest) |
| `; semicolon comment`, `# hash comment` | copy of entry k−1 (each line uses one entry) | `m-mcomm` |
| `red green blue` | copy of entry k−1 | `m-mword` |
| `77 88` (two numbers) | r = 77 >> 2, g = 88 >> 2, b from entry k−1 | `m-mtwo` |
| `12.7 33.2 99.9` | r = 12 >> 2; g and b from entry k−1 (reading stops at `.`) | `m-mfloat` |
| `0x10 0x20 0x30` | r = 0; g and b from entry k−1 | `m-mhex` |
| `0,255,0` (commas) | r read; g and b keep their previous values. On the **first** line they were (80–83, 64–67), i.e. left over from somewhere | `m-mcommas` |

What the "previous values" are for an unparseable **first** line is
**unknown** in general (see the comma case). Readers should use 0.

**Hypothesis:** each line is parsed with something equivalent to
`sscanf("%d %d %d")` into three variables that persist between lines.

### 2.4 Number of entries (confirmed)

- More than 256 lines: lines after the 256th are ignored (`m-mlong`, 300
  lines).
- Fewer than 256 lines: the remaining entries are set to 6-bit
  **(40, 40, 40)**, GIF `a2 a2 a2`. Not black, and not the previous palette:
  `m-mshort` (16 lines), `m-mone` (1 line), and `m-twice`
  (`map=mfull.map map=mshort.map`: entries 16–255 are grey, not `mfull`'s).
  The same filler appears when a `colors=` string in a `.par` file is short
  (see [par.md](par.md) §6.4).

### 2.5 File names (confirmed)

- `map=mfull` (no extension) loads `mfull.map` (`m-noext`).
- On the **command line**, a relative name is opened in the current
  directory, and an absolute path works (`m-*`, `pa-cmd-mapabs`). A
  missing file gives the message `Could not load color map ./nosuch.map` and
  batch mode saves no image (`m-missing`).
- Inside a **`.par` entry** (`map=` there) and in `colors=@name` anywhere,
  XFractint looks only in its own maps directory
  (`/usr/share/xfractint/maps/` in this build): relative names, `./name` and
  absolute paths all failed with `Could not load color map
  /usr/share/xfractint/maps/ff.map` (`c3-mapdot`, `c3-mapabs`,
  `c3-cmapabs`, `pa-cmd-colorsatabs`), and a file copied into that directory
  was found (`special/maps-dir-check.gif`, see par.md §6.5). So the directory part
  is dropped and replaced. **Hypothesis**; FractalForge should resolve
  relative to the `.par` file first, then its own palette directory.
- `map=` given twice: the later one wins (`m-twice`).

### 2.6 How XFractint applies a map (confirmed)

The map replaces the whole 256-entry palette (with grey filler for missing
entries, §2.4). The table written to the GIF is the 6-bit palette expanded
with bit replication. The `fractint` blocks don't change.

The **pixel indices** in XFractint's saved GIF can change too, which is an
artefact of how this build saves (hypothesis: it reads the image back from
the X display and maps colours back to indices), not part of the format:

- Entries with identical colours collapse to the **lowest** such index
  (e.g. `mfull.map` gives entries 0 and 1 the same 6-bit colour, and every
  pixel of colour 1 in the reference is saved as 0).
- Whenever any map or `colors=` palette is loaded, 728 more pixels change
  in this image (index 0 → 16 and 15 → 31 in most cases); unexplained. With
  these two effects removed, every map case has the reference pixels
  (`m-mfull`, `m-mshort`, `m-mover`, `c-cdigits`; `c3-crun2` and `c-cshort1`
  show a variant: 0 → 4, 0 → 1).

FractalForge should simply keep the computed indices and change the palette. `reset` inside a `.par` entry does **not** restore the
default palette (`c3-reset-palette`, `c3-reset-cmdline`).

## 3. What XFractint writes (confirmed for one file)

The palette editor (`e`, Enter, `s`, then a file name) wrote
`written/xf-mapedit.map` (case `sp-mapedit`, native arm64 build of the same Debian package,
palette loaded from `mfull.map`):

```
  0 252   0
  0 252   0
  0 252   4
```

- 256 lines of exactly 11 characters, `\n` line ends (including after the
  last line), nothing else (no header, no comments).
- Each value is right-aligned in **3 characters**, separated by one space
  (`printf("%3d %3d %3d\n")`).
- Values are the 6-bit entries **times 4** (252 for 63), not bit-replicated.

## 4. Writer recipe

To write a `.map` XFractint reads back exactly:

1. Take the 256 palette entries as 6-bit values `v6` (0–63). If the palette
   is 8-bit, use `v6 = v8 >> 2`; that is what XFractint will do anyway.
2. Write 256 lines, entry 0 first, each
   `format!("{:3} {:3} {:3}\n", r * 4, g * 4, b * 4)`.
3. No comments and no blank lines: they would shift every following entry.
   If a description is wanted, put it **after** the three numbers on a line
   (`m-mtrail` shows this text is ignored).

Writing 8-bit values (`v8` instead of `v6 * 4`) is also safe for XFractint,
since it drops the low two bits; it keeps more information for programs that
read 8 bits. Either is acceptable. FractalForge should write `v6 * 4` when
the palette came from a Fractint-format source (map, par, GIF), for byte
compatibility with XFractint.

## 5. Reader recipe (tolerant)

```text
entries = []
prev = (0, 0, 0)
for each line in file (split on \n, strip a trailing \r):
    if entries.len == 256: stop
    parse up to three integers from the start of the line (skip leading
      spaces/tabs; an integer is [+-]?[0-9]+; stop at the first failure)
    r, g, b = parsed values, the rest taken from prev
    entry = ((r mod 256) >> 2, (g mod 256) >> 2, (b mod 256) >> 2)   # 6-bit
    entries.push(entry); prev = (r, g, b)
while entries.len < 256: entries.push((40, 40, 40))
```

This reproduces XFractint for every case in §2. Optional FractalForge
extensions (warnings, not errors): report blank/comment lines and values
outside 0–255, because XFractint's results there are rarely what the author
meant. Do not skip blank or comment lines silently: that would differ from
XFractint.

## 6. Verification performed

All in the black-box containers (`tools/blackbox/par-map/`):

| Test | Cases | Result |
|---|---|---|
| line layout variants | `m-mfull`, `-mlead`, `-mpad`, `-mtabs`, `-mtrail`, `-mcrlf`, `-mnoeol` | all 256 entries = `(v mod 256) >> 2` |
| bad lines | `m-mblank`, `-mcomm`, `-mword`, `-mtwo`, `-mfloat`, `-mhex`, `-mcommas` | §2.3 |
| counts | `m-mshort`, `-mone`, `-mlong`, `-twice` | §2.4 |
| range | `m-mover`, `-m6bit` | §2.2 |
| writer output | `sp-mapedit` → `written/xf-mapedit.map` (native arm64 build) | §3 |

Not tested: files larger than a few kilobytes, non-ASCII bytes, a line longer
than 512 characters (the `.par` reader splits such lines, see par.md §2.6;
the map reader might too).
