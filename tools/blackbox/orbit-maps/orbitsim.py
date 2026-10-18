# SPDX-License-Identifier: GPL-3.0-or-later
# Black-box analysis helper: simulates our hypothesised XFractint orbit-map
# algorithms (docs/specs/orbit-maps.md) and compares them with saved GIFs.
# Own code; standard library only. Run with python3 -I.
import sys, os, math, struct
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'resume'))
import gif
from info import fields

def load(path):
    g = gif.read_gif(path)
    gct = g['gct']
    canon = [next(j for j in range(256) if gct[3*j:3*j+3] == gct[3*i:3*i+3]) for i in range(256)]
    g['canon'] = canon
    g['f'] = fields(g['exts']['fractint001'])
    return g

def sgn(v):
    return 1.0 if v > 0 else (-1.0 if v < 0 else 0.0)

# Step functions: (x, y, p) -> (x, y). p = params list (10 values).
def henon(x, y, p):
    return 1 + y - p[0] * x * x, p[1] * x

def hopalong(x, y, p):
    return y - sgn(x) * math.sqrt(abs(p[1] * x - p[2])), p[0] - x

def martin(x, y, p):
    return y - math.sin(x), p[0] - x

def gingerbreadman(x, y, p):
    return 1 - y + abs(x), x

STEPS = dict(henon=henon, hopalong=hopalong, martin=martin, gingerbreadman=gingerbreadman)

class Mapper:
    def __init__(self, g):
        self.W, self.H = g['w'], g['h']
        self.xmin, self.xmax, self.ymin, self.ymax = g['f']['corners']
        self.sx = (self.W - 1) / (self.xmax - self.xmin)
        self.sy = (self.H - 1) / (self.ymax - self.ymin)
    def pix(self, x, y):
        """Pixel index or -1. C-style truncation toward zero of the scaled offsets."""
        if not (math.isfinite(x) and math.isfinite(y)):
            return -1
        fc = (x - self.xmin) * self.sx
        fr = (self.ymax - y) * self.sy
        if not (-1 < fc < self.W and -1 < fr < self.H):
            return -1
        c = int(fc); r = int(fr)                # truncation toward zero
        if 0 <= c < self.W and 0 <= r < self.H:
            return r * self.W + c
        return -1

def timecolour(j, c0=2):
    """Colour of orbit point j (j = 0 is the start point) for the
    single-orbit types (spec §3)."""
    return ((c0 - 1 + (j + 1) // 1001) % 255) + 1

def compare(g, img):
    px, canon = g['pixels'], g['canon']
    mm = [i for i in range(len(px)) if canon[img[i] % 256] != px[i]]
    return 100.0 * (1 - len(mm) / len(px)), mm
