#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Host helper: render an image interactively, open the stereogram screen
# (Ctrl+S), apply FIELDKEYS (run-keys-fb actions), press Enter, then save.
#   rds-run.sh WORKDIR RENDER_SECS RDS_SECS "FIELDKEYS" xfractint-args...
# Captures: p.xwd (parameter screen after FIELDKEYS), r.xwd (result screen).
# Output: WORKDIR/ff.gif. Inputs (rds.frm, uniq.map) are copied in.
set -u
here="$(cd "$(dirname "$0")" && pwd)"
d="$1"; r="$2"; q="$3"; keys="$4"; shift 4
cp "$here/inputs/rds.frm" "$here/../coloring/uniq.map" "$d/"
"$here/run.sh" "$d" "s$r kctrl+s s5 $keys s2 cp kReturn s$q cr ks w" savename=ff.gif "$@" >"$d/run.log" 2>&1
[ -s "$d/ff.gif" ] && echo "$d ok" || echo "$d NOSAVE"
