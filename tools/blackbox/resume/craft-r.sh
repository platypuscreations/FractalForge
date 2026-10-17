#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Files for the in-program-load (r command) tests. Colour 7 marks pixels whose fate
# tells whether XFractint resumed (marker kept) or recomputed (marker replaced).
cd "$(dirname "$0")"; mkdir -p craft4
P() { python3 -I craft.py "$@"; }
own() {  # own NAME SRC KEY=VAL... : XFractint's own save with markers, blocks untouched
  python3 -I - "$@" <<'PY'
import sys; sys.path.insert(0, '.')
import gif
name, src = sys.argv[1], sys.argv[2]
kv = dict(a.split('=', 1) for a in sys.argv[3:])
g = gif.read_gif(src); w = g['w']; px = bytearray(g['pixels'])
for spec in kv.get('mark', '').split(','):
    if spec:
        r, v = spec.split(':'); px[int(r)*w:(int(r)+1)*w] = bytes([int(v)]) * w
for spec in kv.get('markorig', '').split(','):
    if spec:
        r, v, k = (int(x) for x in spec.split(':'))
        for c in range(0, w, k): px[r*w+c] = v
gif.write_gif('craft4/%s.gif' % name, g['w'], g['h'], g['gct'], bytes(px),
              [(k, g['exts'][k]) for k in g['order'] if k.startswith('fractint')])
print(name, 'from', src)
PY
}
# own saves + markers
own K-MS-1-30  out/i-MS-1-30.gif  mark=10:7
own K-MN-2-45  out/i-MN-2-45.gif  mark=10:7 markorig=200:7:2
own K-J-2-45   out/i-J-2-45.gif   mark=10:7 markorig=140:7:2
own K-MSG-g-60 out/i-MSG-g-60.gif mark=10:7 markorig=140:7:2
own K-MSG-g-30 out/i-MSG-g-30.gif markorig=140:7:4
own K-MN-2-12  out/i-MN-2-12.gif  markorig=100:7:2
own K-MNG-g-10 out/i-MNG-g-10.gif markorig=100:7:4
# third-party style files (literal LZW off: real LZW)
P out/ref-MN-1.gif craft4/C1m.gif R=120 C=200 mark=10:7
P out/ref-MN-1.gif craft4/C2m.gif R=120 C=0 mark=10:7
P out/ref-MN-1.gif craft4/C3m.gif R=0 C=137
P out/ref-MS-1.gif craft4/C4m.gif R=100 C=50 f28=17 mark=50:7
P out/ref-MS-1.gif craft4/C5m.gif R=100 C=50 bot=199 f28=17 mirror=x mark=50:7,240-299:7
P out/ref-MS-1.gif craft4/C5n.gif R=100 C=50 bot=199 f28=16 mirror=x mark=50:7,240-299:7
P out/ref-MS-1.gif craft4/C6m.gif R=200 C=10 f28=17 mark=50:7
P out/ref-J-1.gif  craft4/C7m.gif R=60 C=300 f28=51 mark=50:7
P out/ref-J-1.gif  craft4/C7n.gif R=60 C=300 f28=16 mark=50:7
P out/ref-MN-2.gif craft4/C8m.gif R=0 C=0 top=0 f32=1 before=2 after=2 markorig=100:7:2
P out/ref-MN-2.gif craft4/C9m.gif R=150 C=100 f32=1 after=2 mark=10:7 markorig=200:7:2
P out/ref-MN-2.gif craft4/C10m.gif R=150 C=100 f32=1 after=0 mark=10:7
P out/ref-MN-2.gif craft4/C11m.gif R=100 C=0 top=0 f32=0 before=2 after=0 markorig=50:7:2
P out/ref-MNG-g.gif craft4/C12m.gif R=0 C=0 top=0 f32=1 before=4 after=4 markorig=100:7:4
P out/ref-MNG-g.gif craft4/C13m.gif R=100 C=0 top=0 f32=2 after=2 mark=10:7
P out/ref-MNG-g.gif craft4/C13z.gif R=100 C=0 top=0 f32=2 after=0 mark=10:7
P out/ref-MNG-g.gif craft4/C14m.gif R=96 C=0 top=0 f32=0 before=4 after=0 markorig=10:7:4
P out/ref-MSG-g.gif craft4/C15m.gif R=0 C=0 top=0 f32=2 before=2 after=2 f28=17 markorig=50:7:2
P out/ref-MN-1.gif craft4/C17m.gif R=120 C=0 mode=2 f32=1 after=0 mark=10:7
P out/ref-MN-1.gif craft4/C18m.gif R=120 C=200 bot=199 item2=0,399,200,299,200,16,0,0 f4=2 mark=10:7
# inconsistent / unusual data, all with marker row 10
P out/ref-MN-1.gif craft4/Y1.gif R=120 C=200 f0=1 mark=10:7
P out/ref-MN-1.gif craft4/Y2.gif R=120 C=200 f4=0 mark=10:7
P out/ref-MN-1.gif craft4/Y3.gif R=400 C=0 mark=10:7
P out/ref-MN-1.gif craft4/Y4.gif R=120 C=999 mark=10:7
P out/ref-MN-1.gif craft4/Y5.gif R=120 C=200 no002=1 mark=10:7
P out/ref-MN-1.gif craft4/Y6.gif R=120 C=200 status=4 mark=10:7
P out/ref-MNG-g.gif craft4/Y7.gif R=100 C=0 top=0 f32=7 after=2 mark=10:7
P out/ref-MN-1.gif craft4/Y10.gif R=120 C=200 blank=garbage mark=10:7
P out/ref-MN-1.gif craft4/Y12.gif R=-5 C=0 top=0 mark=10:7
P out/ref-MN-1.gif craft4/Y13.gif R=120 C=200 f28=0 mark=10:7
P out/ref-MN-1.gif craft4/Y14.gif R=120 C=200 f28=17 mark=10:7
P out/ref-MN-1.gif craft4/Y15.gif R=120 C=200 mode=2 mark=10:7
P out/ref-MN-1.gif craft4/Y16.gif R=120 C=200 mode=t mark=10:7
P out/ref-MN-2.gif craft4/Y18.gif R=150 C=100 f32=2 after=2 mark=10:7
P out/ref-MN-1.gif craft4/Y19.gif R=120 C=200 status=1 mark=10:7
P out/ref-MN-1.gif craft4/Y20.gif R=120 C=200 right=199 mark=10:7
P out/ref-MN-1.gif craft4/Y21.gif R=120 C=200 top=50 mark=10:7,80:7
P out/ref-MN-1.gif craft4/Y22.gif R=120 C=200 f4=13 mark=10:7
P out/ref-MN-1.gif craft4/Y23.gif R=120 C=200 f0=0 mark=10:7
