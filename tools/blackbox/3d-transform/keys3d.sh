#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Runs XFractint interactively with a key script (see run-keys.sh, a copy of
# tools/blackbox/resume/run-keys.sh) in a fresh mktemp work directory.
# Usage: keys3d.sh OUTDIR NAME INPUT.gif "ACTIONS" XFRACTINT-ARGS...
set -euo pipefail
out="$1" name="$2" input="$3" actions="$4"; shift 4
here="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$out"; out="$(cd "$out" && pwd)"
work="$(mktemp -d "$out/.work-$name.XXXX")"; chmod 777 "$work"
cp "$input" "$work/in.gif"; cp "$here/run-keys.sh" "$work/run-keys.sh"
docker run --rm -v "$work:/work" --entrypoint bash fractalforge-blackbox \
  /work/run-keys.sh "$actions" "$@" savename=ff.gif >"$out/$name.log" 2>&1 </dev/null || true
if [ -s "$work/ff.gif" ]; then mv "$work/ff.gif" "$out/$name.gif"; else echo "FAILED $name" >&2; fi
rm -f "$work/in.gif" "$work/run-keys.sh"
for f in "$work"/*; do [ -e "$f" ] && mv "$f" "$out/$name.$(basename "$f")"; done
rmdir "$work" 2>/dev/null || true
