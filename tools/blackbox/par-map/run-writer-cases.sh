#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# For each line "<name> <colors> <ncolors|-> <linelen|-> <xfractint args...>"
# of a cases file, renders interactively in the native image, saves the GIF
# (s key) and then the parameters (b key) to out.par, using
# docker/makepar-steps.sh. Results land in OUTDIR/<name>.ff.gif, .out.par.
#   run-writer-cases.sh CASES.txt OUTDIR [WAIT]
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
wait="${3:-4}"
n=0
while read -r name colors ncol llen args; do
  [ "$ncol" = - ] && ncol=""
  [ "$llen" = - ] && llen=""
  steps="$(mktemp)"
  "$here/docker/makepar-steps.sh" "$wait" "$name" "$colors" "$ncol" "$llen" >"$steps"
  # shellcheck disable=SC2086 # word-splitting of args is intended
  ( "$here/run-native.sh" "$2" "$name" "$steps" $args savename=ff.gif; rm -f "$steps" ) &
  n=$((n + 1))
  if [ $((n % 4)) -eq 0 ]; then wait; fi
done < <(grep -v -E '^[[:space:]]*(#|$)' "$1")
wait
