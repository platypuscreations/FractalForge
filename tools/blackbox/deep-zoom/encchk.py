# SPDX-License-Identifier: GPL-3.0-or-later
# Compares fractint005 slots with the exact decimal values given on the command line.
#   python3 -I encchk.py FILE.gif v0,v1,... [first-slot]
# Prints, per slot: exponent, decoded mantissa bytes, error (decoded - exact) in
# units of the last mantissa byte (ulp = 256^(e+1) / 2^(8(M-1))), and whether
# truncation / floor / round-to-nearest would give the stored bytes.
import os, struct, sys
from fractions import Fraction
from decimal import Decimal
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gif, dz
f, vals = sys.argv[1], sys.argv[2].split(',')
first = int(sys.argv[3]) if len(sys.argv) > 3 else 0
g = gif.read_gif(f)
e1 = g['exts']['fractint001']; e5 = g['exts']['fractint005']
N = struct.unpack('<h', e1[414:416])[0]; L = N + 6; M = N + 4
for i, s in enumerate(vals):
    k = first + i
    b = e5[k * L:(k + 1) * L]
    v, m, e = dz.slot_decode(b)
    ex = Fraction(Decimal(s))
    ulp = Fraction(256) ** (e + 1) / (1 << (8 * (M - 1)))
    modes = [md for md in ('trunc', 'floor', 'round') if dz.slot_encode(ex, M, md) == bytes(b)]
    print('slot %2d e=%3d err=%+.3f ulp  match=%s  bytes=%s  value=%s' % (
        k, e, float((v - ex) / ulp), modes, b.hex(' '), dz.fmt(v, 45)))
