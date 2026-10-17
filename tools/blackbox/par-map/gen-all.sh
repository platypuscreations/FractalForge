#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Regenerates all .par/.map black-box evidence into OUTDIR:
#   OUTDIR/read/     amd64 batch cases (cases-*.txt except writer/special)
#   OUTDIR/written/  native interactive "b"-screen saves (cases-writer.txt)
#   OUTDIR/rt/       amd64 batch loads of those written files
#   OUTDIR/special/  other interactive runs (cases-special.txt) and
#                    maps-dir-check.sh
# Builds both images first. Takes roughly 10 minutes on Apple Silicon.
#   gen-all.sh OUTDIR
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="$1"
docker build -q -t fractalforge-blackbox "$here/.." >/dev/null
docker build -q -t ff-parmap-native "$here/docker" >/dev/null
for c in structure structure2 structure3 structure4 values keys colors paths map misc reset; do
  "$here/run-cases.sh" "$here/cases-$c.txt" "$out/read"
done
"$here/run-writer-cases.sh" "$here/cases-writer.txt" "$out/written" 6
mkdir -p "$out/rtin"
for f in "$out"/written/*.out.par; do
  n="$(basename "$f" .out.par)"; cp "$f" "$out/rtin/$n.par"
done
EXTRA_INPUTS="$out/rtin" "$here/run-cases.sh" "$here/cases-roundtrip.txt" "$out/rt"
grep -v -E '^[[:space:]]*(#|$)' "$here/cases-special.txt" | while read -r name steps depth args; do
  # shellcheck disable=SC2086 # word-splitting of args is intended
  XDEPTH="$depth" "$here/run-native.sh" "$out/special" "$name" "$here/steps/$steps.txt" $args
done
"$here/maps-dir-check.sh" "$out/special"
