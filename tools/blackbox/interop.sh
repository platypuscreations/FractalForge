#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Checks that XFractint reads GIFs FractalForge writes. Takes a few minutes
# under emulation. Results go to tools/blackbox/out/interop-*.
#
#   A. XFractint batch-reloads a finished image: it must keep every colour
#      (and re-save our parameters).
#   B. XFractint loads an unfinished image with filename= and recalculates
#      it from our parameter block: its result must match our own render to
#      within 1% of pixels (near-boundary pixels differ slightly).
#   C. XFractint resumes an unfinished image of ours from our resume block
#      (in-program r load): it keeps our finished pixels and completes the
#      rest to match our own full render.
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

echo "== C: XFractint resumes an unfinished render of ours (in-program r load)"
c="$(fresh resume)"
args=(--corners -0.75/-0.70/0.10/0.14 --maxiter 8000 --size 400x300 --passes 1)
"$ff" render "${args[@]}" -o "$c/full.gif" >/dev/null
"$ff" render "${args[@]}" --stop-after 40 -o "$c/ours.gif" >/dev/null 2>&1
# Paint row 10 (finished) in colour 7: if XFractint keeps it, it resumed
# rather than recalculated (fractint-resume.md §5.3).
python3 -I - "$c/ours.gif" "$c/in.gif" "$here/resume" <<'PY'
import sys
sys.path.insert(0, sys.argv[3])
import gif
g = gif.read_gif(sys.argv[1])
w = g['w']
pixels = bytearray(g['pixels'])
pixels[10 * w:11 * w] = bytes([7]) * w
exts = [(k, g['exts'][k]) for k in ('fractint002', 'fractint001')]
gif.write_gif(sys.argv[2], w, g['h'], g['gct'], bytes(pixels), exts)
PY
cp "$here/resume/run-keys.sh" "$c/"
docker run --rm -v "$c:/work" --entrypoint bash fractalforge-blackbox /work/run-keys.sh \
  "s10 kr s3 tin.gif s1 kReturn s120 ks" type=mandel maxiter=10 viewwindows=2 savename=ff.gif \
  >"$c/log" 2>&1 || true
[ -s "$c/ff.gif" ] || { echo "XFractint saved nothing (see $c/log)"; exit 1; }
"$ff" info "$c/ff.gif" | grep -q '^status: *complete' || { echo "XFractint didn't finish"; exit 1; }
python3 -I - "$c/ff.gif" "$c/full.gif" "$here/resume" <<'PY'
import sys
sys.path.insert(0, sys.argv[3])
import gif
out, full = gif.read_gif(sys.argv[1]), gif.read_gif(sys.argv[2])
w = out['w']
assert out['pixels'][10 * w:11 * w] == bytes([7]) * w, "marker row was recomputed: no resume"
pal_o, pal_f = out['gct'], full['gct']
rgb = lambda pal, i: tuple(x >> 2 for x in pal[3 * i:3 * i + 3])
rows = range(11, out['h'])
diff = sum(rgb(pal_o, out['pixels'][y * w + x]) != rgb(pal_f, full['pixels'][y * w + x])
           for y in rows for x in range(w))
print(f"marker kept; {diff} of {len(rows) * w} other pixels differ from our full render")
assert diff <= 0.001 * len(rows) * w
PY

echo "interop OK"
