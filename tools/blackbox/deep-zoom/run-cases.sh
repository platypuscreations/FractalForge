#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Renders every "<name> <xfractint args...>" line of CASESFILE into OUTDIR
# with render.sh, P at a time (default 6). Existing outputs are skipped.
#   run-cases.sh OUTDIR CASESFILE [P]
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="$1"; cases="$2"; p="${3:-6}"
grep -v -E '^[[:space:]]*(#|$)' "$cases" | while read -r name args; do
  [ -s "$out/$name.gif" ] && continue
  echo "$name $args"
done | xargs -P "$p" -L 1 "$here/render.sh" "$out"
