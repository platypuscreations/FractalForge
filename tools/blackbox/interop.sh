#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Checks that XFractint reads GIFs FractalForge writes. Takes a few minutes
# under emulation. Results go to tools/blackbox/out/interop-*.
#
#   A. XFractint batch-reloads a finished image: it must keep every colour
#      (and re-save our parameters).
#   B. XFractint loads an unfinished image and calculates it from our
#      Fractint-format blocks: its result must match our own render to
#      within 1% of pixels (near-boundary pixels differ slightly).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
out="$here/out"
mkdir -p "$out"

cargo build -q --release -p fractalforge-cli --manifest-path "$root/Cargo.toml"
ff="$root/target/release/fractalforge-cli"
docker build -q -t fractalforge-blackbox "$here" >/dev/null

fresh() { local d; d="$(mktemp -d "$out/interop-$1.XXXX")"; chmod 777 "$d"; echo "$d"; }

echo "== A: batch reload of a finished julia"
a="$(fresh reload)"
"$ff" render --type julia --params -0.8/0.156 --maxiter 255 --passes 1 --size 800x600 -o "$a/in.gif" >/dev/null
# XFractint's exit status is unreliable; check for the saved file instead.
docker run --rm -v "$a:/work" fractalforge-blackbox filename=in.gif savename=ff.gif >"$a/log" 2>&1 || true
[ -s "$a/ff.gif" ] || { echo "XFractint saved nothing (see $a/log)"; exit 1; }
"$ff" info "$a/ff.gif" | tee "$a/info" | grep -E '^(type|params|maxiter):'
grep -q '^type: *julia' "$a/info"
grep -q '^params: *-0.8/0.156/0/0' "$a/info"
grep -q '^maxiter: *255' "$a/info"
"$ff" compare "$a/in.gif" "$a/ff.gif" --colors

echo "== B: XFractint calculates an unfinished mandel from our blocks"
b="$(fresh recalc)"
args=(--corners -0.75/-0.70/0.10/0.14 --maxiter 300 --size 800x600)
"$ff" render "${args[@]}" -o "$b/full.gif" >/dev/null
"$ff" render "${args[@]}" --stop-after 0 -o "$b/in.gif" >/dev/null 2>&1
docker run --rm -v "$b:/work" --entrypoint run-load fractalforge-blackbox 10 90 \
  filename=in.gif savename=ff.gif >"$b/log" 2>&1 || true
[ -s "$b/ff.gif" ] || { echo "XFractint saved nothing (see $b/log)"; exit 1; }
"$ff" info "$b/ff.gif" | grep -q '^status: *complete' || { echo "XFractint didn't finish in time"; exit 1; }
"$ff" compare "$b/full.gif" "$b/ff.gif" --colors --tolerance 1

echo "interop OK"
