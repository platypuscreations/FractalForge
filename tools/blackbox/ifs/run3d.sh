#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Renders every constant-map entry of inputs/pt3.ifs with extra XFractint
# arguments: run3d.sh OUTDIR [ARGS...]  (one batch run per 3D point, 4 at a time)
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; out="$1"; shift
for p in p000 p100 p010 p001 p200 p020 p002 p111 p321 p400 p040 p004; do
  "$here/render.sh" batch "$out" "$p" type=ifs ifsfile=pt3.ifs ifs=$p viewwindows=2 maxiter=2 "$@" &
  while [ "$(jobs -r | wc -l)" -ge 8 ]; do sleep 1; done
done
wait
