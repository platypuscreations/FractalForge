#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Regenerates the synthetic input GIFs used by cases.txt into DIR.
# Usage: gen-inputs.sh DIR
# mandel.gif (needed by flat5b and wavsblk) is an XFractint render:
#   run3d.sh DIR/.. mandel - type=mandel maxiter=150 viewwindows=2
# and must already be in DIR.
set -euo pipefail
d="$1"; here="$(cd "$(dirname "$0")" && pwd)"
m() { python3 -I "$here/mkin.py" "$d/$1" "${@:2}"; }
m flat5.gif 400 300 5
m flat5b.gif 400 300 5 "$d/mandel.gif"
m flat5s.gif 100 75 5
m xs.gif 100 75 'x+1';            m ys.gif 100 75 'y+1'
m xl.gif 400 300 'x%250+1';       m yl.gif 400 300 'y%250+1'
m xm.gif 200 150 'x+1';           m ym.gif 200 150 'y+1'
m xb.gif 400 300 'x//2+1'
m wav.gif 200 150 '1+int(127+120*math.sin(x/13.0)*math.cos(y/9.0))'
m wavs.gif 100 75 '1+int(127+120*math.sin(x/7.0)*math.cos(y/5.0))'
for c in 0 1 32 64 96 128 160 192 224 255; do m f$c.gif 200 150 $c; done
for c in 1 64 128 192 255; do m g$c.gif 100 75 $c; done
m grid.gif 100 75 '1+(x%10)*20+(y%10)*2'
m rx.gif 100 75 '1+2*x';          m ry.gif 100 75 '1+3*y'
m one.gif 100 75 '200 if (x==50 and y==37) else 100'
m sp.gif 100 75 '200 if (x==50 and y==37) else (60 if (x==20 and y==20) else 100)'
m lat.gif 200 150 '(1+(x//13)+15*(y//10)) if (x%13==6 and y%10==5) else 0'
m latp.gif 200 150 '(1+((x//13)+15*(y//10))*97%225) if (x%13==6 and y%10==5) else 0'
m lat100.gif 100 75 '(1+((x//7)+15*(y//7))*97%250) if (x%7==3 and y%7==3) else 0'
m x0.gif 200 150 'x'
m chk.gif 200 150 '0 if (x//20+y//15)%2 else 100'
m stripe.gif 200 150 '0 if y%2 else 100'
m stripe7.gif 200 150 '7 if y%2 else 100'
m x16.gif 200 150 'x//13' bits=4
m x16b.gif 200 150 'x//13'
m xk5.gif 200 150 'x+1' black=250
m x16k3.gif 200 150 'x//13' bits=4 black=3
m ys2.gif 100 150 'y+1';          m yw.gif 300 150 'y+1'
m ax.gif 80 75 'x+1';             m ay.gif 80 75 'y+1'
m bx.gif 120 75 'x+1';            m by.gif 120 75 'y+1'
m pad.gif 80 75 'x+10' black=5;   m padh.gif 120 75 'y+10' black=5
for w in 96 97 102 103 104; do m t$w.gif $w 75 'x+1'; done
for h in 72 73 77 78; do m v$h.gif 100 $h 'x+1'; done
G='1+(x%10)*20+(y%10)*2'
m gr60.gif 60 45 "$G"; m gr101.gif 101 77 "$G"; m gr200.gif 200 150 "$G"
m gr400.gif 400 300 "$G"; m gr640.gif 640 480 "$G"
for p in 120:75 160:75 100:80 99:75 98:75 80:75 90:75 105:75 110:75 115:75 140:75 100:60 100:70 100:90 100:100 130:80 200:100; do
  m gs${p%:*}-${p#*:}.gif ${p%:*} ${p#*:} "$G"; done
for p in 240:120 250:125 256:128 257:128 260:130 280:140 300:200 320:200 400:200 640:400 300:300 150:300 200:75 300:150; do
  m z${p%:*}-${p#*:}.gif ${p%:*} ${p#*:} 'x%250+1'; done
# wavs.gif with mandel.gif's fractint001 block, 3D fields edited (see cases.txt: blk3d)
python3 -I - "$d" "$here" <<'PY'
import sys, struct
sys.path.insert(0, sys.argv[2])
from gif import read_gif, write_gif
d = sys.argv[1]
g = read_gif(d + '/wavs.gif')
e = bytearray(read_gif(d + '/mandel.gif')['exts']['fractint001'])
for off, val in ((128, 11), (130, 22), (132, 33), (134, 50), (136, 70), (138, 40), (142, 1)):
    e[off:off+2] = struct.pack('<h', val)
write_gif(d + '/wavsblk.gif', g['w'], g['h'], g['gct'], g['pixels'], [('fractint001', bytes(e))])
PY
