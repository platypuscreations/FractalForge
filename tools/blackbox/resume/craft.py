# SPDX-License-Identifier: GPL-3.0-or-later
# craft.py REF OUT key=value... : build a resumable file from a complete render.
#  mode=1|2|g        pass letter to write at 001[200] (default: keep REF's)
#  before=K after=K  pixel content before / at-or-after (R,C): K>=1 KxK blocks of REF, 0 zero
#  R= C= top= bot= f28= f32= f0= f4= left= right= : fractint002 i32 fields
#  status= extlen= : 001 fields
import sys, os, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gif
ref, out = sys.argv[1], sys.argv[2]
kv = dict(a.split('=', 1) for a in sys.argv[3:])
g = gif.read_gif(ref); w, h = g['w'], g['h']
px = bytearray(g['pixels'])
R = int(kv.get('R', 0)); C = int(kv.get('C', 0))
blank = kv.get('blank', '1')
# before=K / after=K: pixels before / at-or-after (R,C) in row-major order take the value of
# their KxK block origin in REF (K=1: exact; K=0: zero). blank=1 means before=1 after=0.
kb = int(kv.get('before', 1)); ka = int(kv.get('after', 0))
if blank == 'garbage': ka = -1
def val(r, c, k):
    if k == 0: return 0
    if k == -1: return 7
    return g['pixels'][(r - r % k)*w + c - c % k]
for r in range(h):
    for c in range(w):
        after = r > R or (r == R and c >= C)
        px[r*w+c] = val(r, c, ka if after else kb)
m = kv.get('mirror')  # x | origin : bottom half becomes a mirror of the top half
if m:
    for r in range((h + 1) // 2, h):
        for c in range(w):
            px[r*w+c] = px[(h-1-r)*w + (c if m == 'x' else w-1-c)]
for spec in kv.get('mark', '').split(','):  # mark=A-B:V,... sets rows A..B to colour V
    if not spec: continue
    rows, v = spec.split(':'); a, _, b = rows.partition('-'); b = b or a
    for r in range(int(a), int(b) + 1):
        for c in range(w): px[r*w+c] = int(v)
for spec in kv.get('markorig', '').split(','):  # markorig=ROW:V:K sets columns c%K==0 of ROW
    if not spec: continue
    rr, v, k = (int(x) for x in spec.split(':'))
    for c in range(0, w, k): px[rr*w+c] = v
f = [0] * 98
f[0] = int(kv.get('f0', 2)); f[1] = int(kv.get('f4', 1))
f[2] = int(kv.get('left', 0)); f[3] = int(kv.get('right', w - 1))
f[4] = int(kv.get('top', R)); f[5] = int(kv.get('bot', h - 1))
f[6] = R; f[7] = int(kv.get('f28', 16)); f[8] = int(kv.get('f32', 0)); f[9] = C
for k, v in kv.items():
    if k.startswith('o') and k[1:].isdigit():  # raw slot override oNN = byte offset NN
        f[int(k[1:]) // 4] = int(v)
for n in (2, 3):  # itemN=left,right,top,bottom,row,sym,pass,col  -> slot N-1
    if ('item%d' % n) in kv:
        f[2+8*(n-1):10+8*(n-1)] = [int(x) for x in kv['item%d' % n].split(',')]
e2 = struct.pack('<98i', *f)
e1 = bytearray(g['exts']['fractint001'])
if 'mode' in kv: e1[200] = ord(kv['mode'])
exts = []
if kv.get('no002') != '1': exts.append(('fractint002', e2))
exts.append(('fractint001', e1))
extlen = sum(3 + 11 + len(p) + (len(p) + 254) // 255 + 1 for _, p in exts)
e1[202:204] = struct.pack('<h', int(kv.get('status', 2)))
e1[204:206] = struct.pack('<H', int(kv.get('extlen', extlen)))
exts[-1] = ('fractint001', bytes(e1))
gif.write_gif(out, w, h, g['gct'], bytes(px), exts, literal=kv.get('literal') == '1')
print(out, 'fields', f[:10], 'extlen', extlen)
