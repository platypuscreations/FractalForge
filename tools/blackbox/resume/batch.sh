#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# batch.sh NAME args...  -> runs batch render into fresh dir, copies to out/NAME.gif
set -u
name="$1"; shift
here="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$here/out"
d="$(mktemp -d "$here/work.XXXXXX")"
chmod 777 "$d"
start=$(date +%s)
docker run --rm -v "$d:/work" fractalforge-blackbox "$@" savename=ff.gif >"$d/log" 2>&1
end=$(date +%s)
if [ -s "$d/ff.gif" ]; then cp "$d/ff.gif" "$here/out/$name.gif"; echo "$name ok $((end-start))s"; else echo "$name FAIL $((end-start))s"; cat "$d/log"; fi
echo "$name batch $*" >> "$here/out/cases.log"
