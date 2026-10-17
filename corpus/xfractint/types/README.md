<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Fractal-type reference renders

Evidence for [docs/specs/types.md](../../../docs/specs/types.md). XFractint
(Debian `xfractint 20.4.10-5`, amd64) saved every GIF here as a black box in
`tools/blackbox`. Each one is 400×300 and was rendered with
`passes=1 periodicity=0 viewwindows=2`, so implementers can compare their own
pixels one for one.

- `cases.txt`: the command line for every file, one `NAME ARGS` per line.
  `tools/blackbox/types/render.sh` adds `batch=yes passes=1 periodicity=0
  viewwindows=2 … savename=ff.gif`.
- `<type>-def.gif`: the type with nothing but `type=` given, so default
  corners, parameters, bailout and maxiter (150).
- `<type>-p*.gif`: the same type with non-default `params=`.
- Pairs `*-off` / `*-offnosym`, `*-wN` / `*-wNn` and
  `*-s` / `*-sn`: the same view with the default symmetry setting and with
  `symmetry=none`. The views straddle an axis off-centre, so we can see which
  rows and columns XFractint mirrors.
- `*-mi*.gif`, `*-bo*.gif`: maxiter and bailout variants.
- `SHA256SUMS`: run `shasum -a 256 -c SHA256SUMS` in this folder.

Regenerate with `tools/blackbox/types/gen.sh OUTDIR` (it reads this
`cases.txt`). Pixels come out the same. Only the seed and the calculation time
in the `fractint001` block change between runs.
