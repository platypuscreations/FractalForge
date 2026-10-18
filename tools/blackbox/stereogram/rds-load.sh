#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Host helper: load DEPTH.gif interactively (filename=), open the stereogram
# screen (Ctrl+S), apply FIELDKEYS (run-keys-fb actions), press Enter, save.
#   rds-load.sh WORKDIR DEPTH.gif LOAD_SECS RDS_SECS "FIELDKEYS" [xfractint-args...]
# Captures l.xwd (after load), p.xwd (parameter screen), r.xwd (result).
# Output: WORKDIR/ff.gif; also WORKDIR/time.txt (host Unix time around Enter).
set -u
here="$(cd "$(dirname "$0")" && pwd)"
d="$1"; src="$2"; l="$3"; q="$4"; keys="$5"; shift 5
cp "$src" "$d/depth.gif"; cp "$here/inputs/rds.frm" "$here/../coloring/uniq.map" "$d/"
date +%s > "$d/time.txt"
"$here/run.sh" "$d" "s$l cl kctrl+s s5 $keys s2 cp kReturn s$q cr ks w" filename=depth.gif savename=ff.gif "$@" >"$d/run.log" 2>&1
date +%s >> "$d/time.txt"
[ -s "$d/ff.gif" ] && echo "$d ok" || echo "$d NOSAVE"
