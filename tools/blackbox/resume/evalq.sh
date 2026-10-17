#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# evalq.sh : marker report for every finished q-* (in-program load) test
cd "$(dirname "$0")"
for f in craft4/*.gif; do
  n=$(basename $f .gif); [ -f out/q-$n.gif ] || continue
  case $n in
    K-*) v=$(echo $n | cut -d- -f2); p=$(echo $n | cut -d- -f3); ref=out/ref-$v-$p.gif;;
    *) ref=$(grep "craft4/$n.gif " craft-r.sh | awk '{print $2}');;
  esac
  python3 -I marker.py out/q-$n.gif $f $ref
done
