#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Interrupt / resume evidence for docs/specs/attractors-3d.md.
# Usage: intr.sh OUTDIR
#   NAME-int.gif    interactive render, "s" pressed after 12 s (run-interrupt)
#   NAME-rload.gif  NAME-int.gif loaded with the in-program load command "r"
#                   (fractint-resume.md §5.2), "s" pressed 8 s later
# Each run uses a fresh mktemp directory; results are copied to OUTDIR.
set -euo pipefail
out="$1"; mkdir -p "$out"; out="$(cd "$out" && pwd)"
here="$(cd "$(dirname "$0")" && pwd)"
keys="$here/../resume/run-keys.sh"
run_int() {  # NAME ARGS...
  local name="$1"; shift
  local w; w="$(mktemp -d "$out/.int-$name.XXXX")"; chmod 777 "$w"
  docker run --rm -v "$w:/work" --entrypoint run-interrupt fractalforge-blackbox \
    12 "$@" viewwindows=2 savename=ff.gif >"$out/$name-int.log" 2>&1 </dev/null || true
  [ -s "$w/ff.gif" ] && cp "$w/ff.gif" "$out/$name-int.gif" || echo "FAILED $name-int" >&2
}
run_rload() {  # NAME
  local name="$1"
  local w; w="$(mktemp -d "$out/.rload-$name.XXXX")"; chmod 777 "$w"
  cp "$keys" "$w/run-keys.sh"; cp "$out/$name-int.gif" "$w/in.gif"; chmod 666 "$w/in.gif"
  docker run --rm -v "$w:/work" --entrypoint bash fractalforge-blackbox /work/run-keys.sh \
    "s10 kr s3 tin.gif s1 kReturn s8 ks" type=mandel maxiter=10 viewwindows=2 savename=ff.gif \
    >"$out/$name-rload.log" 2>&1 </dev/null || true
  [ -s "$w/ff.gif" ] && cp "$w/ff.gif" "$out/$name-rload.gif" || echo "FAILED $name-rload" >&2
}
run_int lorenz type=lorenz maxiter=30000
run_int lorenz3d type=lorenz3d maxiter=30000
run_int pickover type=pickover maxiter=30000
run_rload lorenz
run_rload lorenz3d
