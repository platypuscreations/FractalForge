#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Runs one XFractint batch case in a fresh mktemp work directory.
# Usage: run3d.sh OUTDIR NAME INPUT.gif|- XFRACTINT-ARGS...
# INPUT.gif (if not "-") is copied into the work dir as in.gif, so the
# arguments can say filename=in.gif. savename=ff.gif is appended. The result
# is moved to OUTDIR/NAME.gif; other files XFractint leaves are kept as
# OUTDIR/NAME.<file>. Set ENTRY=run-interrupt (plus its args first) to use
# another entrypoint.
set -euo pipefail
out="$1" name="$2" input="$3"; shift 3
mkdir -p "$out"; out="$(cd "$out" && pwd)"
work="$(mktemp -d "$out/.work-$name.XXXX")"
chmod 777 "$work"
[ "$input" != - ] && cp "$input" "$work/in.gif"
ep=(); [ -n "${ENTRY:-}" ] && ep=(--entrypoint "$ENTRY")
docker run --rm -v "$work:/work" ${ep[@]+"${ep[@]}"} fractalforge-blackbox \
  "$@" savename=ff.gif >"$out/$name.log" 2>&1 </dev/null || true
if [ -s "$work/ff.gif" ]; then
  mv "$work/ff.gif" "$out/$name.gif"
else
  echo "FAILED $name (see $out/$name.log)" >&2
fi
rm -f "$work/in.gif"
for f in "$work"/*; do [ -e "$f" ] && mv "$f" "$out/$name.$(basename "$f")"; done
rmdir "$work" 2>/dev/null || true
