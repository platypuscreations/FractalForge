# SPDX-License-Identifier: GPL-3.0-or-later
# Black-box model of XFractint's random-dot stereogram (docs/specs/stereogram.md).
# Own code, derived only from XFractint's output files.
#   python3 -I rdsmodel.py OUT.gif DEPTH.gif SEED WIDTH_IN DEPTH_EFFECT [A] [middle|top|none]
# Prints per-row mismatch counts between OUT.gif and the model.
import sys, os, math
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gifio import read_gif

def glibc_rand(seed, n):
    """glibc rand() outputs 0..n-1 after srand(seed) (published TYPE_3 algorithm)."""
    s = [0] * (344 + n)
    s[0] = seed & 0xffffffff or 1
    if s[0] >= 2**31: s[0] -= 2**32
    for i in range(1, 31):
        s[i] = (16807 * s[i - 1]) % 2147483647
    for i in range(31, 34):
        s[i] = s[i - 31]
    for i in range(34, 344 + n):
        s[i] = (s[i - 31] + s[i - 3]) & 0xffffffff
    return [s[344 + k] >> 1 for k in range(n)]

A_INCH = 125 / 67   # hypothesis for the far-plane separation in inches (fit range 1.86563..1.86585)

def grey_values(gct):
    """Approximation only (spec: grayscale option): 0.30 R + 0.59 G + 0.11 B of the 8-bit palette."""
    return [0.30 * gct[3 * i] + 0.59 * gct[3 * i + 1] + 0.11 * gct[3 * i + 2] for i in range(len(gct) // 3)]

def separation(c, xdots, width_in, effect, a=A_INCH, cmin=0, cmax=255):
    lmax = abs(effect) * xdots // 4000    # number of depth steps above the farthest
    k = lmax + 1
    cn = int((c - cmin) * 255 // (cmax - cmin)) if cmax > cmin else 0
    if effect < 0:
        cn = 256 - cn
    level = (cn * k - 1) // 256          # cn = 0 -> -1
    return math.floor(xdots / width_in * a * (1 - (lmax - level) * 8 / xdots))

def stereo_row(depth_row, rnd_row, y, sep):
    w = len(depth_row)
    same = list(range(w))
    for x in range(w):
        s = sep(depth_row[x])
        if s < 0:
            continue    # not modelled: XFractint's output is undefined-looking here (spec 4.6)
        left = x - (s + (y & 1)) // 2
        right = left + s
        if left < 0 or right >= w:
            continue
        k = same[left]
        while k != left and k != right:
            if k < right:
                left = k
            else:
                left, right = right, k
            k = same[left]
        same[left] = right
    out = [0] * w
    for x in range(w - 1, -1, -1):
        out[x] = rnd_row[x] if same[x] == x else out[same[x]]
    return out

def bar_colour(gct):
    """Palette index with the largest R+G+B (lowest index on ties)."""
    sums = [sum(gct[3 * i:3 * i + 3]) for i in range(len(gct) // 3)]
    return sums.index(max(sums))

def stereogram(depth, w, h, seed, width_in, effect, a=A_INCH, bars='middle', gct=None,
               xhalf=None, yhalf=8, pattern=None):
    """depth: colour indices (or grey values) row-major. pattern: (pw, ph, pixels) for the
    image-map option, else free pixels come from glibc rand() seeded with seed."""
    r = glibc_rand(seed, w * h) if pattern is None else None
    lo, hi = min(depth), max(depth)
    sep = lambda c: separation(c, w, width_in, effect, a, lo, hi)
    out = []
    for y in range(h):
        if pattern is None:
            free = [v & 255 for v in r[y * w:(y + 1) * w]]
        else:
            pw, ph, pp = pattern
            free = [pp[(y % ph) * pw + x % pw] for x in range(w)]
        out += stereo_row(depth[y * w:(y + 1) * w], free, y, sep)
    if bars != 'none':
        ybar = h // 2 if bars == 'middle' else 8
        xh = w // 16 if xhalf is None else xhalf
        xs = range(w // 2 - xh, w // 2 + xh + 1)
        ys = range(max(0, ybar - yhalf), min(h, ybar + yhalf + 1))
        tot = sum(sep(depth[y * w + x]) for y in ys for x in xs)
        half = (tot // (len(xs) * len(ys))) // 2
        col = bar_colour(gct) if gct is not None else 255
        bar_w, bar_h = w // 200 + 1, h // 20 + 1   # 3 x 16 at 400x300, 5 x 31 at 800x600
        for y in range(ybar, ybar + bar_h):
            for x in (w // 2 - half, w // 2 + half):
                for dx in range(bar_w):
                    out[y * w + x + dx] = col
    return out

if __name__ == '__main__':
    o = read_gif(sys.argv[1]); d = read_gif(sys.argv[2])
    seed, width_in, effect = int(sys.argv[3]), float(sys.argv[4]), int(sys.argv[5])
    a = float(sys.argv[6]) if len(sys.argv) > 6 and sys.argv[6] != '-' else A_INCH
    w, h = o['w'], o['h']
    bars = sys.argv[7] if len(sys.argv) > 7 else 'middle'
    m = stereogram(d['pixels'], w, h, seed, width_in, effect, a, bars, d['gct'])
    bad = [(y, sum(m[y*w+x] != o['pixels'][y*w+x] for x in range(w))) for y in range(h)]
    bad = [b for b in bad if b[1]]
    print('rows with mismatches:', len(bad), bad[:20])
    print('total mismatched pixels:', sum(b[1] for b in bad))
