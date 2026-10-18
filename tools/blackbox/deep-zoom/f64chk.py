# SPDX-License-Identifier: GPL-3.0-or-later
# Are the f64 corners / params of fractint001 the nearest doubles of the fractint005 slots?
#   python3 -I f64chk.py FILE.gif...
import os, struct, sys
from fractions import Fraction
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gif, dz
offs = [12, 20, 28, 36, 184, 192, 44, 52]  # slots 0..5, 6, 7
tot = {'nearest': 0, 'other': 0}
for f in sys.argv[1:]:
    g = gif.read_gif(f); e5 = g['exts'].get('fractint005')
    if not e5: continue
    e1 = g['exts']['fractint001']
    N = struct.unpack('<h', e1[414:416])[0]; L = N + 6
    for i, o in enumerate(offs):
        v = dz.slot_decode(e5[i * L:(i + 1) * L])[0]
        d = struct.unpack('<d', e1[o:o + 8])[0]
        near = float(v)  # Python float(Fraction) rounds to nearest
        if d == near: tot['nearest'] += 1
        else:
            tot['other'] += 1
            print(os.path.basename(f), 'slot', i, 'off', o, 'f64', repr(d), 'nearest', repr(near),
                  'ulps', (Fraction(d) - Fraction(near)) / Fraction(abs(near)).limit_denominator(1) if False else '')
print(tot)
