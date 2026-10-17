#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Writes a drive.sh steps file: wait for the image, press "s" to save it as a
# GIF, open the "save current parameters" screen (b key), fill in the file
# name /work/out.par, the entry name, a comment, the "record colors" choice,
# optionally the number of colours and the maximum line length, then Enter.
#   makepar-steps.sh WAIT ENTRY COLORS [NCOLORS [LINELEN]] > steps.txt
# COLORS is keep (leave XFractint's suggestion), no, yes, only or @file.map.
wait="$1" entry="$2" colors="$3" ncolors="${4:-}" linelen="${5:-}"
cat <<STEPS
sleep $wait
key s
waitfile /work/ff.gif
sleep 1
key b
sleep 1
type /work/out.par
key Down
type $entry
key Down
type our comment one
key Down Down Down Down
STEPS
case "$colors" in
  keep) ;;
  @*) printf 'key at\ntype %s\n' "${colors#@}" ;;   # xdotool type drops a leading @
  *) printf 'type %s\n' "$colors" ;;
esac
printf 'key Down\n'
[ -n "$ncolors" ] && printf 'type %s\n' "$ncolors"
printf 'key Down\n'
[ -n "$linelen" ] && printf 'type %s\n' "$linelen"
cat <<STEPS
shot form
key Return
waitfile /work/out.par
sleep 1
shot after
STEPS
