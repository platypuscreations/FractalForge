#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Loads IN.gif with XFractint's in-program load command "r" (which resumes an
# unfinished image, docs/specs/fractint-resume.md §5.2), waits DELAY seconds,
# saves with "s". Uses ../resume/run-keys.sh.
# Usage: rload.sh OUTDIR NAME IN.gif DELAY  -> OUTDIR/NAME.gif
# With ENTER=1 it instead starts with filename=in.gif and presses Enter
# (which recalculates from the parameters).
set -euo pipefail
out="$1" name="$2" in="$3" delay="$4"
here="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$out"; out="$(cd "$out" && pwd)"
work="$(mktemp -d "$out/.work-$name.XXXX")"; chmod 777 "$work"
cp "$here/../resume/run-keys.sh" "$work/run-keys.sh"
cp "$in" "$work/in.gif"; chmod 666 "$work/in.gif"
if [ "${ENTER:-0}" = 1 ]; then
  docker run --rm -v "$work:/work" --entrypoint bash fractalforge-blackbox /work/run-keys.sh \
    "s10 kReturn s$delay ks" filename=in.gif savename=ff.gif >"$out/$name.log" 2>&1 || true
else
  docker run --rm -v "$work:/work" --entrypoint bash fractalforge-blackbox /work/run-keys.sh \
    "s10 kr s3 tin.gif s1 kReturn s$delay ks" type=mandel maxiter=10 viewwindows=2 savename=ff.gif \
    >"$out/$name.log" 2>&1 || true
fi
if [ -s "$work/ff.gif" ]; then mv "$work/ff.gif" "$out/$name.gif"; else echo "FAILED $name" >&2; fi
rm -f "$work/run-keys.sh" "$work/in.gif"; rmdir "$work" 2>/dev/null || true
