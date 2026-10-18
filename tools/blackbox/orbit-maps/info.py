# SPDX-License-Identifier: GPL-3.0-or-later
# Dumps the fractint001 fields relevant to orbit maps and a colour histogram.
# Usage: python3 -I info.py FILE.gif...
import struct, sys, os, collections
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'resume'))
import gif

def fields(b):
    f = lambda o: struct.unpack_from('<d', b, o)[0]
    i16 = lambda o: struct.unpack_from('<h', b, o)[0]
    i32 = lambda o: struct.unpack_from('<i', b, o)[0]
    return dict(type=i16(10), corners=[f(12), f(20), f(28), f(36)], third=[f(184), f(192)],
                params=[f(44), f(52), f(270), f(278)] + [f(354 + 8 * k) for k in range(6)],
                maxiter=i32(408), bailout=i32(402), status=i16(202), passes=chr(b[200]),
                sym=i16(124), seed=struct.unpack_from('<H', b, 98)[0], inside=i16(104),
                outside=i16(182), periodicity=i16(240), time=struct.unpack_from('<I', b, 214)[0])

if __name__ == '__main__':
    for p in sys.argv[1:]:
        g = gif.read_gif(p)
        b = g['exts']['fractint001']
        fl = fields(b)
        hist = collections.Counter(g['pixels'])
        print(os.path.basename(p), g['w'], 'x', g['h'], g['order'])
        print('  ', fl)
        print('   colours:', len(hist), sorted(hist.items(), key=lambda kv: -kv[1])[:12])
