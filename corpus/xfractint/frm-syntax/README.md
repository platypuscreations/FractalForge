<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Evidence for docs/specs/frm-syntax.md

Batch renders by Debian's prebuilt `xfractint 20.4.10-5` (amd64 under qemu)
of formula files we wrote ourselves (`tools/blackbox/frm-syntax/inputs/`).
`cases.txt` lists every case with its arguments; `tools/blackbox/frm-syntax/
run-case.sh` adds `type=formula passes=1 periodicity=0 viewwindows=2
map=uniq.map` before them and `savename=ff.gif` after them, and runs each
case in a fresh directory that holds all of `inputs/`.

One folder per round (`cases-<round>.txt`):

| Folder | What it tests |
|---|---|
| `cal` | the colour-index encoding of the probe entries |
| `a` | first error messages, missing entry and missing file |
| `p`, `q` | operators, precedence, associativity, number literals |
| `u`, `n` | exponent-sign and unary-minus-before-`(` behaviour |
| `s`, `t` | file structure: comments, braces, names, separators, continuation, limits |
| `i` | identifiers and `if`/`elseif`/`else`/`endif` |
| `b`, `v` | built-in variable and function names |
| `y` | symmetry qualifiers in entry headers |
| `r3` | `z` in the initialisation section, backslashes, jump limit, PI_SYM views, `fractint003` flags, long entries, `.par` embedding |
| `e` | error classes |
| `r4` | errors in the iteration section, formulas inside `.par` files |
| `r5` | `lastsqr`, missing-file fallback |
| `r6` | characters that end an entry name |
| `w` | modulus bars, identifier and literal length |

Per case: `<case>.gif` (the saved image, renamed from `ff.gif`),
`<case>.stopmsg.txt` (XFractint's batch-mode message file, when written)
and `<case>.log` (console output, kept only when it holds more than the qemu
platform warning and the closing banner; it may contain BEL characters).
`results.txt` in each folder is `tools/blackbox/frm-syntax/summarise.py`
run on that folder; the spec cites its lines by case name.

Notes:

- `b-rrand` and `b-rirand` did not finish within 10 minutes and were
  stopped by us (the "Fatal X IO error" in their logs comes from that).
- Several large-formula cases crashed XFractint (segmentation fault or
  `munmap_chunk(): invalid pointer`); that is the observed behaviour, see
  spec §1.7.
- `t-latin1-name` is inconclusive: the non-ASCII byte reached XFractint as
  a UTF-8 replacement character.
