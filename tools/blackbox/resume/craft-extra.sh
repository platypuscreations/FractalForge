#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Marker tests: rows set to colour 7 show whether XFractint recomputes them.
cd "$(dirname "$0")"; mkdir -p craft2
P() { python3 -I craft.py "$@"; }
P out/ref-MN-1.gif craft2/M1.gif R=120 C=200 mark=10:7
P out/ref-MS-1.gif craft2/M2.gif R=100 C=50 f28=17 mark=50:7
P out/ref-MS-1.gif craft2/M3.gif R=100 C=50 bot=199 f28=17 mirror=x mark=50:7,240-299:7
P out/ref-MS-1.gif craft2/M3b.gif R=100 C=50 bot=199 f28=16 mirror=x mark=50:7,240-299:7
P out/ref-MN-2.gif craft2/M4.gif R=150 C=100 f32=1 after=2 mark=10:7
P out/ref-MNG-g.gif craft2/M5.gif R=100 C=0 top=0 f32=2 after=2 mark=10:7
P out/ref-MN-1.gif craft2/M6.gif R=120 C=200 top=60 mark=80:7
P out/ref-MN-1.gif craft2/M7.gif R=120 C=200 right=199 mark=10:7
P out/ref-MN-1.gif craft2/M8.gif R=120 C=200 mark=200-299:7 bot=199
P out/ref-J-1.gif craft2/M9.gif R=60 C=300 bot=239 f28=51 mirror=origin mark=50:7
