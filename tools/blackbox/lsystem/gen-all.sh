#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Regenerates every input and case file and runs all batch rounds for
# docs/specs/lsystem.md on the amd64 reference image:
#   gen-all.sh OUTDIR      (results in OUTDIR/<round>/)
# JOBS (default 10) cases run in parallel; CASE_TIMEOUT (default 300 s)
# stops runs that do not finish (the very high orders in cases-sem.txt).
# The interrupt/reload tests and the bisection probes are separate:
# run-interrupts.sh, bisect-prec.py, bisect-origin.py.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="$1"
for g in gen-chars gen-args gen-fmt gen-fmt2 gen-fmt3 gen-lhs gen-seg gen-rich gen-prec gen-sem gen-ext gen-turtle gen-misc; do
  python3 -I "$here/$g.py"
done
for r in chars args fmt fmt2 fmt3 lhs seg rich prec sem ext mv deg deg2 arr misc; do
  mkdir -p "$out/$r"
  JOBS="${JOBS:-10}" CASE_TIMEOUT="${CASE_TIMEOUT:-300}" "$here/run-cases.sh" "$here/cases-$r.txt" "$out/$r"
done
