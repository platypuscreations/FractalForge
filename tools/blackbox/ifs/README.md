<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# IFS experiments

Scripts and input files behind [docs/specs/ifs.md](../../../docs/specs/ifs.md)
and the evidence in [corpus/xfractint/ifs](../../../corpus/xfractint/ifs).
Black-box only: they drive the `fractalforge-blackbox` image and read the
files it saves. Every `.ifs` file in `inputs/` is our own (`ffcopy.ifs` is a
copy of `../inputs/ff.ifs`). See [docs/CLEANROOM.md](../../../docs/CLEANROOM.md).

| File | Purpose |
|---|---|
| `render.sh MODE OUT NAME ARGS…` | one case in a fresh `mktemp` directory under `OUT`: `batch`, `int:DELAY` (press `s` after DELAY s) or `keys:"ACTIONS"` (`../resume/run-keys.sh`). Copies `inputs/*.ifs` (unless `NOINPUTS=1`) and the files in `EXTRA` |
| `run3d.sh OUT ARGS…`, `runpair.sh OUT ARGS…` | every entry of `inputs/pt3.ifs` / `inputs/pair3.ifs` with the same extra arguments |
| `gen-digits.py OUT.ifs K…` | writes the "digit recorder" entries `dK` (`inputs/digits.ifs`) |
| `ifsmodel.py IMAGE.gif FILE.ifs ENTRY [rot=a/b/c] [shift=x/y]` | the spec's model: predicts every pixel of a 2D or 3D case and counts differences (reads corners, maxiter and param 1 from the GIF) |
| `par/` | `.par` experiments: `ip.par` (image entries plus embedded `ifs:` / `lsys:` bodies), `pf.ifs`, `lf.l`, reference copies `ref.ifs`, `lref.l`, and `run-writer.sh OUT NAME ARGS…` (interactive render plus the `b` screen in the native par-map image) |
| `inputs/*.ifs` | test entries; `cases.txt` in the corpus says which case uses which |

`ifsmodel.py` imports `../resume/gif.py`. Python runs as `python3 -I` and needs
only the standard library. Example:

```sh
python3 -I tools/blackbox/ifs/ifsmodel.py corpus/xfractint/ifs/leaf.gif \
  tools/blackbox/ifs/inputs/ffleaf.ifs ffleaf
```
