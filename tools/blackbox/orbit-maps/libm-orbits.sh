#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Computes, inside the black-box container (glibc libm, IEEE doubles), the
# orbits that verify.py needs for the trigonometric single-orbit types.
# Usage: libm-orbits.sh CORPUS-DIR OUT-DIR
# Writes OUT-DIR/NAME.bin (start point + 1024*maxiter iterates, raw doubles)
# for every martin/chip/quadruptwo/threeply GIF in CORPUS-DIR.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
corpus="$(cd "$1" && pwd)"; mkdir -p "$2"; out="$(cd "$2" && pwd)"
for f in "$corpus"/*.gif; do
  name="$(basename "$f" .gif)"
  info="$(python3 -I "$here/typeparams.py" "$f")"
  read -r t mi p1 p2 p3 <<<"$info"
  [ "$t" = "-" ] && continue
  docker run --rm -v "$here:/w" --entrypoint perl fractalforge-blackbox \
    /w/libm-orbit.pl "$t" $((1024 * mi)) 0 0 "$p1" "$p2" "$p3" 2>/dev/null >"$out/$name.bin"
  echo "$name $t"
done
