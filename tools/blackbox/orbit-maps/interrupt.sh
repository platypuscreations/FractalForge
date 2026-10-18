#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Interrupted render: starts XFractint interactively, presses "s" after
# DELAY seconds (image's run-interrupt entrypoint).
# Usage: interrupt.sh OUTDIR NAME DELAY XFRACTINT-ARGS...  -> OUTDIR/NAME.gif
set -euo pipefail
out="$1" name="$2" delay="$3"; shift 3
mkdir -p "$out"; out="$(cd "$out" && pwd)"
work="$(mktemp -d "$out/.work-$name.XXXX")"; chmod 777 "$work"
docker run --rm -v "$work:/work" --entrypoint run-interrupt fractalforge-blackbox \
  "$delay" viewwindows=2 "$@" savename=ff.gif >"$out/$name.log" 2>&1 </dev/null || true
if [ -s "$work/ff.gif" ]; then mv "$work/ff.gif" "$out/$name.gif"; else echo "FAILED $name" >&2; fi
rmdir "$work" 2>/dev/null || true
