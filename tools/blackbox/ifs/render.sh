#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Renders one IFS case with XFractint in the black-box container.
# Usage: render.sh MODE OUTDIR NAME XFRACTINT-ARGS...
#   MODE = batch            run-batch (batch=yes)
#        = int:DELAY        run-interrupt: press s after DELAY seconds
#        = keys:"ACTIONS"   ../resume/run-keys.sh actions (sN sleep, kKEY key,
#                           tTEXT type), e.g. "s10 kr s3 tin.gif s1 kReturn s12 ks"
# Works in a fresh mktemp directory under OUTDIR. Copies inputs/*.ifs into it
# (unless NOINPUTS=1) plus any files listed in EXTRA, adds savename=ff.gif and
# moves the result to OUTDIR/NAME.gif (also NAME.log and NAME.stopmsg.txt).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
mode="$1" out="$2" name="$3"; shift 3
mkdir -p "$out"; out="$(cd "$out" && pwd)"
work="$(mktemp -d "$out/.work-$name.XXXX")"
chmod 777 "$work"
[ -n "${NOINPUTS:-}" ] || cp "$here"/inputs/*.ifs "$work/" 2>/dev/null || true
if [ -n "${EXTRA:-}" ]; then cp $EXTRA "$work/"; fi
case "$mode" in
  batch) docker run --rm -v "$work:/work" fractalforge-blackbox \
           "$@" savename=ff.gif >"$out/$name.log" 2>&1 </dev/null || true ;;
  int:*) docker run --rm -v "$work:/work" --entrypoint run-interrupt \
           fractalforge-blackbox "${mode#int:}" "$@" savename=ff.gif \
           >"$out/$name.log" 2>&1 </dev/null || true ;;
  keys:*) cp "$here/../resume/run-keys.sh" "$work/"
          docker run --rm -v "$work:/work" --entrypoint bash \
           fractalforge-blackbox /work/run-keys.sh "${mode#keys:}" "$@" savename=ff.gif \
           >"$out/$name.log" 2>&1 </dev/null || true
          rm -f "$work/run-keys.sh" ;;
esac
if [ -s "$work/ff.gif" ]; then mv "$work/ff.gif" "$out/$name.gif"; else echo "FAILED $name" >&2; fi
if [ -f "$work/stopmsg.txt" ]; then mv "$work/stopmsg.txt" "$out/$name.stopmsg.txt"; fi
