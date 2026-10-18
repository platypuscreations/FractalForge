# SPDX-License-Identifier: GPL-3.0-or-later
# Deep-zoom analysis helpers (own code, black-box work only).
#   python3 -I dz.py FILE.gif...      dump fractint001 deep fields + fractint005 slots
import struct, sys, os
from fractions import Fraction
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gif

def slot_decode(b):
    """Slot of M+2 bytes: signed LE mantissa (M bytes) + i16 LE exponent."""
    M = len(b) - 2
    mant = int.from_bytes(b[:M], 'little', signed=True)
    e = struct.unpack('<h', b[M:M + 2])[0]
    return Fraction(mant, 1 << (8 * (M - 1))) * Fraction(256) ** (e + 1), mant, e

def slot_encode(v, M, mode='round'):
    """Encode Fraction v into M mantissa bytes + i16 exponent, normalised the
    way XFractint does: e is the smallest integer with -1 <= v/256^(e+1) < 1
    (zero: all bytes 0, e = 0). mode: 'trunc' (toward zero), 'floor', 'round'."""
    v = Fraction(v)
    if v == 0:
        return bytes(M) + struct.pack('<h', 0)
    e = 0
    fits = lambda e: -1 <= v / Fraction(256) ** (e + 1) < 1
    while not fits(e):
        e += 1
    while fits(e - 1):
        e -= 1
    scaled = v / Fraction(256) ** (e + 1) * (1 << (8 * (M - 1)))
    if mode == 'trunc':
        m = int(scaled)  # toward zero
    elif mode == 'floor':
        m = scaled.numerator // scaled.denominator
    else:
        m = round(scaled)
    if m >= 1 << (8 * (M - 1)):  # rounding overflowed the mantissa
        return slot_encode(Fraction(m, 1 << (8 * (M - 1))) * Fraction(256) ** (e + 1), M, mode)
    return m.to_bytes(M, 'little', signed=True) + struct.pack('<h', e)

def fmt(fr, digits=60):
    # exact-ish decimal string of a Fraction
    sign = '-' if fr < 0 else ''
    fr = abs(fr)
    ip = fr.numerator // fr.denominator
    rem = fr - ip
    s = str(ip) + '.'
    for _ in range(digits):
        rem *= 10
        d = rem.numerator // rem.denominator
        s += str(d)
        rem -= d
    return sign + s

def info(path, digits=50):
    g = gif.read_gif(path)
    e1 = g['exts']['fractint001']
    h = lambda o: struct.unpack('<h', e1[o:o + 2])[0]
    d = lambda o: struct.unpack('<d', e1[o:o + 8])[0]
    print('==', path, g['w'], 'x', g['h'], 'blocks', g['order'])
    print(' type', h(10), 'maxiter', struct.unpack('<i', e1[408:412])[0], 'status', h(202),
          'pass', chr(e1[200]), '[412]', h(412), '[414]', h(414), '[416]', h(416),
          'tol', d(474), d(482))
    print(' f64 xmin xmax ymin ymax', [repr(d(o)) for o in (12, 20, 28, 36)],
          'x3 y3', repr(d(184)), repr(d(192)), 'p1 p2', repr(d(44)), repr(d(52)))
    e5 = g['exts'].get('fractint005')
    if e5:
        N = h(414)
        L = N + 6
        print(' fractint005 len', len(e5), 'N', N, 'slots', len(e5) / L)
        for i in range(len(e5) // L):
            v, m, e = slot_decode(e5[i * L:(i + 1) * L])
            if i < 16 or e5[i * L:(i + 1) * L] != e5[(i - 16) * L:(i - 15) * L]:
                print('  slot %2d e=%3d %s' % (i, e, fmt(v, digits)))
        print('  slots16-21==0-5:', e5[16 * L:22 * L] == e5[:6 * L])
    return g

if __name__ == '__main__':
    for f in sys.argv[1:]:
        info(f)
