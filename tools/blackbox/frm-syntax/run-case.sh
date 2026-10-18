#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Runs one black-box case for docs/specs/frm-syntax.md.
#   run-case.sh OUTDIR NAME XFRACTINT-ARGS...
# Copies inputs/* (our own test files) into a fresh mktemp work directory,
# runs XFractint in batch mode there with
#   type=formula passes=1 periodicity=0 viewwindows=2 map=uniq.map
# prepended and savename=ff.gif appended, then moves every file the run created
# to OUTDIR/NAME.<file> and the console output to OUTDIR/NAME.log.
# IMG=native uses the native-architecture image ff-parmap-native (same Debian
# package, built by tools/blackbox/par-map/docker) for faster exploration;
# the default is the amd64 reference image fractalforge-blackbox.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="$1" name="$2"; shift 2
mkdir -p "$out"
work="$(mktemp -d "$out/.work-$name.XXXX")"
chmod 777 "$work"
cp "$here"/inputs/* "$work"/
cname="fffs-$name-$$"
img=(fractalforge-blackbox)
if [ "${IMG:-}" = native ]; then
  img=(-v "$here/../run-batch.sh:/usr/local/bin/run-batch:ro" --entrypoint /usr/local/bin/run-batch ff-parmap-native)
fi
docker run --rm --name "$cname" -v "$work:/work" \
  "${img[@]}" type=formula passes=1 periodicity=0 viewwindows=2 map=uniq.map "$@" savename=ff.gif \
  >"$out/$name.log" 2>&1 </dev/null &
pid=$!
# Watchdog: some formulas never finish; remove the container after CASE_TIMEOUT s.
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
