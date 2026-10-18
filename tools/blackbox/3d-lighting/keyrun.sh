#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Interactive 3D run with key presses (see keys.sh for ACTIONS syntax).
# Usage: keyrun.sh OUTDIR NAME INPUT.gif "ACTIONS" XFRACTINT-ARGS...
# Runs  xfractint 3d=yes filename=in.gif ARGS savename=ff.gif  interactively.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="$1" name="$2" input="$3" actions="$4"; shift 4
mkdir -p "$out"; out="$(cd "$out" && pwd)"
work="$(mktemp -d "$out/.work-$name.XXXX")"; chmod 777 "$work"
cp "$input" "$work/in.gif"; cp "$here/keys.sh" "$work/keys.sh"
three="3d=yes"; [ "${NO3D:-0}" = 1 ] && three=""
docker run --rm -v "$work:/work" --entrypoint bash fractalforge-blackbox \
  /work/keys.sh "$actions" $three filename=in.gif "$@" savename=ff.gif >"$out/$name.log" 2>&1 </dev/null || true
[ -s "$work/ff.gif" ] && mv "$work/ff.gif" "$out/$name.gif" || echo "FAILED $name" >&2
rm -f "$work/in.gif" "$work/keys.sh"
for f in "$work"/*; do [ -e "$f" ] && mv "$f" "$out/$name.$(basename "$f")"; done
rmdir "$work" 2>/dev/null || true
