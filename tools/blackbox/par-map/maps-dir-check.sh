#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Checks where map= inside a .par entry is looked up: copies our own
# inputs/mfull.map into the program's maps directory under a new name
# (writing only; nothing there is read by us), points a .par entry at that
# name and renders in batch mode. Result: OUTDIR/maps-dir-check.ff.gif.
#   maps-dir-check.sh OUTDIR
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="$1"; mkdir -p "$out"
work="$(mktemp -d "$out/.work-mapsdir.XXXX")"; chmod 777 "$work"
cp "$here/inputs/mfull.map" "$work/ffcopy.map"
cat >"$work/md.par" <<'PAR'
; SPDX-License-Identifier: GPL-3.0-or-later
maprel {
  type=julia params=-0.8/0.156 maxiter=200 corners=-1.5/1.5/-1.125/1.125 map=ffmapcp.map
}
PAR
docker run --rm -u 0 -v "$work:/work" --entrypoint bash fractalforge-blackbox -c \
  'mv /work/ffcopy.map /usr/share/xfractint/maps/ffmapcp.map && run-batch @md.par/maprel viewwindows=2 savename=ff.gif' \
  >"$out/maps-dir-check.log" 2>&1
mv "$work/ff.gif" "$out/maps-dir-check.ff.gif" 2>/dev/null || echo "no image" >>"$out/maps-dir-check.log"
[ -f "$work/stopmsg.txt" ] && mv "$work/stopmsg.txt" "$out/maps-dir-check.stopmsg.txt"
rm -f "$work/md.par"; rmdir "$work" 2>/dev/null || true
