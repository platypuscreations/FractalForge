#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Renders one fractal-type case in batch mode.
# Usage: render.sh OUTDIR NAME XFRACTINT-ARGS...
# Adds passes=1 periodicity=0 viewwindows=2 (400x300) and savename=ff.gif,
# runs in a fresh mktemp directory and moves the result to OUTDIR/NAME.gif.
set -euo pipefail
out="$1" name="$2"; shift 2
mkdir -p "$out"; out="$(cd "$out" && pwd)"
work="$(mktemp -d "$out/.work-$name.XXXX")"
chmod 777 "$work"
docker run --rm -v "$work:/work" fractalforge-blackbox \
  passes=1 periodicity=0 viewwindows=2 "$@" savename=ff.gif \
  >"$out/$name.log" 2>&1 </dev/null || true
if [ -s "$work/ff.gif" ]; then
  mv "$work/ff.gif" "$out/$name.gif"
else
  echo "FAILED $name (see $out/$name.log)" >&2
fi
if [ -f "$work/stopmsg.txt" ]; then mv "$work/stopmsg.txt" "$out/$name.stopmsg.txt"; fi
rmdir "$work" 2>/dev/null || true
