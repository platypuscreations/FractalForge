#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Inconsistent / unusual resume data, each with marker row 10 = colour 7
# (marker survives => XFractint resumed; marker gone => it recomputed from scratch).
cd "$(dirname "$0")"; mkdir -p craft3
P() { python3 -I craft.py "$@" mark=10:7; }
P out/ref-MN-1.gif craft3/Y1.gif R=120 C=200 f0=1
P out/ref-MN-1.gif craft3/Y2.gif R=120 C=200 f4=0
P out/ref-MN-1.gif craft3/Y3.gif R=400 C=0
P out/ref-MN-1.gif craft3/Y4.gif R=120 C=999
P out/ref-MN-1.gif craft3/Y5.gif R=120 C=200 no002=1
P out/ref-MN-1.gif craft3/Y6.gif R=120 C=200 status=4
P out/ref-MNG-g.gif craft3/Y7.gif R=100 C=0 top=0 f32=7 after=2
P out/ref-MN-1.gif craft3/Y10.gif R=120 C=200 blank=garbage
P out/ref-MN-1.gif craft3/Y12.gif R=-5 C=0 top=0
P out/ref-MN-1.gif craft3/Y13.gif R=120 C=200 f28=0
P out/ref-MN-1.gif craft3/Y14.gif R=120 C=200 f28=17
P out/ref-MN-1.gif craft3/Y15.gif R=120 C=200 mode=2
P out/ref-MN-1.gif craft3/Y16.gif R=120 C=200 mode=t
P out/ref-MN-1.gif craft3/Y17.gif R=120 C=200 o40=5
P out/ref-MN-2.gif craft3/Y18.gif R=150 C=100 f32=2 after=2
P out/ref-MN-1.gif craft3/Y19.gif R=120 C=200 status=1
