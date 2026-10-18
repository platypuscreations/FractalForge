# SPDX-License-Identifier: GPL-3.0-or-later
# High-precision elementary functions rounded first to 64-bit mantissa
# (x87 extended) and then to double, the evaluation that reproduced
# XFractint's function-slot results (docs/specs/orbit-maps.md §2.4).
# Own code; standard library only.
from decimal import Decimal, getcontext, localcontext
import math

_PI = Decimal('3.14159265358979323846264338327950288419716939937510582097494459')

def _sin_dec(x):
    with localcontext() as ctx:
        ctx.prec = 45
        x = x % (2 * _PI)
        if x > _PI: x -= 2 * _PI
        s = term = x; i = 1; x2 = x * x
        while True:
            i += 2
            term = -term * x2 / (i * (i - 1))
            ns = s + term
            if ns == s: return s
            s = ns

def _round_bits(d, p):
    """Round Decimal d to p significant bits (nearest-even); return Decimal-exact
    value as (mantissa, exponent) integer pair."""
    if d == 0: return 0, 0
    neg = d < 0; d = abs(d)
    m, e = math.frexp(float(d))      # approx exponent
    e -= 1                           # d ~ 2^e
    with localcontext() as ctx:
        ctx.prec = 60
        scaled = d * (Decimal(2) ** (p - 1 - e))
        n = int(scaled)
        if n >= (1 << p):
            e += 1; scaled = d * (Decimal(2) ** (p - 1 - e)); n = int(scaled)
        if n < (1 << (p - 1)):
            e -= 1; scaled = d * (Decimal(2) ** (p - 1 - e)); n = int(scaled)
        rem = scaled - n
        if rem > Decimal('0.5') or (rem == Decimal('0.5') and n & 1): n += 1
    return (-n if neg else n), e - (p - 1)

def ext_then_double(d):
    """Decimal -> round to 64-bit mantissa -> round to 53-bit -> float."""
    n, e = _round_bits(d, 64)
    if n == 0: return 0.0
    neg = n < 0; n = abs(n)
    # round 64-bit integer mantissa to 53 bits, nearest-even
    sh = n.bit_length() - 53
    if sh > 0:
        q, r = n >> sh, n & ((1 << sh) - 1); half = 1 << (sh - 1)
        if r > half or (r == half and q & 1): q += 1
        n, e = q, e + sh
    v = math.ldexp(n, e)
    return -v if neg else v

def xsin(v): return ext_then_double(_sin_dec(Decimal(v)))
def xcos(v): return ext_then_double(_sin_dec(Decimal(v) + _PI / 2))

def xtan_of_product(a, b):
    """tan(a*b) where the product a*b is NOT rounded to double first
    (as when it stays in an extended-precision register)."""
    d = Decimal(a) * Decimal(b)          # exact product of two doubles
    s = _sin_dec(d); c = _sin_dec(d + _PI / 2)
    with localcontext() as ctx:
        ctx.prec = 45
        return ext_then_double(s / c)

def xtan(v):
    d = Decimal(v); s = _sin_dec(d); c = _sin_dec(d + _PI / 2)
    with localcontext() as ctx:
        ctx.prec = 45
        return ext_then_double(s / c)
