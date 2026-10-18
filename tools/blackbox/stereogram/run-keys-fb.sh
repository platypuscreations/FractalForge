#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Runs inside the black-box container (mounted at /work):
#   bash /work/run-keys-fb.sh "ACTIONS" <xfractint parameters...>
# Like tools/blackbox/resume/run-keys.sh, plus screen captures:
#   sN sleep N s, kKEY xdotool key, tTEXT xdotool type, cNAME copy the Xvfb
#   framebuffer (XWD format) to /work/NAME.xwd, w wait for ff.gif.
set -uo pipefail
actions="$1"; shift
export DISPLAY=:99
mkdir -p /tmp/fb
Xvfb "$DISPLAY" -screen 0 1024x768x24 -fbdir /tmp/fb -nolisten tcp >/tmp/xvfb.log 2>&1 &
xvfb_pid=$!
for _ in $(seq 1 100); do [ -S /tmp/.X11-unix/X99 ] && break; sleep 0.2; done
/usr/bin/xfractint "$@" >/tmp/xf.log 2>&1 &
xf_pid=$!
trap 'kill "$xf_pid" "$xvfb_pid" 2>/dev/null || true' EXIT
win() { xdotool search --name "^Xfractint$" | head -1 || true; }
for a in $actions; do
  case "$a" in
    s*) sleep "${a#s}";;
    t*) xdotool type --window "$(win)" --delay 100 "${a#t}";;
    k*) w="$(win)"; [ -n "$w" ] || { echo "no XFractint window" >&2; cat /tmp/xf.log >&2; exit 1; }
        xdotool key --window "$w" "${a#k}";;
    c*) cp /tmp/fb/Xvfb_screen0 "/work/${a#c}.xwd";;
    w)  for _ in $(seq 1 300); do [ -s ff.gif ] && break; sleep 0.2; done; sleep 2;;
  esac
done
cp /tmp/xf.log /work/xf.log 2>/dev/null || true
