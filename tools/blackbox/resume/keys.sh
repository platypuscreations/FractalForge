#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# keys.sh NAME IN.gif|- "ACTIONS" args...  -> out/NAME.gif
set -u
name="$1"; in="$2"; actions="$3"; shift 3
here="$(cd "$(dirname "$0")" && pwd)"
d="$(mktemp -d "$here/work.XXXXXX")"; chmod 777 "$d"
cp "$here/run-keys.sh" "$d/run-keys.sh"
extra=()
if [ "$in" != "-" ]; then cp "$in" "$d/in.gif"; chmod 666 "$d/in.gif"; [ "${NOFILEARG:-}" = 1 ] || extra=(filename=in.gif); fi
start=$(date +%s)
docker run --rm -v "$d:/work" --entrypoint bash fractalforge-blackbox /work/run-keys.sh "$actions" ${extra[@]+"${extra[@]}"} "$@" savename=ff.gif >"$d/log" 2>&1
end=$(date +%s)
if [ -s "$d/ff.gif" ]; then cp "$d/ff.gif" "$here/out/$name.gif"; echo "$name ok $((end-start))s"; else echo "$name FAIL"; tail -5 "$d/log"; fi
echo "$name keys $in [$actions] $*" >> "$here/out/cases.log"
