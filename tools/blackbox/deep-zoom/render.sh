#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Renders one deep-zoom case in batch mode (amd64 image fractalforge-blackbox).
# Usage: render.sh OUTDIR NAME XFRACTINT-ARGS...
# Adds viewwindows=2 (400x300), map=uniq.map (distinct palette, see
# coloring/render.sh) and savename=ff.gif; runs in a fresh mktemp directory
# and moves the result to OUTDIR/NAME.gif. Files named in FFINPUTS (space
# separated paths) are copied into the work directory first (e.g. a GIF to
# load with filename=).
set -euo pipefail
out="$1" name="$2"; shift 2
mkdir -p "$out"
work="$(mktemp -d "$out/.work-$name.XXXX")"
chmod 777 "$work"
here="$(cd "$(dirname "$0")" && pwd)"
cp "$here/uniq.map" "$work/uniq.map"
for f in ${FFINPUTS-}; do cp "$f" "$work/"; done
img="${FFIMAGE:-fractalforge-blackbox}"
vw=(viewwindows=2); [ -n "${FFNOVW-}" ] && vw=()
docker run --rm -v "$work:/work" "$img" \
  ${vw[@]+"${vw[@]}"} map=uniq.map "$@" savename=ff.gif \
  >"$out/$name.log" 2>&1 </dev/null || true
if [ -s "$work/ff.gif" ]; then
  mv "$work/ff.gif" "$out/$name.gif"
else
  echo "FAILED $name (see $out/$name.log)" >&2
fi
for f in "$work"/*; do
  [ -e "$f" ] || continue
  case "$(basename "$f")" in uniq.map) rm -f "$f";; *) mv "$f" "$out/$name.$(basename "$f")";; esac
done
rmdir "$work" 2>/dev/null || true
