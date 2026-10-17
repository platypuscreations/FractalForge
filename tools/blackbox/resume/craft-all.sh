#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
cd "$(dirname "$0")"; mkdir -p craft
P() { python3 -I craft.py "$@" literal=1; }
P out/ref-MN-1.gif craft/C1.gif R=120 C=200
P out/ref-MN-1.gif craft/C2.gif R=120 C=0
P out/ref-MN-1.gif craft/C3.gif R=0 C=137
P out/ref-MS-1.gif craft/C4a.gif R=100 C=50 f28=17
P out/ref-MS-1.gif craft/C4b.gif R=100 C=50 f28=16
P out/ref-MS-1.gif craft/C4c.gif R=100 C=50 f28=0
P out/ref-MS-1.gif craft/C5.gif R=100 C=50 bot=199 f28=17 mirror=x
P out/ref-MS-1.gif craft/C6.gif R=200 C=10 f28=17
P out/ref-J-1.gif craft/C7.gif R=60 C=300 f28=51
P out/ref-J-1.gif craft/C7b.gif R=60 C=300 f28=16
P out/ref-MN-2.gif craft/C8.gif R=0 C=0 top=0 f32=1 before=2 after=2
P out/ref-MN-2.gif craft/C9.gif R=150 C=100 f32=1 after=2
P out/ref-MN-2.gif craft/C10.gif R=150 C=100 f32=1 after=0
P out/ref-MN-2.gif craft/C11.gif R=100 C=0 top=0 f32=0 before=2 after=0
P out/ref-MNG-g.gif craft/C12.gif R=0 C=0 top=0 f32=1 before=4 after=4
P out/ref-MNG-g.gif craft/C13.gif R=100 C=0 top=0 f32=2 after=2
P out/ref-MNG-g.gif craft/C14.gif R=96 C=0 top=0 f32=0 before=4 after=0
P out/ref-MSG-g.gif craft/C15.gif R=0 C=0 top=0 f32=2 before=2 after=2 f28=17
P out/ref-MN-1.gif craft/C16.gif R=120 C=0 mode=g f32=2 top=0
P out/ref-MN-1.gif craft/X1.gif R=120 C=200 f0=1
P out/ref-MN-1.gif craft/X2.gif R=120 C=200 f4=0
P out/ref-MN-1.gif craft/X3.gif R=400 C=0
P out/ref-MN-1.gif craft/X4.gif R=120 C=999
P out/ref-MN-1.gif craft/X5.gif R=120 C=200 no002=1
P out/ref-MN-1.gif craft/X6.gif R=120 C=200 status=4
P out/ref-MNG-g.gif craft/X7.gif R=100 C=0 top=0 f32=7 after=2
P out/ref-MN-1.gif craft/X8.gif R=120 C=200 right=199
P out/ref-MN-1.gif craft/X9.gif R=120 C=200 bot=199
P out/ref-MN-1.gif craft/X10.gif R=120 C=200 blank=garbage
P out/ref-MN-1.gif craft/X11.gif R=120 C=200 top=50
P out/ref-MN-1.gif craft/X12.gif R=-5 C=0 top=0
