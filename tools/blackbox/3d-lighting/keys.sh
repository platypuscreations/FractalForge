#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Runs inside the black-box container (mounted at /work):
#   bash /work/keys.sh "ACTIONS" <xfractint parameters...>
# ACTIONS is a space-separated list of  sN (sleep N seconds), kKEY (xdotool key KEY)
# tTEXT (xdotool type TEXT), w (list window names and work files) and
# p (copy the virtual screen to /work/shotN.xwd),
# e.g. "s10 kReturn s60 ks". Waits for ff.gif after the last action.
set -euo pipefail
actions="$1"; shift
export DISPLAY=:99
mkdir -p /tmp/fb; Xvfb "$DISPLAY" -screen 0 1024x768x24 -fbdir /tmp/fb -nolisten tcp >/tmp/xvfb.log 2>&1 &
xvfb_pid=$!
for _ in $(seq 1 100); do [ -S /tmp/.X11-unix/X99 ] && break; sleep 0.2; done
/usr/bin/xfractint "$@" >/tmp/xf.log 2>&1 &
xf_pid=$!
trap 'kill "$xf_pid" "$xvfb_pid" 2>/dev/null || true' EXIT
for a in $actions; do
  case "$a" in
    s*) sleep "${a#s}";;
    p)  n=$(( ${n:-0} + 1 )); cp /tmp/fb/Xvfb_screen0 "/work/shot$n.xwd";;
    w)  for id in $(xdotool search --name '.' 2>/dev/null); do echo "win $id: $(xdotool getwindowname "$id" 2>/dev/null)"; done; ls -la /work;;
    t*) win="$(xdotool search --name "^Xfractint$" | head -1 || true)"
        xdotool type --window "$win" --delay 100 "${a#t}";;
    k*) win="$(xdotool search --name "^Xfractint$" | head -1 || true)"
        [ -n "$win" ] || { echo "no XFractint window" >&2; cat /tmp/xf.log >&2; exit 1; }
        xdotool key --window "$win" "${a#k}";;
  esac
done
for _ in $(seq 1 150); do [ -s ff.gif ] && break; sleep 0.2; done
sleep 2
[ -s ff.gif ] || { echo "no image saved" >&2; exit 1; }
