#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Runs every case of a cases file (lines "<name> <args...>", # comments)
# with run-case.sh, JOBS at a time (default 4).
#   run-cases.sh CASESFILE OUTDIR
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
cases="$1" out="$2"
grep -v '^[[:space:]]*#' "$cases" | grep -v '^[[:space:]]*$' |
  xargs -P "${JOBS:-4}" -L 1 "$here/run-case.sh" "$out"
