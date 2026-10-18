#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Re-scores the simulators against the evidence in corpus/xfractint/3d-lighting.
# Usage: tools/blackbox/3d-lighting/verify.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
C="$here/../../../corpus/xfractint/3d-lighting"
sim() {  # NAME INPUT ARGS...   (top view, scalexyz=100/100)
  local n="$1" i="$2"; shift 2
  printf '%-10s ' "$n"; python3 -I "$here/sim.py" "$here/inputs/$i.gif" "$C/$n.gif" "$@"
}
echo "== light fills (sim.py)"
sim e_fl5   flat100
sim e_sp5   spike   lightsource=1/1/1 ambient=0
sim e_sp5b  spike   lightsource=1/-1/1 ambient=0
sim e_sp5c  spike   lightsource=-1/1/1 ambient=0
sim e_sp5d  spike   lightsource=-1/-1/1 ambient=0
sim e_sp6   spike   filltype=6 lightsource=1/1/1 ambient=0
sim e_sx5   stepx   lightsource=1/0/1 ambient=0
sim e_sy5   stepy   lightsource=0/1/1 ambient=0
sim e_sx1   sx1     lightsource=1/0/1 ambient=0
sim x_r3    stepx   lightsource=1/0/1 ambient=0 scalexyz=100/100/3
sim y_r3    stepy   lightsource=0/1/1 ambient=0 scalexyz=100/100/3
sim p_r3    spike   lightsource=1/1/1 ambient=0 scalexyz=100/100/3
sim x_sm    stepx   lightsource=1/0/1 ambient=0 smoothing=3
sim sm1     stepx   lightsource=1/0/1 ambient=0 smoothing=1
sim sm2     stepx   lightsource=1/0/1 ambient=0 smoothing=2
sim sm5     stepx   lightsource=1/0/1 ambient=0 smoothing=5
sim smy3    stepy   lightsource=0/1/1 ambient=0 smoothing=3
sim n_r60   sx1     lightsource=1/0/1 ambient=0 scalexyz=100/100/60
sim n_y1    sy1     lightsource=0/1/1 ambient=0
sim q_f6a   flat100 filltype=6 lightsource=1/0/1 ambient=0
sim q_f6c   flat100 filltype=6 lightsource=1/0/1 ambient=0 scalexyz=100/100/100
sim q_f6e   sx1     filltype=6 lightsource=1/0/1 ambient=0
sim rp8     flat100 ambient=10 lightsource=100/0/10
sim v_nz5   noise   roughness=3 lightsource=2/-1/3 ambient=0
sim v_nz5s  noise   roughness=3 lightsource=-1/2/2 ambient=0 smoothing=2
sim v_nz6   noise   filltype=6 roughness=5 lightsource=1/1/1 ambient=0
sim v_nz5a  noise   roughness=3 lightsource=2/-1/3 ambient=35
sim v_py5   pyramid lightsource=1/-1/1 ambient=0
sim v_py6   pyramid filltype=6 lightsource=1/-1/1 ambient=0
sim v_py5d  pyramid
sim v_py5r  pyramid
sim v_md5   mandel  roughness=10 ambient=0
echo "== flat-surface colour formula"
python3 -I "$here/score_flat.py" "$C"
echo "== stereo (stereo_check.py MONO STEREO MODE SHIFT [CROP])"
st() { printf '%-12s ' "$2"; python3 -I "$here/stereo_check.py" "$C/$1.gif" "$C/$2.gif" "${@:3}"; }
st t0_ry0 t1_ry0 1 0
st t0_ry0 t2_ry0 2 0
st st0 st1 1 0
st st0 st2 2 0
st t3_pl t1_pl 1 0
st t3_pl t2_pl 2 0
st t3_pl t1_pl_i5 1 10
st t3_pl t2_pl_i5 2 10
st t3_pl t1_pl_i3 1 6
st t3_pl t1_pl_i10 1 20
st t3_pl t1_pl_c1 1 4
st t3_pl t1_pl_c5 1 20
st t3_pl t1_pl_cm5 1 -20
st t3_pl t1_pl_i5c5 1 30
st t3_pl t1_crop1 1 0 10/0/0/10
st t3_pl t1_crop2 1 0 0/10/10/0
st t3_pl t1_crop3 1 0 0/0/0/0
st t3_pl t1_br1 1 0
st t3_pl t2_br1 2 0
st t3_pl t2_br2 2 0
st st3f5 st1f5 1 0
st st3f5 st2f5 2 0
