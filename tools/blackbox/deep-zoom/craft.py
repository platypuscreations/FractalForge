# SPDX-License-Identifier: GPL-3.0-or-later
# Makes hand-edited copies of an XFractint deep-zoom save (own code).
#   python3 -I craft.py IN.gif OUT.gif op [op ...]
# ops (applied in order):
#   slots=A-B:x0,x1,y0,y1,x3,y3   write decimal values into slots A..B (6 values: corners)
#   slot=I:value                  write one decimal value into slot I
#   N=n                           re-encode every slot with size n (also sets [414])
#   set412=v  set414=v            raw i16 overwrite
#   f64corners=x0,x1,y0,y1        write doubles at 12/20/28/36 and x3,y3 = x0,y0 at 184/192
#   f64=OFF:value                 write one double at payload offset OFF of fractint001
#   drop005                       remove fractint005
#   add005=N:x0,x1,y0,y1          add a fractint005 (N) with corners (x3,y3 = x0,y0), slots 6-15 = 0,
#                                 slots 16-21 = copy of 0-5; also sets [412]=1, [414]=N
#   len005=n                      truncate / zero-pad fractint005 to n bytes
#   status=v                      i16 at 202
#   markrow=r:v                   repaint image row r with colour v
#   blank=v                       set every pixel to colour v
# Slot values are encoded with round-to-nearest (dz.slot_encode mode 'round').
import os, struct, sys
from fractions import Fraction
from decimal import Decimal
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gif, dz

def frac(s):
    return Fraction(Decimal(s))

def main():
    src, dst = sys.argv[1], sys.argv[2]
    g = gif.read_gif(src)
    exts = [(k, bytearray(v)) for k, v in g['exts'].items()]
    ex = dict(exts)
    order = [k for k, _ in exts]
    pixels = bytearray(g['pixels'])
    e1 = ex['fractint001']
    def N():
        return struct.unpack('<h', e1[414:416])[0]
    for op in sys.argv[3:]:
        k, _, v = op.partition('=')
        if k == 'slots':
            rng, vals = v.split(':')
            a, b = (int(t) for t in rng.split('-'))
            vals = vals.split(',')
            L = N() + 6
            for i, s in zip(range(a, b + 1), vals):
                ex['fractint005'][i * L:(i + 1) * L] = dz.slot_encode(frac(s), N() + 4, 'round')
        elif k == 'slot':
            i, s = v.split(':'); i = int(i)
            L = N() + 6
            ex['fractint005'][i * L:(i + 1) * L] = dz.slot_encode(frac(s), N() + 4, 'round')
        elif k == 'N':
            n = int(v); old = N(); L = old + 6
            e5 = ex['fractint005']
            vals = [dz.slot_decode(e5[i * L:(i + 1) * L])[0] for i in range(22)]
            ex['fractint005'] = bytearray(b''.join(dz.slot_encode(x, n + 4, 'round') for x in vals))
            e1[414:416] = struct.pack('<h', n)
        elif k == 'set412':
            e1[412:414] = struct.pack('<h', int(v))
        elif k == 'set414':
            e1[414:416] = struct.pack('<h', int(v))
        elif k == 'f64corners':
            x0, x1, y0, y1 = (float(t) for t in v.split(','))
            for off, val in ((12, x0), (20, x1), (28, y0), (36, y1), (184, x0), (192, y0)):
                e1[off:off + 8] = struct.pack('<d', val)
        elif k == 'f64':
            off, val = v.split(':')
            e1[int(off):int(off) + 8] = struct.pack('<d', float(val))
        elif k == 'drop005':
            ex.pop('fractint005'); order.remove('fractint005')
        elif k == 'add005':
            n, vals = v.split(':'); n = int(n)
            x0, x1, y0, y1 = (frac(t) for t in vals.split(','))
            six = [x0, x1, y0, y1, x0, y0]
            sl = six + [Fraction(0)] * 10 + six
            ex['fractint005'] = bytearray(b''.join(dz.slot_encode(x, n + 4, 'round') for x in sl))
            if 'fractint005' not in order:
                order.insert(order.index('fractint001'), 'fractint005')
            e1[412:414] = struct.pack('<h', 1)
            e1[414:416] = struct.pack('<h', n)
        elif k == 'len005':
            n = int(v); b = ex['fractint005']
            ex['fractint005'] = (b + bytearray(max(0, n - len(b))))[:n]
        elif k == 'status':
            e1[202:204] = struct.pack('<h', int(v))
        elif k == 'markrow':  # markrow=ROW:COLOUR repaints one image row
            r, col = (int(t) for t in v.split(':'))
            pixels[r * g['w']:(r + 1) * g['w']] = bytes([col]) * g['w']
        elif k == 'blank':
            pixels = bytearray([int(v)]) * len(pixels)
        else:
            raise SystemExit('unknown op ' + op)
    out = [(k, bytes(ex[k])) for k in order]
    # keep [204] (total extension bytes) consistent
    tot = sum(3 + 11 + len(gif.subblocks(p)) for _, p in out)
    e1[204:206] = struct.pack('<H', tot)
    out = [(k, bytes(ex[k])) for k in order]
    gif.write_gif(dst, g['w'], g['h'], g['gct'], bytes(pixels), out)

if __name__ == '__main__':
    main()
