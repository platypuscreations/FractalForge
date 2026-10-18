<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Formula-semantics evidence

Batch renders by XFractint (Debian `xfractint 20.4.10-5`, amd64) running as a
black box in `tools/blackbox`. They back the rules in
[`docs/specs/frm-semantics.md`](../../../docs/specs/frm-semantics.md). Nothing
here was derived from Fractint source or documentation, and no formula file
shipped with XFractint was opened.

- One `<name>.gif` per non-comment line of
  [`tools/blackbox/frm-semantics/cases.txt`](../../../tools/blackbox/frm-semantics/cases.txt),
  which records the XFractint parameters of each file. The formulas are the
  entries of our own probe file
  [`tools/blackbox/frm-semantics/sem.frm`](../../../tools/blackbox/frm-semantics/sem.frm).
- Every case was rendered by `tools/blackbox/frm-semantics/render.sh`, which
  runs XFractint with
  `type=formula formulafile=sem.frm passes=1 periodicity=0 viewwindows=2 map=uniq.map`,
  then the case's own parameters (a later `periodicity=` overrides), then
  `savename=ff.gif`. Images are 400x300. `uniq.map` is
  `tools/blackbox/coloring/uniq.map` (256 distinct colours; see
  `docs/specs/coloring.md` section 1).
- `<name>.stopmsg.txt` is the error text XFractint wrote for cases whose
  formula it rejected (it still saved an image, every pixel colour 1).
- Renders are deterministic except where a formula reads `rand`: re-rendering
  `quad` and `hsh-PI_SYM-v` gave identical pixels.
- `SHA256SUMS`: `shasum -a 256` of every file here except itself, run from
  this folder.

Regenerate with `tools/blackbox/frm-semantics/run-cases.sh OUTDIR` (skips
names that already exist in OUTDIR).
