# SPDX-License-Identifier: GPL-3.0-or-later
# Writes simple height-field GIFs (own code) for 3D lighting tests.
# Usage: python3 -I mkinput.py OUTDIR
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gif import write_gif, read_gif

def grey():
    # 256 distinct colours even at 6 bits/channel (copy of coloring/uniq.map)
    here = os.path.dirname(os.path.abspath(__file__))
    vals = []
    for line in open(os.path.join(here, 'inputs', 'uniq.map')):
        vals += [int(v) for v in line.split()[:3]]
    return bytes(vals)

def save(path, w, h, f, pal=None):
    px = bytes(max(0, min(255, int(f(x, y)))) for y in range(h) for x in range(w))
    write_gif(path, w, h, pal or grey(), px, [])

out = sys.argv[1]
os.makedirs(out, exist_ok=True)
W, H = 400, 300
save(f'{out}/flat0.gif', W, H, lambda x, y: 0)
save(f'{out}/flat100.gif', W, H, lambda x, y: 100)
save(f'{out}/plateau.gif', W, H, lambda x, y: 200 if (150 <= x < 250 and 100 <= y < 200) else 50)
save(f'{out}/rampx.gif', W, H, lambda x, y: x * 255 // (W - 1))
save(f'{out}/rampy.gif', W, H, lambda x, y: y * 255 // (H - 1))
save(f'{out}/pyramid.gif', W, H, lambda x, y: max(0, 200 - max(abs(x - 200), abs(y - 150))))
# exact slope-1 ramps (one colour step per input pixel) and their mirrors
save(f'{out}/sx1.gif', W, H, lambda x, y: min(255, max(0, x - 72)))
save(f'{out}/sxm.gif', W, H, lambda x, y: min(255, max(0, 327 - x)))
save(f'{out}/sy1.gif', W, H, lambda x, y: min(255, max(0, y - 22)))
save(f'{out}/sym.gif', W, H, lambda x, y: min(255, max(0, 277 - y)))
# a single raised pixel block 2x2 at the centre on height 100
save(f'{out}/bump.gif', W, H, lambda x, y: 150 if (x in (199, 200) and y in (149, 150)) else 100)
# a vertical step edge (x < 200: 100, else 110) and a single raised column / row
save(f'{out}/stepx.gif', W, H, lambda x, y: 110 if x >= 200 else 100)
save(f'{out}/stepy.gif', W, H, lambda x, y: 110 if y >= 150 else 100)
save(f'{out}/spike.gif', W, H, lambda x, y: 110 if (x == 200 and y == 150) else 100)
# pseudo-random heights (own LCG) for probing how fills mix vertex values
_s = [12345]
def _rnd(x, y):
    _s[0] = (_s[0] * 1103515245 + 12345) & 0x7fffffff
    return 20 + (_s[0] >> 16) % 200
save(f'{out}/noise.gif', W, H, _rnd)
# the x ramp again with a plain grey palette (i, i, i)
_g = read_gif(f'{out}/rampx.gif')
write_gif(f'{out}/rampxg.gif', W, H, bytes(v for i in range(256) for v in (i, i, i)), _g['pixels'], [])
