# SPDX-License-Identifier: GPL-3.0-or-later
# Compares a loaded-and-recalculated save with reference renders.
#   python3 -I cmp.py OUT.gif REF1.gif [REF2.gif ...]
# Prints the output's deep fields and, per reference, how many rows are
# identical, and how many pixels differ.
import os, struct, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gif, dz
o = gif.read_gif(sys.argv[1])
e1 = o['exts']['fractint001']; h = lambda o_: struct.unpack('<h', e1[o_:o_ + 2])[0]
d = lambda o_: struct.unpack('<d', e1[o_:o_ + 8])[0]
e5 = o['exts'].get('fractint005')
s = '%s: status %d 412=%d 414=%d 005=%s f64 x0 %r y0 %r p %r/%r' % (
    os.path.basename(sys.argv[1]), h(202), h(412), h(414), len(e5) if e5 else '-', d(12), d(28), d(44), d(52))
if e5:
    L = h(414) + 6
    s += ' slot0 %s slot2 %s' % (dz.fmt(dz.slot_decode(e5[:L])[0], 25), dz.fmt(dz.slot_decode(e5[2 * L:3 * L])[0], 25))
    if len(e5) >= 22 * L:
        s += ' copy-ok' if e5[16 * L:22 * L] == e5[:6 * L] else ' COPY-DIFFERS'
print(s)
W = o['w']
for r in sys.argv[2:]:
    g = gif.read_gif(r)
    if (g['w'], g['h']) != (o['w'], o['h']):
        print('  vs', os.path.basename(r), 'size differs'); continue
    rows = sum(o['pixels'][i * W:(i + 1) * W] == g['pixels'][i * W:(i + 1) * W] for i in range(o['h']))
    px = sum(a != b for a, b in zip(o['pixels'], g['pixels']))
    print('  vs %-14s equal rows %3d/%d, pixels differing %d' % (os.path.basename(r), rows, o['h'], px))
