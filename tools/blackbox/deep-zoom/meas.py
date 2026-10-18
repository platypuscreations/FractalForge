# SPDX-License-Identifier: GPL-3.0-or-later
# For center-mag cases: error of the stored f64 corners against the exact
# corners (centre +- (4/3)/mag, +- 1/mag), in units of the pixel spacing.
#   python3 -I meas.py W H FILE.gif:cx:cy:mag ...
import struct, sys, os
from fractions import Fraction as F
from decimal import Decimal
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gif
W, H = int(sys.argv[1]), int(sys.argv[2])
for a in sys.argv[3:]:
    f, cx, cy, mag = a.split(':')
    cx, cy, mag = (F(Decimal(t)) for t in (cx, cy, mag))
    g = gif.read_gif(f)
    e = g['exts']['fractint001']
    d = lambda o: F(struct.unpack('<d', e[o:o + 8])[0])
    hw = F(4, 3) / mag; hh = 1 / mag
    ex = [cx - hw, cx + hw, cy - hh, cy + hh]
    fs = [d(12), d(20), d(28), d(36)]
    sx = 2 * hw / (W - 1); sy = 2 * hh / (H - 1)
    errs = [float(abs(fs[i] - ex[i]) / (sx if i < 2 else sy)) for i in range(4)]
    wd = float(abs((fs[1] - fs[0]) - 2 * hw) / (2 * hw)); hd = float(abs((fs[3] - fs[2]) - 2 * hh) / (2 * hh))
    print('%-22s deep=%d corner-err/spacing %s  width-relerr %.3g height-relerr %.3g' % (
        os.path.basename(f)[:-4], struct.unpack('<h', e[412:414])[0], ' '.join('%.3f' % x for x in errs), wd, hd))
