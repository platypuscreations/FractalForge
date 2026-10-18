#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Black-box probe: is each KEY=VALUE accepted by XFractint's command line?
# Usage: probe-keys.sh SCRATCHDIR key=value ...
# Prints "accepted"/"rejected" per argument (rejected = "couldn't understand" message).
set -u
scratch="$1"; shift
for kv in "$@"; do
  d="$(mktemp -d "$scratch/probe.XXXXXX")"
  docker run --rm -v "$d:/work" fractalforge-blackbox type=mandel viewwindows=2 maxiter=10 "$kv" savename=ff.gif >"$d/out.txt" 2>&1
  if grep -q "couldn't understand" "$d/out.txt" "$d"/stopmsg.txt 2>/dev/null; then r=rejected; else r=accepted; fi
  [ -s "$d/ff.gif" ] && g=saved || g=nosave
  printf '%-28s %s %s %s\n' "$kv" "$r" "$g" "$(tr '\n' ' ' < "$d/out.txt" | grep -v WARNING | cut -c1-80)"
done
