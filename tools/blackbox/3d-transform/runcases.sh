#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Runs cases from a list: each non-comment line is
#   NAME INPUT ARGS...
# where INPUT is an input GIF name in INDIR (or "-"), ARGS are XFractint
# parameters (filename=in.gif refers to INPUT). Runs JOBS cases at once.
# Usage: runcases.sh LIST INDIR OUTDIR [JOBS]
set -euo pipefail
list="$1" indir="$2" out="$3" jobs="${4:-4}"
here="$(cd "$(dirname "$0")" && pwd)"
n=0
while read -r name input args; do
  case "$name" in ''|'#'*) continue ;; esac
  [ -s "$out/$name.gif" ] && continue
  src=-; [ "$input" != - ] && src="$indir/$input"
  # shellcheck disable=SC2086
  "$here/run3d.sh" "$out" "$name" "$src" $args &
  n=$((n+1)); if [ $((n % jobs)) = 0 ]; then wait; fi
done < "$list"
wait
