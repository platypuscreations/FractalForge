#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Renders every case of a cases file (default: corpus/xfractint/attractors-3d/cases.txt).
# Usage: gen.sh OUTDIR [CASES-FILE] [JOBS]
# Each non-comment line is "NAME ARGS..."; existing OUTDIR/NAME.gif are skipped.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="$1"
cases="${2:-$here/../../../corpus/xfractint/attractors-3d/cases.txt}"
jobs="${3:-4}"
grep -v '^\s*#' "$cases" | grep -v '^\s*$' | while read -r name args; do
  [ -s "$out/$name.gif" ] && continue
  echo "$name $args"
done | xargs -P "$jobs" -L 1 sh -c 'n="$0"; "'"$here"'/render.sh" "'"$out"'" "$n" "$@"'
