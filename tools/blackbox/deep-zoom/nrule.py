# SPDX-License-Identifier: GPL-3.0-or-later
# Checks the fitted rule for N ([414]) against deep saves:
#   D = ceil(-log10(dx)), dx = (xmax - xmin) / (W - 1) from the fractint005 corners
#   N = smallest multiple of 4 with 8*N*log10(2) - K >= D   (K = 3.38 by default)
#   python3 -I nrule.py [K=3.38] FILE.gif...
import math, os, struct, sys
from fractions import Fraction
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gif, dz
args = sys.argv[1:]
K = 3.38
if args and args[0].startswith('K='):
    K = float(args.pop(0)[2:])
def log10(fr):
    return math.log10(fr.numerator) - math.log10(fr.denominator)
def predict(D):
    n = 4
    while 8 * n * math.log10(2) - K < D:
        n += 4
    return n
ok = bad = 0
for f in args:
    g = gif.read_gif(f); e1 = g['exts']['fractint001']; e5 = g['exts'].get('fractint005')
    if not e5: continue
    N = struct.unpack('<h', e1[414:416])[0]; L = N + 6
    x0, x1, y0, y1 = (dz.slot_decode(e5[i * L:(i + 1) * L])[0] for i in range(4))
    dx = abs(x1 - x0) / (g['w'] - 1); dy = abs(y1 - y0) / (g['h'] - 1)
    D = math.ceil(-log10(min(dx, dy)))
    p = predict(D)
    if p == N: ok += 1
    else:
        bad += 1; print('MISMATCH', os.path.basename(f), 'N', N, 'predicted', p, 'D', D, '-log10 dx %.4f' % -log10(dx))
print('K', K, 'agree', ok, 'disagree', bad)
