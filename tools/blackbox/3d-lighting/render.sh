#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Renders one 3D-lighting case in batch mode.
# Usage: render.sh OUTDIR NAME INPUT.gif XFRACTINT-ARGS...
# Copies INPUT.gif into a fresh mktemp work directory as in.gif, runs
#   xfractint batch=yes 3d=yes filename=in.gif ARGS savename=ff.gif
# and moves the result to OUTDIR/NAME.gif (other files to OUTDIR/NAME.*).
# Set NO3D=1 to omit 3d=yes.  Extra files to copy: EXTRA="a b".
set -euo pipefail
out="$1" name="$2" input="$3"; shift 3
mkdir -p "$out"; out="$(cd "$out" && pwd)"
work="$(mktemp -d "$out/.work-$name.XXXX")"
chmod 777 "$work"
cp "$input" "$work/in.gif"
for e in ${EXTRA:-}; do cp "$e" "$work/"; done
three="3d=yes"; [ "${NO3D:-0}" = 1 ] && three=""
docker run --rm -v "$work:/work" fractalforge-blackbox \
  $three filename=in.gif "$@" savename=ff.gif >"$out/$name.log" 2>&1 </dev/null || true
if [ -s "$work/ff.gif" ]; then
  mv "$work/ff.gif" "$out/$name.gif"
else
  echo "FAILED $name (see $out/$name.log)" >&2
fi
rm -f "$work/in.gif"
for e in ${EXTRA:-}; do rm -f "$work/$(basename "$e")"; done
for f in "$work"/*; do [ -e "$f" ] && mv "$f" "$out/$name.$(basename "$f")"; done
rmdir "$work" 2>/dev/null || true
