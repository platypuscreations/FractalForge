#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Copies the outputs of gen-all.sh (OUTDIR/<round>/), run-interrupts.sh
# (OUTDIR/interrupt/) and the bisection probes (OUTDIR/bisect/*.txt) into
# corpus/xfractint/lsystem/: NAME.ff.gif -> NAME.gif, NAME.stopmsg.txt as is,
# NAME.log only when it holds more than the qemu platform warning, the X
# server focus warnings and XFractint's closing banner.  Writes results.txt
# per round, cases.txt, verify.txt (verify.py on the corpus) and SHA256SUMS.
#   collect.sh OUTDIR
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
src="$1"
dst="$here/../../../corpus/xfractint/lsystem"
mkdir -p "$dst"
meaningful() { grep -v -e 'WARNING: The requested' -e 'Thank You' -e 'XGetInputFocus' "$1" | grep -q '[[:alnum:]]'; }
{ echo '# SPDX-License-Identifier: GPL-3.0-or-later'
  echo '# Every case: <name> [only=FILES] <arguments>. tools/blackbox/lsystem/run-case.sh runs'
  echo '#   xfractint batch=yes <arguments> savename=ff.gif'
  echo '# in a fresh directory holding the listed files of tools/blackbox/lsystem/inputs/'
  echo '# (all of them when there is no only=).'
} > "$dst/cases.txt"
for r in chars args fmt fmt2 fmt3 lhs seg rich prec sem ext mv deg deg2 arr misc; do
  mkdir -p "$dst/$r"
  for f in "$src/$r"/*; do
    b="$(basename "$f")"
    case "$b" in
      *.ff.gif) cp "$f" "$dst/$r/${b%.ff.gif}.gif" ;;
      *.stopmsg.txt) cp "$f" "$dst/$r/$b" ;;
      *.log) if meaningful "$f"; then cp "$f" "$dst/$r/$b"; fi ;;
    esac
  done
  { echo '# SPDX-License-Identifier: GPL-3.0-or-later'
    echo "# python3 -I tools/blackbox/lsystem/summarise.py on round $r (cases-$r.txt)"
    python3 -I "$here/summarise.py" "$dst/$r"; } > "$dst/$r/results.txt"
  { echo "# --- cases-$r.txt -> $r/"; grep -v '^#' "$here/cases-$r.txt" | grep -v '^[[:space:]]*$'; } >> "$dst/cases.txt"
done
mkdir -p "$dst/interrupt" "$dst/bisect"
for f in "$src/interrupt"/*; do
  b="$(basename "$f")"
  case "$b" in
    *.gif|*.keys.txt) cp "$f" "$dst/interrupt/$b" ;;
    *.log) if meaningful "$f"; then cp "$f" "$dst/interrupt/$b"; fi ;;
  esac
done
{ echo '# SPDX-License-Identifier: GPL-3.0-or-later'
  echo '# python3 -I tools/blackbox/lsystem/intsum.py interrupt sem/s-kobig-12.gif'
  python3 -I "$here/intsum.py" "$dst/interrupt" "$dst/sem/s-kobig-12.gif"; } > "$dst/interrupt/results.txt"
for f in "$src/bisect"/*.txt; do
  { echo '# SPDX-License-Identifier: GPL-3.0-or-later'; cat "$f"; } > "$dst/bisect/$(basename "$f")"
done
{ echo '# SPDX-License-Identifier: GPL-3.0-or-later'
  echo '# python3 -I tools/blackbox/lsystem/verify.py corpus/xfractint/lsystem   (MAXORDER=8)'
  python3 -I "$here/verify.py" "$dst"
  echo '# MAXORDER=200 SKIP=<orders above 12, F=FF above 13> verify.py corpus/xfractint/lsystem sem ext'
  MAXORDER=200 SKIP='s-dbl-(1[4-9]|2)|s-kobig-(1[2-9]|[2-9][0-9]|1000|40000)' python3 -I "$here/verify.py" "$dst" sem ext
} > "$dst/verify.txt"
(cd "$dst" && find . -type f ! -name SHA256SUMS | sed 's#^\./##' | LC_ALL=C sort | xargs shasum -a 256 > SHA256SUMS)
