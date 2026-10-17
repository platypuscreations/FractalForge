#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Renders every case of a cases file (default cases.txt) into OUTDIR,
# four at a time.  Usage: run-cases.sh OUTDIR [CASESFILE]
# Lines: <name> <xfractint parameters...>  (# comments and blank lines skipped)
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="$1"; cases="${2:-$here/cases.txt}"
grep -v -E '^[[:space:]]*(#|$)' "$cases" | while read -r name args; do
  [ -s "$out/$name.gif" ] && continue
  echo "$name $args"
done | xargs -P 4 -L 1 "$here/render.sh" "$out"
