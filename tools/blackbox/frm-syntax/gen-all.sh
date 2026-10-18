#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Regenerates every input and case file of docs/specs/frm-syntax.md, runs all
# cases (amd64 reference image, about 15 minutes under qemu with JOBS=10) into
# OUTDIR/<round>/, and writes OUTDIR/<round>/results.txt with summarise.py.
#   gen-all.sh OUTDIR
# collect.sh then copies the results into corpus/xfractint/frm-syntax/.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="$1"
cd "$here"
for p in p u n i b e q v w; do python3 -I gen-probe.py "probe-$p.txt" "$p"; done
python3 -I gen-struct.py; python3 -I gen-struct2.py; python3 -I gen-sym.py; python3 -I gen-r3.py
# cal, a, r4, r5 and r6 inputs and case files are written by hand.
for r in cal a p u s n i b t y r3 e r4 q v r5 r6 w; do
  JOBS="${JOBS:-10}" ./run-cases.sh "cases-$r.txt" "$out/$r" </dev/null
  python3 -I summarise.py "$out/$r" > "$out/$r/results.txt"
done
