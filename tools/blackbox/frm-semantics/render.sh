#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Renders one formula-semantics case in batch mode.
# Usage: render.sh OUTDIR NAME XFRACTINT-ARGS...
# Copies sem.frm (next to this script) and ../coloring/uniq.map into a fresh
# mktemp work directory, adds
#   type=formula formulafile=sem.frm passes=1 periodicity=0 viewwindows=2
#   map=uniq.map savename=ff.gif
# in front of / after the case arguments (later arguments override earlier
# ones), and moves the result to OUTDIR/NAME.gif.
set -euo pipefail
out="$1" name="$2"; shift 2
mkdir -p "$out"
work="$(mktemp -d "$out/.work-$name.XXXX")"
chmod 777 "$work"
here="$(cd "$(dirname "$0")" && pwd)"
cp "$here/sem.frm" "$work/sem.frm"
cp "$here/../coloring/uniq.map" "$work/uniq.map"
docker run --rm -v "$work:/work" fractalforge-blackbox \
  type=formula formulafile=sem.frm passes=1 periodicity=0 viewwindows=2 \
  map=uniq.map "$@" savename=ff.gif \
  >"$out/$name.log" 2>&1 </dev/null || true
if [ -s "$work/ff.gif" ]; then
  mv "$work/ff.gif" "$out/$name.gif"
else
  echo "FAILED $name (see $out/$name.log)" >&2
fi
if [ -f "$work/stopmsg.txt" ]; then mv "$work/stopmsg.txt" "$out/$name.stopmsg.txt"; fi
rm -f "$work/uniq.map" "$work/sem.frm"
rmdir "$work" 2>/dev/null || true
