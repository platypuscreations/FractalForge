# SPDX-License-Identifier: GPL-3.0-or-later
# Measures where XFractint places the turtle origin, to ~1e-15 pixel.
# A probe branch "[\90@aD]" is put in front of the axiom of a test curve: it
# draws a segment of length a straight up from the origin (inside the
# curve's own bounding box, so the fit is unchanged).  For a target row M,
# bisection on a finds where the segment's top end flips between rows M and
# M+1; then origin_y = M + 0.5 + a*scale.  The scale must be given (it is the
# float32 value of (H-6)/height, see docs/specs/lsystem.md).
#   python3 -I bisect-origin.py OUTDIR ANGLE AXIOM RULES(;-separated) ORDER SCALE Y0GUESS M...
# PROBE_COL must name the origin's pixel column.
# Prints per M the bracket of the flip offset d* and origin_y - Y0GUESS = -d*.
import os, sys, subprocess, concurrent.futures as cf
from fractions import Fraction as Fr
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(here, '..', 'resume'))
from gif import read_gif
out, ang, axiom, rules, order, scale, y0g = sys.argv[1:8]
Ms = [int(v) for v in sys.argv[8:]]
scale = Fr(float(scale)); y0g = Fr(y0g)
os.makedirs(out, exist_ok=True)
def run(m, a, tag):
    txt = f'{float(a):.17g}'
    w = os.path.join(out, tag); os.makedirs(w, exist_ok=True)
    body = [f'  angle {ang}', f'  axiom [\\90@{txt}D]{axiom}'] + [f'  {r}' for r in rules.split(';') if r]
    open(os.path.join(w, 'p.l'), 'w').write('e {\n' + '\n'.join(body) + '\n}\n')
    subprocess.run(['docker', 'run', '--rm', '-v', f'{w}:/work', 'fractalforge-blackbox', 'type=lsystem',
                    'lfile=p.l', 'lname=e', f'params={order}', 'viewwindows=2', 'savename=ff.gif'],
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    G = read_gif(os.path.join(w, 'ff.gif')); W = G['w']; px = G['pixels']
    for f in os.listdir(w): os.remove(os.path.join(w, f))
    os.rmdir(w)
    return G, txt
def probe_col(G, x0col):
    W, H = G['w'], G['h']; px = G['pixels']
    return [y for y in range(H) if px[y * W + x0col]]
def bisect(m):
    # a such that y0 - a*scale = m + 0.5 + d
    lo, hi = Fr(-1, 10**6), Fr(1, 10**6)
    xcol = None
    for i in range(36):
        mid = (lo + hi) / 2
        a = (y0g - m - Fr(1, 2) - mid) / scale
        G, txt = run(m, a, f'w{m}-{i}')
        W = G['w']; px = G['pixels']
        c = int(os.environ['PROBE_COL'])   # the origin's column (no curve pixels near row M there)
        rows = [y for y in range(m - 3, m + 4) if px[y * W + c]]
        top = min(rows) if rows else None
        # top end row = floor(end + 1/2), end = M + 1/2 + mid + (y0 - y0guess)
        if top == m + 1: hi = mid
        elif top == m: lo = mid
        else: return m, None, None, top
    return m, lo, hi, None
with cf.ThreadPoolExecutor(max_workers=10) as ex:
    for m, lo, hi, err in ex.map(bisect, Ms):
        if lo is None: print(m, 'unexpected top row', err); continue
        # recompute with the doubles XFractint actually parsed
        alo = Fr(float((y0g - m - Fr(1, 2) - lo) / scale)); ahi = Fr(float((y0g - m - Fr(1, 2) - hi) / scale))
        ylo, yhi = sorted((m + Fr(1, 2) + alo * scale, m + Fr(1, 2) + ahi * scale))
        print(m, 'origin y in', f'[{float(ylo - y0g):.4e}, {float(yhi - y0g):.4e}] relative to the guess', flush=True)
