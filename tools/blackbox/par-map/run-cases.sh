#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Runs every line "<name> <xfractint parameters...>" of a cases file through
# run-case.sh (amd64 batch image), four at a time.
#   run-cases.sh CASES.txt OUTDIR
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
n=0
while read -r name args; do
  # shellcheck disable=SC2086 # word-splitting of args is intended
  "$here/run-case.sh" "$2" "$name" $args &
  n=$((n + 1))
  if [ $((n % 4)) -eq 0 ]; then wait; fi
done < <(grep -v -E '^[[:space:]]*(#|$)' "$1")
wait
