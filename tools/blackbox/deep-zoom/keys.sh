#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Interactive XFractint run driven by a key script (run-keys.sh inside the
# container), in a fresh mktemp directory.
#   keys.sh OUTDIR NAME IN.gif|- "ACTIONS" XFRACTINT-ARGS...
# IN.gif (if not -) is copied in as in.gif; add filename=in.gif to the args
# for a command-line load, or use "kr s3 tin.gif s1 kReturn" in ACTIONS for
# the in-program load. ACTIONS: sN sleep, kKEY key, tTEXT type (run-keys.sh).
# uniq.map is copied in too (pass map=uniq.map to use it).
# Result: OUTDIR/NAME.gif (+ NAME.log).
set -u
out="$1" name="$2" in="$3" actions="$4"; shift 4
here="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$out"
d="$(mktemp -d "$out/.work-$name.XXXX")"; chmod 777 "$d"
cp "$here/run-keys.sh" "$here/uniq.map" "$d/"
if [ "$in" != "-" ]; then cp "$in" "$d/in.gif"; chmod 666 "$d/in.gif"; fi
start=$(date +%s)
docker run --rm -v "$d:/work" --entrypoint bash "${FFIMAGE:-fractalforge-blackbox}" /work/run-keys.sh "$actions" "$@" savename=ff.gif >"$out/$name.log" 2>&1 </dev/null
end=$(date +%s)
echo "elapsed $((end-start))s" >>"$out/$name.log"
if [ -s "$d/ff.gif" ]; then mv "$d/ff.gif" "$out/$name.gif"; echo "$name ok $((end-start))s"; else echo "$name FAIL"; fi
rm -f "$d/run-keys.sh" "$d/uniq.map" "$d/in.gif"
for f in "$d"/*; do [ -e "$f" ] && mv "$f" "$out/$name.$(basename "$f")"; done
rmdir "$d" 2>/dev/null || true
