#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Runs XFractint in batch mode under a virtual X display.
# Arguments are passed straight through as XFractint command-line parameters,
# e.g.  run-batch type=mandel maxiter=500 savename=out.gif
#
# Xvfb is started directly (not via xvfb-run) because xvfb-run's signal-based
# readiness check hangs under qemu amd64 emulation on Apple Silicon hosts.
set -euo pipefail

export DISPLAY=:99
Xvfb "$DISPLAY" -screen 0 1024x768x24 -nolisten tcp >/tmp/xvfb.log 2>&1 &
xvfb_pid=$!
trap 'kill "$xvfb_pid" 2>/dev/null || true' EXIT

for _ in $(seq 1 100); do
  [ -S /tmp/.X11-unix/X99 ] && break
  sleep 0.2
done
[ -S /tmp/.X11-unix/X99 ] || { echo "Xvfb did not start" >&2; cat /tmp/xvfb.log >&2; exit 1; }

/usr/bin/xfractint batch=yes "$@"
