#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Starts an interactive XFractint render under a virtual X display, waits
# DELAY seconds, then presses "s" to save the unfinished image (which carries
# resume data) and exits.
#   run-interrupt DELAY <xfractint parameters...>
# The image is saved as ff.gif (savename= should be ff.gif).
set -euo pipefail
delay="$1"; shift

export DISPLAY=:99
Xvfb "$DISPLAY" -screen 0 1024x768x24 -nolisten tcp >/tmp/xvfb.log 2>&1 &
xvfb_pid=$!
for _ in $(seq 1 100); do [ -S /tmp/.X11-unix/X99 ] && break; sleep 0.2; done

/usr/bin/xfractint "$@" >/tmp/xf.log 2>&1 &
xf_pid=$!
trap 'kill "$xf_pid" "$xvfb_pid" 2>/dev/null || true' EXIT

sleep "$delay"
win="$(xdotool search --name "^Xfractint$" | head -1 || true)"
[ -n "$win" ] || { echo "no XFractint window" >&2; cat /tmp/xf.log >&2; exit 1; }
xdotool key --window "$win" s

for _ in $(seq 1 150); do
  [ -s ff.gif ] && break
  sleep 0.2
done
sleep 2  # let the save finish writing
[ -s ff.gif ] || { echo "no image saved" >&2; exit 1; }
