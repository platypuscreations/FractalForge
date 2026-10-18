# SPDX-License-Identifier: GPL-3.0-or-later
# Prints the fractint001 fields relevant to attractor types and pixel stats.
import sys, struct, collections
sys.path.insert(0, __import__('os').path.dirname(__file__))
from gif import read_gif

def info(path):
    g = read_gif(path)
    b = g['exts'].get('fractint001')
    f = lambda o: struct.unpack_from('<d', b, o)[0]
    i16 = lambda o: struct.unpack_from('<h', b, o)[0]
    i32 = lambda o: struct.unpack_from('<i', b, o)[0]
    params = [f(44), f(52), f(270), f(278)] + [f(354 + 8 * k) for k in range(6)]
    cnt = collections.Counter(g['pixels'])
    print('%s: %dx%d type=%d maxiter=%d status=%d corners=%r 3rd=%r' % (
        path.split('/')[-1], g['w'], g['h'], i16(10), i32(408), i16(202),
        (f(12), f(20), f(28), f(36)), (f(184), f(192))))
    print('   params=%s' % '/'.join('%.17g' % p for p in params))
    print('   rot=%d/%d/%d scale=%d/%d/%d persp=%d shift=%d/%d passes=%s exts=%s' % (
        i16(128), i16(130), i16(132), i16(134), i16(136), i16(138), i16(144),
        i16(146), i16(148), chr(b[200]), list(g['exts'])))
    print('   colours=%s' % sorted(cnt.items(), key=lambda kv: -kv[1])[:12])

for p in sys.argv[1:]:
    info(p)
