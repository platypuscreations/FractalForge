#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Loads a GIF in interactive XFractint, presses Enter after DELAY1 seconds
# (which calculates or resumes the image), then presses "s" after DELAY2
# more seconds to save it.
#   run-load DELAY1 DELAY2 filename=in.gif savename=ff.gif [more parameters]
set -euo pipefail
delay1="$1" delay2="$2"; shift 2

export DISPLAY=:99
Xvfb "$DISPLAY" -screen 0 1024x768x24 -nolisten tcp >/tmp/xvfb.log 2>&1 &
xvfb_pid=$!
for _ in $(seq 1 100); do [ -S /tmp/.X11-unix/X99 ] && break; sleep 0.2; done

/usr/bin/xfractint "$@" >/tmp/xf.log 2>&1 &
xf_pid=$!
trap 'kill "$xf_pid" "$xvfb_pid" 2>/dev/null || true' EXIT

sleep "$delay1"
win="$(xdotool search --name "^Xfractint$" | head -1 || true)"
[ -n "$win" ] || { echo "no XFractint window" >&2; cat /tmp/xf.log >&2; exit 1; }
xdotool key --window "$win" Return
sleep "$delay2"
xdotool key --window "$win" s

for _ in $(seq 1 150); do
  [ -s ff.gif ] && break
  sleep 0.2
done
sleep 2  # let the save finish writing
[ -s ff.gif ] || { echo "no image saved" >&2; exit 1; }
