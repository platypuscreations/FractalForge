#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# load.sh NAME INPUT.gif D1 D2 [args...] -> load INPUT interactively, Enter after D1, s after D2
set -u
name="$1"; in="$2"; d1="$3"; d2="$4"; shift 4
here="$(cd "$(dirname "$0")" && pwd)"
d="$(mktemp -d "$here/work.XXXXXX")"; chmod 777 "$d"
cp "$in" "$d/in.gif"; chmod 666 "$d/in.gif"
start=$(date +%s)
docker run --rm -v "$d:/work" --entrypoint run-load fractalforge-blackbox "$d1" "$d2" filename=in.gif "$@" savename=ff.gif >"$d/log" 2>&1
end=$(date +%s)
if [ -s "$d/ff.gif" ]; then cp "$d/ff.gif" "$here/out/$name.gif"; echo "$name ok $((end-start))s"; else echo "$name FAIL"; tail -5 "$d/log"; fi
echo "$name load $in $d1 $d2 $*" >> "$here/out/cases.log"
