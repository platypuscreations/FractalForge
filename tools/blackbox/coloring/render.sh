#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Renders one colouring case in batch mode.
# Usage: render.sh OUTDIR NAME XFRACTINT-ARGS...
# Adds passes=1 periodicity=0 viewwindows=2 (400x300), map=uniq.map and
# savename=ff.gif, runs in a fresh mktemp directory and moves the result to
# OUTDIR/NAME.gif.
#
# uniq.map gives 256 colours that stay distinct after 6-bit truncation. With
# the default palette, XFractint saves the FIRST palette index whose colour
# equals the computed one (index 16 and 248-255 are saved as 0, 31 as 15), so
# colour indices can only be measured with a palette that has no duplicates.
# Set FFMAP= (empty) to render with the default palette instead.
set -euo pipefail
out="$1" name="$2"; shift 2
mkdir -p "$out"
work="$(mktemp -d "$out/.work-$name.XXXX")"
chmod 777 "$work"
here="$(cd "$(dirname "$0")" && pwd)"
mapargs=()
if [ "${FFMAP-uniq.map}" != "" ]; then
  cp "$here/uniq.map" "$work/uniq.map"
  mapargs=(map=uniq.map)
fi
docker run --rm -v "$work:/work" fractalforge-blackbox \
  passes=1 periodicity=0 viewwindows=2 ${mapargs[@]+"${mapargs[@]}"} "$@" savename=ff.gif \
  >"$out/$name.log" 2>&1 </dev/null || true
if [ -s "$work/ff.gif" ]; then
  mv "$work/ff.gif" "$out/$name.gif"
else
  echo "FAILED $name (see $out/$name.log)" >&2
fi
if [ -f "$work/stopmsg.txt" ]; then mv "$work/stopmsg.txt" "$out/$name.stopmsg.txt"; fi
rm -f "$work/uniq.map"
rmdir "$work" 2>/dev/null || true
