# SPDX-License-Identifier: GPL-3.0-or-later
# High-precision escape-time simulator for deep-zoom XFractint saves (own code).
#   python3 -I sim.py FILE.gif [rows=0,150,299] [step=1] [bits=800|xf] [step_coords=1]
# bits=xf uses 8*(N+3) fraction bits (the slot's own fraction size); step_coords=1
# builds pixel coordinates as corner + index * truncated step instead of exactly.
# Reads the corners from fractint005 (exactly), maxiter from fractint001[408],
# the type from [10] (4 = mandel, 6 = julia) and the julia constant from
# fractint005 slots 6/7, iterates z^2 + c in binary fixed point with `bits`
# fraction bits (truncating products toward minus infinity) and compares the
# colour index with the stored pixel.
# Colour model (gif-fractint-blocks.md §9 / types.md §1.2):
#   z0 = 0 (mandel, so z1 = c) or the pixel (julia); k = index of the first
#   iterate z_k, k >= 1, with |z_k|^2 > 4; colour = max(k - 1, 1) with the
#   255-wrap; inside (colour 0) when k - 1 >= maxiter.
import os, struct, sys
from fractions import Fraction
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gif, dz

def fixed(fr, bits):
    if 'tzc' in os.environ.get('SIMVAR', '') and fr < 0:  # toward zero
        return -fixed(-fr, bits)
    return (fr.numerator << bits) // fr.denominator

VARIANT = os.environ.get('SIMVAR', '')

def mul(a, b, bits):
    p = a * b
    if 'tz' in VARIANT and p < 0:  # truncate toward zero instead of floor
        return -((-p) >> bits)
    return p >> bits

def escape(cx, cy, zx, zy, maxiter, bits):
    """k = index of the first iterate z_k (k >= 1; z_0 is never tested) with
    |z_k|^2 > 4, or None if k would exceed maxiter + 1.
    SIMVAR may contain: tz (truncate products toward zero), xy2 (2xy as
    2*(x*y) instead of (2x)*y), sum (x^2-y^2 via (x+y)(x-y))."""
    four = 4 << bits
    for k in range(1, maxiter + 2):
        x2 = mul(zx, zx, bits)
        y2 = mul(zy, zy, bits)
        if 'xy2' in VARIANT:
            ny = 2 * mul(zx, zy, bits) + cy
        else:
            ny = mul(2 * zx, zy, bits) + cy
        if 'sum' in VARIANT:
            zx = mul(zx + zy, zx - zy, bits) + cx
        else:
            zx = x2 - y2 + cx
        zy = ny
        if mul(zx, zx, bits) + mul(zy, zy, bits) > four:
            return k
    return None

def colour(k, maxiter):
    """colour = k - 1 (minimum 1); inside (colour 0) when k - 1 >= maxiter."""
    if k is None or k - 1 >= maxiter:
        return 0
    n = max(k - 1, 1)
    return n if n <= 255 else ((n - 1) % 255) + 1

def main():
    path = sys.argv[1]
    opts = dict(a.split('=', 1) for a in sys.argv[2:])
    g = gif.read_gif(path)
    e1 = g['exts']['fractint001']
    e5 = g['exts']['fractint005']
    N = struct.unpack('<h', e1[414:416])[0]
    L = N + 6
    slot = lambda i: dz.slot_decode(e5[i * L:(i + 1) * L])[0]
    xmin, xmax, ymin, ymax = (slot(i) for i in range(4))
    ftype = struct.unpack('<h', e1[10:12])[0]
    maxiter = struct.unpack('<i', e1[408:412])[0]
    bits = int(opts.get('bits', 800)) if opts.get('bits') != 'xf' else 8 * (N + 3)
    stepmode = opts.get('step_coords') == '1'
    W, H = g['w'], g['h']
    rows = [int(r) for r in opts.get('rows', '0,%d,%d' % (H // 2, H - 1)).split(',')]
    step = int(opts.get('step', 1))
    gct = g['gct']
    canon = {}
    for i in range(256):
        rgb = gct[3 * i:3 * i + 3]
        canon.setdefault(rgb, i)
    can = lambda i: canon[gct[3 * i:3 * i + 3]]
    jc = (slot(6), slot(7)) if ftype == 6 else None
    total = bad = 0
    diffs = []
    for r in rows:
        if stepmode:  # pixel = corner + index * (span/(n-1) truncated to `bits`)
            Y = fixed(ymax, bits) - r * fixed((ymax - ymin) / (H - 1), bits)
        else:
            Y = fixed(ymax - r * (ymax - ymin) / (H - 1), bits)
        for c in range(0, W, step):
            if stepmode:
                X = fixed(xmin, bits) + c * fixed((xmax - xmin) / (W - 1), bits)
            else:
                X = fixed(xmin + c * (xmax - xmin) / (W - 1), bits)
            if jc:
                n = escape(fixed(jc[0], bits), fixed(jc[1], bits), X, Y, maxiter, bits)
            else:
                n = escape(X, Y, 0, 0, maxiter, bits)
            want = can(colour(n, maxiter))
            got = can(g['pixels'][r * W + c])
            total += 1
            if want != got:
                bad += 1
                if len(diffs) < 20:
                    diffs.append((r, c, n, colour(n, maxiter), g['pixels'][r * W + c]))
    print(os.path.basename(path), 'type', ftype, 'maxiter', maxiter, 'N', N, 'checked', total,
          'mismatch', bad, 'examples (row,col,n,pred,stored):', diffs)

if __name__ == '__main__':
    main()
