<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Colouring evidence

Batch renders by XFractint (Debian `xfractint 20.4.10-5`, amd64) running as a
black box in `tools/blackbox`. They back the rules in
[`docs/specs/coloring.md`](../../../docs/specs/coloring.md). Nothing here was
derived from Fractint source or documentation.

- One `<name>.gif` per non-comment line of
  [`tools/blackbox/coloring/cases.txt`](../../../tools/blackbox/coloring/cases.txt),
  which records the XFractint parameters of each file.
- Every case was rendered by `tools/blackbox/coloring/render.sh`, which adds
  `passes=1 periodicity=0 viewwindows=2 map=uniq.map savename=ff.gif`
  in front of the case's own parameters (a later `periodicity=` in the case
  overrides the default). Images are 400x300.
- `uniq.map` (in the tools folder) is our own palette whose 256 colours stay
  distinct after XFractint's 6-bit truncation. It is needed because XFractint
  saves the first palette index that has the computed colour, so duplicate
  colours in the default palette hide the real index (spec section 1).
- `defpal-m-iter.gif` and `defpal-z-iter-1000.gif` were rendered the same way
  but **without** `map=uniq.map` (`FFMAP= render.sh ...`), with the parameters
  of `m-iter` and `z-iter-1000`. They show the duplicate-colour effect.
- Renders are deterministic: re-rendering `m-in-zmag` and `j2-out-tdis` gave
  identical pixels and palettes.
- `SHA256SUMS`: `shasum -a 256` of every file here except itself, run from
  this folder.

Regenerate with `tools/blackbox/coloring/run-cases.sh OUTDIR` (skips names
that already exist in OUTDIR).
