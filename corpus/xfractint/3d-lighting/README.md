<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# 3D lighting and stereo evidence

Evidence for [docs/specs/3d-lighting.md](../../../docs/specs/3d-lighting.md).
Produced black-box with Debian `xfractint 20.4.10-5` in the
`fractalforge-blackbox` container; no XFractint source or documentation was
used.

- `cases.txt`: one line per GIF, `NAME INPUT ARGS`. Unless a section says
  otherwise the file was made by `tools/blackbox/3d-lighting/render.sh`, i.e.
  `xfractint batch=yes 3d=yes filename=in.gif ARGS savename=ff.gif` in a fresh
  `mktemp` directory containing only `in.gif` (copy of
  `tools/blackbox/3d-lighting/inputs/INPUT`) and `uniq.map`. Lines starting
  `# REJECTED` are arguments XFractint refused (it stops with an
  unknown-argument message and saves no image).
- Inputs: `tools/blackbox/3d-lighting/mkinput.py` writes every
  `inputs/*.gif` except `mandel.gif` / `mandel800.gif` (made by XFractint:
  `type=mandel maxiter=255 viewwindows=2`, and the same at 800×600) and
  `inputs/pal/*` (`mkpal.py IDX R G B`; `flat100rev.gif` = `flat100.gif` with
  the palette reversed, `flat100grey.gif` = with a grey palette).
- `reload-pl_ft5.gif`: `pl_ft5.gif` loaded interactively
  (`run-load 8 6 filename=in.gif savename=ff.gif`). `plasma-base.gif`:
  `type=plasma viewwindows=2` (no 3D), for the type number.
- Interrupt attempts (no files kept): `keyrun.sh OUT NAME INPUT "ACTIONS"
  filltype=N` with ACTIONS such as
  `s8 kReturn s3 kReturn s3 kReturn s3 kReturn s3 kReturn s2 ks` on
  `mandel800.gif` and `plateau.gif`, fill types 0, 2, 5, 6 and `stereo=1`.
  Screen captures showed the 3D parameter screens; after the last Enter
  XFractint died with a segmentation fault (qemu reported signal 11), so no
  partial image was saved.
- `SHA256SUMS`: `shasum -a 256` of every file here, run in this folder.

## Verification output

`tools/blackbox/3d-lighting/verify.sh`:

```
== light fills (sim.py)
e_fl5      match 1396/120000 (1.163%) [(1, 118604), (0, 1396)]
e_sp5      match 120000/120000 (100.000%) [(0, 120000)]
e_sp5b     match 120000/120000 (100.000%) [(0, 120000)]
e_sp5c     match 120000/120000 (100.000%) [(0, 120000)]
e_sp5d     match 120000/120000 (100.000%) [(0, 120000)]
e_sp6      match 120000/120000 (100.000%) [(0, 120000)]
e_sx5      match 120000/120000 (100.000%) [(0, 120000)]
e_sy5      match 120000/120000 (100.000%) [(0, 120000)]
e_sx1      match 120000/120000 (100.000%) [(0, 120000)]
x_r3       match 120000/120000 (100.000%) [(0, 120000)]
y_r3       match 120000/120000 (100.000%) [(0, 120000)]
p_r3       match 120000/120000 (100.000%) [(0, 120000)]
x_sm       match 120000/120000 (100.000%) [(0, 120000)]
sm1        match 120000/120000 (100.000%) [(0, 120000)]
sm2        match 120000/120000 (100.000%) [(0, 120000)]
sm5        match 120000/120000 (100.000%) [(0, 120000)]
smy3       match 120000/120000 (100.000%) [(0, 120000)]
n_r60      match 120000/120000 (100.000%) [(0, 120000)]
n_y1       match 120000/120000 (100.000%) [(0, 120000)]
q_f6a      match 120000/120000 (100.000%) [(0, 120000)]
q_f6c      match 120000/120000 (100.000%) [(0, 120000)]
q_f6e      match 102418/120000 (85.348%) [(0, 102418), (-1, 11026), (1, 6556)]
rp8        match 1396/120000 (1.163%) [(-1, 118604), (0, 1396)]
v_nz5      match 119528/120000 (99.607%) [(0, 119528), (-1, 174), (1, 22), (-45, 9), (45, 8)]
v_nz5s     match 118321/120000 (98.601%) [(0, 118321), (-1, 1485), (-6, 11), (-4, 10), (-8, 10)]
v_nz6      match 119280/120000 (99.400%) [(0, 119280), (-1, 320), (1, 179), (8, 6), (2, 5)]
v_nz5a     match 92115/120000 (76.763%) [(0, 92115), (1, 21762), (-1, 5850), (-29, 11), (29, 10)]
v_py5      match 119892/120000 (99.910%) [(0, 119892), (-1, 108)]
v_py6      match 98670/120000 (82.225%) [(0, 98670), (-1, 16801), (1, 4529)]
v_py5d     match 119150/120000 (99.292%) [(0, 119150), (1, 445), (-1, 405)]
v_py5r     match 119150/120000 (99.292%) [(0, 119150), (1, 445), (-1, 405)]
v_md5      match 119824/120000 (99.853%) [(0, 119824), (-254, 121), (-1, 22), (-249, 8), (1, 8)]
== flat-surface colour formula
flat cases 368, exact 279 (75.8%), got-predicted: {0: 279, 1: 50, -1: 39}
== stereo (stereo_check.py MONO STEREO MODE SHIFT [CROP])
t1_ry0       match 119639/120000 (99.699%) mismatch (got,predicted): [((128, 0), 361)]
t2_ry0       match 119844/120000 (99.870%) mismatch (got,predicted): [((240, 0), 156)]
st1          match 119865/120000 (99.888%) mismatch (got,predicted): [((128, 0), 135)]
st2          match 119768/120000 (99.807%) mismatch (got,predicted): [((240, 0), 232)]
t1_pl        match 120000/120000 (100.000%) mismatch (got,predicted): []
t2_pl        match 120000/120000 (100.000%) mismatch (got,predicted): []
t1_pl_i5     match 120000/120000 (100.000%) mismatch (got,predicted): []
t2_pl_i5     match 120000/120000 (100.000%) mismatch (got,predicted): []
t1_pl_i3     match 120000/120000 (100.000%) mismatch (got,predicted): []
t1_pl_i10    match 120000/120000 (100.000%) mismatch (got,predicted): []
t1_pl_c1     match 120000/120000 (100.000%) mismatch (got,predicted): []
t1_pl_c5     match 120000/120000 (100.000%) mismatch (got,predicted): []
t1_pl_cm5    match 120000/120000 (100.000%) mismatch (got,predicted): []
t1_pl_i5c5   match 120000/120000 (100.000%) mismatch (got,predicted): []
t1_crop1     match 120000/120000 (100.000%) mismatch (got,predicted): []
t1_crop2     match 120000/120000 (100.000%) mismatch (got,predicted): []
t1_crop3     match 120000/120000 (100.000%) mismatch (got,predicted): []
t1_br1       match 120000/120000 (100.000%) mismatch (got,predicted): []
t2_br1       match 120000/120000 (100.000%) mismatch (got,predicted): []
t2_br2       match 120000/120000 (100.000%) mismatch (got,predicted): []
st1f5        match 120000/120000 (100.000%) mismatch (got,predicted): []
st2f5        match 120000/120000 (100.000%) mismatch (got,predicted): []
```
