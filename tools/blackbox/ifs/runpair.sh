#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Renders every entry of inputs/pair3.ifs: runpair.sh OUTDIR [ARGS...]
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; out="$1"; shift
for p in a100 a400 a010 a040 a001 a004 a111 a321 s100; do
  "$here/render.sh" batch "$out" "$p" type=ifs ifsfile=pair3.ifs ifs=$p viewwindows=2 maxiter=2 "$@" &
  while [ "$(jobs -r | wc -l)" -ge 9 ]; do sleep 1; done
done
wait
