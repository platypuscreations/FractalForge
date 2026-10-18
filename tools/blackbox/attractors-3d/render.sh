#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Renders one attractor case in batch mode.
# Usage: render.sh OUTDIR NAME XFRACTINT-ARGS...
# Adds viewwindows=2 (400x300) unless NOVW=1 or the first argument is the
# token FULL (800x600), and savename=ff.gif; runs in a fresh mktemp directory
# and moves the result to OUTDIR/NAME.gif.
set -euo pipefail
out="$1" name="$2"; shift 2
mkdir -p "$out"; out="$(cd "$out" && pwd)"
work="$(mktemp -d "$out/.work-$name.XXXX")"
chmod 777 "$work"
vw="viewwindows=2"; [ "${NOVW:-0}" = 1 ] && vw=""
if [ "${1:-}" = FULL ]; then vw=""; shift; fi
docker run --rm -v "$work:/work" fractalforge-blackbox \
  $vw "$@" savename=ff.gif >"$out/$name.log" 2>&1 </dev/null || true
if [ -s "$work/ff.gif" ]; then
  mv "$work/ff.gif" "$out/$name.gif"
else
  echo "FAILED $name (see $out/$name.log)" >&2
fi
for f in "$work"/*; do [ -e "$f" ] && mv "$f" "$out/$name.$(basename "$f")"; done
rmdir "$work" 2>/dev/null || true
