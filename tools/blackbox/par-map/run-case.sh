#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Runs one black-box case for the .par/.map specs.
#   run-case.sh OUTDIR NAME [docker-run args: --entrypoint X] IMAGE-ARGS...
# Copies tools/blackbox/par-map/inputs/* into a fresh work directory, runs the
# container there (savename=ff.gif is appended), then moves every file the run
# created to OUTDIR/NAME.<original name> and the console log to OUTDIR/NAME.log.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="$1" name="$2"; shift 2
mkdir -p "$out"
work="$(mktemp -d "$out/.work-$name.XXXX")"
chmod 777 "$work"
cp "$here"/inputs/* "$work"/
# EXTRA_INPUTS: optional directory whose files are also copied in.
if [ -n "${EXTRA_INPUTS:-}" ]; then cp "$EXTRA_INPUTS"/* "$work"/; fi
ep=()
if [ "${1:-}" = "--entrypoint" ]; then ep=(--entrypoint "$2"); shift 2; fi
cname="ffpm-$name-$$"
perl -e 'alarm shift; exec @ARGV' "${CASE_TIMEOUT:-90}" docker run --rm --name "$cname" -v "$work:/work" ${ep[@]+"${ep[@]}"} fractalforge-blackbox "$@" savename=ff.gif \
  >"$out/$name.log" 2>&1 </dev/null || echo "(exit status $?)" >>"$out/$name.log"
docker rm -f "$cname" >/dev/null 2>&1 || true
for f in "$here"/inputs/* ${EXTRA_INPUTS:+"$EXTRA_INPUTS"/*}; do
  b="$(basename "$f")"
  if cmp -s "$f" "$work/$b"; then rm -f "$work/$b"; fi
done
for f in "$work"/* ; do
  [ -e "$f" ] || continue
  mv "$f" "$out/$name.$(basename "$f")"
done
rmdir "$work" 2>/dev/null || true
ls "$out" | grep "^$name\." | tr '\n' ' '; echo
