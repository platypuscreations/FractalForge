#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# intr.sh NAME DELAY args...  -> interrupted interactive render to out/NAME.gif
set -u
name="$1"; delay="$2"; shift 2
here="$(cd "$(dirname "$0")" && pwd)"
d="$(mktemp -d "$here/work.XXXXXX")"; chmod 777 "$d"
start=$(date +%s)
docker run --rm -v "$d:/work" --entrypoint run-interrupt fractalforge-blackbox "$delay" "$@" savename=ff.gif >"$d/log" 2>&1
end=$(date +%s)
if [ -s "$d/ff.gif" ]; then cp "$d/ff.gif" "$here/out/$name.gif"; echo "$name ok $((end-start))s"; else echo "$name FAIL"; tail -5 "$d/log"; fi
echo "$name interrupt $delay $*" >> "$here/out/cases.log"
