#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Copies the outputs of gen-all.sh (OUTDIR/<round>/) into the corpus folder:
# NAME.ff.gif -> NAME.gif, NAME.stopmsg.txt as is, NAME.log only when it holds
# more than the qemu platform warning and XFractint's closing banner. Then
# writes results.txt per round, cases.txt and SHA256SUMS.
#   collect.sh OUTDIR
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
src="$1"
dst="$here/../../../corpus/xfractint/frm-syntax"
mkdir -p "$dst"
{ echo '# SPDX-License-Identifier: GPL-3.0-or-later'
  echo '# Every case: <name> <arguments>. tools/blackbox/frm-syntax/run-case.sh runs'
  echo '#   xfractint batch=yes type=formula passes=1 periodicity=0 viewwindows=2 map=uniq.map <arguments> savename=ff.gif'
  echo '# in a fresh directory holding tools/blackbox/frm-syntax/inputs/*.'
} > "$dst/cases.txt"
for r in cal a p u s n i b t y r3 e r4 q v r5 r6 w; do
  mkdir -p "$dst/$r"
  for f in "$src/$r"/*; do
    b="$(basename "$f")"
    case "$b" in
      *.ff.gif) cp "$f" "$dst/$r/${b%.ff.gif}.gif" ;;
      *.stopmsg.txt) cp "$f" "$dst/$r/$b" ;;
      *.log) if grep -v -e 'WARNING: The requested' -e 'Thank You' "$f" | grep -q '[[:alnum:]]'; then cp "$f" "$dst/$r/$b"; fi ;;
    esac
  done
  { echo '# SPDX-License-Identifier: GPL-3.0-or-later'
    echo "# python3 -I tools/blackbox/frm-syntax/summarise.py on round $r (cases-$r.txt)"
    python3 -I "$here/summarise.py" "$dst/$r"; } > "$dst/$r/results.txt"
  { echo "# --- cases-$r.txt -> $r/"; grep -v '^#' "$here/cases-$r.txt" | grep -v '^[[:space:]]*$'; } >> "$dst/cases.txt"
done
(cd "$dst" && find . -type f ! -name SHA256SUMS | sed 's#^\./##' | LC_ALL=C sort | xargs shasum -a 256 > SHA256SUMS)
