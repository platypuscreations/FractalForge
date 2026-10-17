#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Builds the black-box image and renders every case in cases-complete.txt to
# tools/blackbox/out/complete/ and every case in cases-interrupted.txt to
# tools/blackbox/out/interrupted/, as <name>.gif plus <name>.log and, when
# XFractint reports a problem, <name>.stopmsg.txt. Curate results into
# corpus/xfractint/.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
docker build -q -t fractalforge-blackbox "$here" >/dev/null

failed=0

# render_case OUTDIR NAME DOCKER-RUN-ARGS... (image, entrypoint args, XFractint args)
render_case() {
  local out="$1" name="$2"; shift 2
  mkdir -p "$out"
  # XFractint truncates save names to DOS 8.3 form and auto-increments on
  # collisions, so each case saves as ff.gif in its own fresh directory.
  local work
  work="$(mktemp -d "$out/.work-$name.XXXX")"
  chmod 777 "$work"
  # Our own formula/L-system/IFS/map files (tools/blackbox/inputs/) are made
  # available to every case, then removed again after the render.
  cp "$here"/inputs/* "$work"/
  docker run --rm -v "$work:/work" "$@" savename=ff.gif \
    >"$out/$name.log" 2>&1 </dev/null || true
  for f in "$here"/inputs/*; do rm -f "$work/$(basename "$f")"; done
  if [ -s "$work/ff.gif" ]; then
    mv "$work/ff.gif" "$out/$name.gif"
  else
    echo "   FAILED (see $out/$name.log)"
    failed=1
  fi
  if [ -f "$work/stopmsg.txt" ]; then mv "$work/stopmsg.txt" "$out/$name.stopmsg.txt"; fi
  rmdir "$work" 2>/dev/null || echo "   left extra files in $work"
}

cases() { grep -v -E '^[[:space:]]*(#|$)' "$here/$1"; }

while read -r name args; do
  echo "== $name: $args"
  # shellcheck disable=SC2086 # word-splitting of args is intended
  render_case "$here/out/complete" "$name" fractalforge-blackbox $args
done < <(cases cases-complete.txt)

while read -r name delay args; do
  echo "== $name (stop after ${delay}s): $args"
  # shellcheck disable=SC2086
  render_case "$here/out/interrupted" "$name" \
    --entrypoint run-interrupt fractalforge-blackbox "$delay" $args
done < <(cases cases-interrupted.txt)

exit "$failed"
