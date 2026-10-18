#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Runs every case of a cases file through render.sh, JOBS at a time.
# Usage: run-cases.sh CASES.txt OUTDIR [PATTERN]
# Case line format:  NAME INPUT.gif XFRACTINT-ARGS...   ('#' starts a comment)
# INPUT is relative to this folder's inputs/. Cases whose output exists are skipped.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
cases="$1" out="$2" pat="${3:-.}"
jobs="${JOBS:-4}"
mkdir -p "$out"
while read -r name input args; do
  [ -s "$out/$name.gif" ] && continue
  while [ "$(jobs -rp | wc -l)" -ge "$jobs" ]; do sleep 0.5; done
  # shellcheck disable=SC2086
  EXTRA="$here/inputs/uniq.map" "$here/render.sh" "$out" "$name" "$here/inputs/$input" $args &
done < <(grep -v '^\s*#' "$cases" | grep -v '^\s*$' | grep -E -- "$pat")
wait
