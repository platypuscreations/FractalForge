# SPDX-License-Identifier: GPL-3.0-or-later
# Simulator for XFractint's light-source fills (filltype 5/6) in the
# top-view case (rotation=0/0/0, scalexyz=100/100/R, no perspective), as
# described in docs/specs/3d-lighting.md.  Own code, black-box derived.
# Usage: python3 -I sim.py INPUT.gif OUTPUT.gif [key=value ...]
#   keys: filltype=5|6 lightsource=x/y/z ambient=A roughness=R smoothing=S
#         scalexyz=100/100/R
import sys, os, math, collections
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gif import read_gif

def light_colour(n, L, amb):
    m = math.sqrt(sum(v * v for v in n))
    c = sum(a * b for a, b in zip(n, L)) / m
    c = max(c, 0.0)
    A = math.ceil(amb * 255 / 100)
    return max(1, math.floor(254 * (1 - c) * (255 - A) / 255))

def simulate(inp, W, H, ft=5, L=(1, -1, 1), amb=20, sx=100, sy=100, r=30, smooth=0):
    z = lambda i, j: inp[j * W + i] * r / 100.0
    L = list(L)
    if ft == 6:
        L = [L[0] * sx / 100, L[1] * sy / 100, L[2] * r / 100]
    m = math.sqrt(sum(v * v for v in L)); L = [v / m for v in L]
    V = {}
    for j in range(1, H):
        acc = None
        for i in range(1, W):
            gx = (z(i, j) - z(i - 1, j)) / (sx / 100)
            gy = (z(i, j) - z(i, j - 1)) / (sy / 100)
            n = (-gx, gy, 1.0)
            if smooth:
                nn = math.sqrt(sum(v * v for v in n)); n = tuple(v / nn for v in n)
                acc = n if acc is None else tuple((smooth * a + b) / (smooth + 1) for a, b in zip(acc, n))
                n = acc
            V[i, j] = light_colour(n, L, amb)
    out = bytearray(W * H)
    for y in range(H):
        for x in range(W):
            i, j = x - 1, y - 1
            if (i, j) in V and (i, j + 1) in V and (i - 1, j + 1) in V:
                out[y * W + x] = (3 * V[i, j] + 2 * V[i, j + 1] + V[i - 1, j + 1]) // 6
            elif (i, j) in V and (i, j + 1) in V:      # first filled column
                out[y * W + x] = (3 * V[i, j] + V[i, j + 1]) // 4
    return out

if __name__ == '__main__':
    g = read_gif(sys.argv[1])
    kv = dict(a.split('=', 1) for a in sys.argv[3:])
    sc = [int(v) for v in kv.get('scalexyz', '100/100/30').split('/')]
    r = int(kv.get('roughness', sc[2] if len(sc) > 2 else 30))
    out = simulate(g['pixels'], g['w'], g['h'], int(kv.get('filltype', 5)),
                   tuple(int(v) for v in kv.get('lightsource', '1/-1/1').split('/')),
                   int(kv.get('ambient', 20)), sc[0], sc[1], r, int(kv.get('smoothing', 0)))
    ref = read_gif(sys.argv[2])
    rp = ref['pixels']
    diff = collections.Counter(rp[k] - out[k] for k in range(len(out)))
    print('match %d/%d (%.3f%%)' % (diff[0], len(out), 100 * diff[0] / len(out)), diff.most_common(5))
