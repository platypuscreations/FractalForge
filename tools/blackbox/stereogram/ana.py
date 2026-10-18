# SPDX-License-Identifier: GPL-3.0-or-later
# Summarise a saved GIF: size, blocks, palette, and per-row best "repeat distance".
import sys, os, collections
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gifio import read_gif
g = read_gif(sys.argv[1])
w, h, px = g['w'], g['h'], g['pixels']
print('size', w, h, 'order', g['order'], 'flags', hex(g['flags']))
for k, v in g['exts'].items(): print(' ext', k, len(v))
c = collections.Counter(px); print('colours used', len(c), c.most_common(8))
rows = [int(a) for a in sys.argv[2].split(',')] if len(sys.argv) > 2 else [0, h//2, h-1]
for y in rows:
    r = px[y*w:(y+1)*w]
    best = []
    for s in range(1, w//2):
        m = sum(r[x] == r[x+s] for x in range(w-s)) / (w-s)
        best.append((m, s))
    best.sort(reverse=True)
    print('row', y, 'top shifts', [(s, round(m, 3)) for m, s in best[:4]])
