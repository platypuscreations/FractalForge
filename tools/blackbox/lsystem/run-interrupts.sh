#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Interrupt / save / reload tests for docs/specs/lsystem.md (amd64 image).
#   run-interrupts.sh OUTDIR
# 1. int-8, int-15, int-30: interactive render of sem.l entry "kobig" at
#    order 12 (about 22-40 s under qemu), "s" pressed after 8, 15, 30 s
#    (entrypoint run-interrupt).
# 2. The int-15 save (status 3) is reloaded, as is (in.gif) and with marker
#    pixels painted by mark.py (marked.gif):
#    ld-batch      batch=yes filename=in.gif
#    ld-none       interactive filename=in.gif, no key, "s" after 30 s
#    ld-ret-nolf   interactive filename=in.gif (no lfile=), Enter, Escape, "s"
#    ld-ret        interactive filename=in.gif lfile=sem.l, Enter, "s" after 70 s
#    ld-ret-mark   the same with marked.gif
#    ld-r-mark     in-program load command r (marked.gif), "s" after 70 s
# Keys are sent with tools/blackbox/resume/run-keys.sh; every work directory
# keeps a keys.txt with the key list and arguments.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="$1"; mkdir -p "$out"
fresh() { local d; d="$(mktemp -d "$out/.work-$1.XXXX")"; chmod 777 "$d"; cp "$here/inputs/sem.l" "$here/inputs/uniq.map" "$d"/; echo "$d"; }
finish() { # finish WORKDIR NAME
  for f in "$1"/*.gif; do
    b="$(basename "$f")"; [ "$b" = in.gif ] && continue
    [ "$b" = ff.gif ] && mv "$f" "$out/$2.gif" || mv "$f" "$out/$2.$b"
  done
  [ -e "$1/keys.txt" ] && mv "$1/keys.txt" "$out/$2.keys.txt"
  [ -e "$1/run.log" ] && mv "$1/run.log" "$out/$2.log"
}
for dl in 8 15 30; do
  d="$(fresh int-$dl)"
  echo "run-interrupt $dl | type=lsystem lfile=sem.l lname=kobig params=12 viewwindows=2 map=uniq.map" > "$d/keys.txt"
  docker run --rm -v "$d:/work" --entrypoint run-interrupt fractalforge-blackbox "$dl" \
    type=lsystem lfile=sem.l lname=kobig params=12 viewwindows=2 map=uniq.map savename=ff.gif > "$d/run.log" 2>&1
  finish "$d" "int-$dl"
done
python3 -I "$here/mark.py" "$out/int-15.gif" "$out/marked.gif"
keys() { # keys NAME INPUT "KEYS" ARGS...
  local name="$1" input="$2" k="$3"; shift 3
  local d; d="$(fresh "$name")"; cp "$input" "$d/in.gif"
  echo "$k | $*" > "$d/keys.txt"
  docker run --rm -v "$d:/work" -v "$here/../resume/run-keys.sh:/work/run-keys.sh:ro" --entrypoint bash \
    fractalforge-blackbox /work/run-keys.sh "$k" "$@" savename=ff.gif > "$d/run.log" 2>&1
  finish "$d" "$name"
}
d="$(fresh ld-batch)"; cp "$out/int-15.gif" "$d/in.gif"; echo "batch | filename=in.gif" > "$d/keys.txt"
docker run --rm -v "$d:/work" fractalforge-blackbox filename=in.gif savename=ff.gif > "$d/run.log" 2>&1
finish "$d" ld-batch
keys ld-none "$out/int-15.gif" "s30 ks" filename=in.gif
keys ld-ret-nolf "$out/int-15.gif" "s10 kReturn s45 kEscape s2 ks" filename=in.gif
keys ld-ret "$out/int-15.gif" "s10 kReturn s70 ks" filename=in.gif lfile=sem.l
keys ld-ret-mark "$out/marked.gif" "s10 kReturn s70 ks" filename=in.gif lfile=sem.l
keys ld-r-mark "$out/marked.gif" "s10 kr s3 tin.gif s1 kReturn s70 ks" type=mandel maxiter=10 viewwindows=2 lfile=sem.l
