#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Runs one case in the native-architecture image (docker/Dockerfile, tag
# ff-parmap-native), driving the interactive program with a steps file.
#   run-native.sh OUTDIR NAME STEPSFILE XFRACTINT-ARGS...
# inputs/* are copied into a fresh work directory. Every file the run creates
# (out.par, ff.gif, screenshots, xf-console.log) is moved to OUTDIR/NAME.<file>.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="$1" name="$2" steps="$3"; shift 3
mkdir -p "$out"
work="$(mktemp -d "$out/.work-$name.XXXX")"
chmod 777 "$work"
cp "$here"/inputs/* "$work"/
cp "$steps" "$work/steps.txt"
cname="ffpmn-$name-$$"
perl -e 'alarm shift; exec @ARGV' "${CASE_TIMEOUT:-120}" docker run --rm --name "$cname" \
  -e XDEPTH="${XDEPTH:-24}" -v "$work:/work" ff-parmap-native "$@" >"$out/$name.log" 2>&1 </dev/null \
  || echo "(exit status $?)" >>"$out/$name.log"
docker rm -f "$cname" >/dev/null 2>&1 || true
rm -f "$work/steps.txt"
for f in "$here"/inputs/*; do
  b="$(basename "$f")"
  if cmp -s "$f" "$work/$b"; then rm -f "$work/$b"; fi
done
for f in "$work"/*; do
  [ -e "$f" ] || continue
  mv "$f" "$out/$name.$(basename "$f")"
done
rmdir "$work" 2>/dev/null || true
