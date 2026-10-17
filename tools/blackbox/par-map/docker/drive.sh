#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Starts XFractint under Xvfb with the given parameters, then runs the steps
# in the file named by $STEPS (default /work/steps.txt), one per line:
#   sleep N         wait N seconds
#   key K...        xdotool key names sent to the XFractint window
#   type TEXT       xdotool type TEXT
#   shot NAME       screenshot to NAME.png
#   waitfile F      wait up to 120 s for file F to exist and be non-empty
# After the steps it kills XFractint.
set -uo pipefail
export DISPLAY=:99
Xvfb "$DISPLAY" -screen 0 1024x768x"${XDEPTH:-24}" -nolisten tcp >/tmp/xvfb.log 2>&1 &
xvfb_pid=$!
for _ in $(seq 1 100); do [ -S /tmp/.X11-unix/X99 ] && break; sleep 0.2; done
/usr/bin/xfractint "$@" >/work/xf-console.log 2>&1 &
xf_pid=$!
trap 'kill "$xf_pid" "$xvfb_pid" 2>/dev/null || true' EXIT
win=""
steps="${STEPS:-/work/steps.txt}"
getwin() { [ -n "$win" ] || win="$(xdotool search --name "^Xfractint$" | head -1 || true)"; }
while IFS= read -r line || [ -n "$line" ]; do
  cmd="${line%% *}"; arg="${line#* }"
  case "$cmd" in
    sleep) sleep "$arg" ;;
    key) getwin; xdotool key --window "$win" --delay 150 $arg ;;
    type) getwin; xdotool type --window "$win" --delay 120 "$arg" ;;
    shot) xwd -root -silent | convert xwd:- "/work/$arg.png" ;;
    waitfile) for _ in $(seq 1 600); do [ -s "$arg" ] && break; sleep 0.2; done ;;
    *) ;;
  esac
done < "$steps"
sleep 1
