#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Loads every crafted file of INDIR with filename= on the command line,
# presses Enter after 8 s (XFractint recalculates from the file's
# parameters) and s after WAIT more seconds.  P runs at a time.
#   load-all.sh INDIR OUTDIR [WAIT] [P]
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
in="$1" out="$2" wait="${3:-150}" p="${4:-5}"
ls "$in"/*.gif | while read -r f; do
  n="$(basename "$f" .gif)"
  [ -s "$out/$n.gif" ] && continue
  echo "$f"
done | xargs -P "$p" -I{} sh -c 'f="{}"; n="$(basename "$f" .gif)"; "$0" "$1" "$n" "$f" "s8 kReturn s$2 ks" viewwindows=2 filename=in.gif' "$here/keys.sh" "$out" "$wait"
