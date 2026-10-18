#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Interactive render in the native image (tools/blackbox/par-map, tag
# ff-parmap-native), "s" to save the GIF, then the "b" screen to write the
# parameters to out.par (par-map/docker/makepar-steps.sh, colours "no").
#   run-writer.sh OUTDIR NAME XFRACTINT-ARGS...
# The files of this folder (ip.par, pf.ifs, ref.ifs, lf.l, lref.l) are copied
# into a fresh work directory; results go to OUTDIR/NAME.out.par, NAME.ff.gif.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
pm="$here/../../par-map"
out="$1" name="$2"; shift 2
mkdir -p "$out"
work="$(mktemp -d "$out/.work-$name.XXXX")"; chmod 777 "$work"
cp "$here"/ip.par "$here"/pf.ifs "$here"/ref.ifs "$here"/lf.l "$here"/lref.l "$work"/
"$pm/docker/makepar-steps.sh" "${WAIT:-8}" "$name" no >"$work/steps.txt"
perl -e 'alarm shift; exec @ARGV' 150 docker run --rm -v "$work:/work" ff-parmap-native \
  "$@" savename=ff.gif >"$out/$name.log" 2>&1 </dev/null || true
for f in out.par ff.gif xf-console.log; do [ -e "$work/$f" ] && mv "$work/$f" "$out/$name.$f"; done
