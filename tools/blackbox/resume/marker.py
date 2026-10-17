# SPDX-License-Identifier: GPL-3.0-or-later
# marker.py OUT IN REF : per-row comparison of a resumed save OUT with its crafted input IN
# and the reference REF. Reports rows equal to REF, rows unchanged from IN, others.
import sys, os, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gif
o = gif.read_gif(sys.argv[1]); i = gif.read_gif(sys.argv[2]); r = gif.read_gif(sys.argv[3])
w, h = o['w'], o['h']
st = struct.unpack('<h', o['exts']['fractint001'][202:204])[0]
def row(p, k): return p[k*w:(k+1)*w]
cls = []
for k in range(h):
    a, b, c = row(o['pixels'], k), row(i['pixels'], k), row(r['pixels'], k)
    if a == c and a == b: t = 'same'      # input already right and kept/recomputed equal
    elif a == c: t = 'REF'                # fixed by XFractint
    elif a == b: t = 'IN'                 # left as in input
    else: t = 'mix'
    cls.append(t)
runs = []; s = 0
for k in range(1, h + 1):
    if k == h or cls[k] != cls[s]: runs.append('%d-%d:%s' % (s, k - 1, cls[s])); s = k
diff = sum(x != y for x, y in zip(o['pixels'], r['pixels']))
print(os.path.basename(sys.argv[1]), 'status', st, 'diff_vs_ref', diff, ' '.join(runs))
