#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Runs one black-box case for docs/specs/lsystem.md.
#   run-case.sh OUTDIR NAME XFRACTINT-ARGS...
# Copies inputs/* (our own test files) into a fresh mktemp work directory
# (only the listed files of inputs/ when the first argument is
# only=FILE1,FILE2,...; the empty list only= copies nothing),
# runs XFractint in batch mode there with the given arguments and
# savename=ff.gif appended, then moves every file the run created to
# OUTDIR/NAME.<file> and the console output to OUTDIR/NAME.log.
# IMG=native uses the native-architecture image ff-parmap-native (same Debian
# package, built by tools/blackbox/par-map/docker) for faster exploration;
# the default is the amd64 reference image fractalforge-blackbox.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="$1" name="$2"; shift 2
mkdir -p "$out"
work="$(mktemp -d "$out/.work-$name.XXXX")"
chmod 777 "$work"
if [ "${1#only=}" != "$1" ]; then
  IFS=, read -r -a only <<<"${1#only=}"; shift
  for f in "${only[@]}"; do [ -n "$f" ] && cp "$here/inputs/$f" "$work"/; done
else
  cp "$here"/inputs/* "$work"/
fi
cname="ffls-$name-$$"
img=(fractalforge-blackbox)
if [ "${IMG:-}" = native ]; then
  img=(-v "$here/../run-batch.sh:/usr/local/bin/run-batch:ro" --entrypoint /usr/local/bin/run-batch ff-parmap-native)
fi
docker run --rm --name "$cname" -v "$work:/work" \
  "${img[@]}" "$@" savename=ff.gif \
  >"$out/$name.log" 2>&1 </dev/null &
pid=$!
( for _ in $(seq "${CASE_TIMEOUT:-120}"); do sleep 1; kill -0 "$pid" 2>/dev/null || exit 0; done
  docker rm -f "$cname" && echo "(killed after ${CASE_TIMEOUT:-120} s)" >>"$out/$name.log" ) >/dev/null 2>&1 &
wait "$pid" || echo "(exit status $?)" >>"$out/$name.log"
docker rm -f "$cname" >/dev/null 2>&1 || true
for f in "$here"/inputs/*; do
  b="$(basename "$f")"
  if cmp -s "$f" "$work/$b"; then rm -f "$work/$b"; fi
done
for f in "$work"/*; do
  [ -e "$f" ] || continue
  mv "$f" "$out/$name.$(basename "$f")"
done
rmdir "$work" 2>/dev/null || true
