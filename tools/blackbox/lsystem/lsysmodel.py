# SPDX-License-Identifier: GPL-3.0-or-later
# Reference model of XFractint's L-system renderer, written from the
# black-box observations in docs/specs/lsystem.md (own code, no XFractint
# source).  It emulates the arithmetic the spec describes exactly:
#   - turtle positions, steps and the fit in x87 extended precision
#     (64-bit significand, round to nearest even), emulated with Fractions;
#   - the scale and the aspect factor rounded to float32;
#   - integer-heading directions cos/sin(ext(ext(2*pi_double/N)*k)) kept in
#     extended precision; degree headings via a double angle in radians.
# Usage as a library: render(lfile_text, entry_name, order, W, H) -> bytes.
import math, re, struct
from decimal import Decimal, getcontext
from fractions import Fraction as Fr

# ---------------------------------------------------------------- numbers
def rnd(q, bits):
    """Round Fraction q to `bits` significant bits, nearest, ties to even."""
    if q == 0:
        return Fr(0)
    sign = -1 if q < 0 else 1
    n, d = abs(q).numerator, abs(q).denominator
    k = bits - 1 - (n.bit_length() - d.bit_length())
    if (n << k if k >= 0 else n) < ((1 << (bits - 1)) * (d if k >= 0 else d << -k)):
        k += 1
    num, den = (n << k, d) if k >= 0 else (n, d << -k)
    q_, r = divmod(num, den)
    if 2 * r > den or (2 * r == den and q_ & 1):
        q_ += 1
    return sign * (Fr(q_, 1 << k) if k >= 0 else Fr(q_ << -k))

X = lambda q: rnd(Fr(q), 64)                       # x87 extended
F32 = lambda v: struct.unpack('f', struct.pack('f', float(v)))[0]

getcontext().prec = 60
def _pi():
    def atan_inv(x):
        x = Decimal(x); s = Decimal(0); t = 1 / x; n = 1; x2 = x * x; sg = 1
        while t / n > Decimal(10) ** -70:
            s += sg * t / n; t /= x2; n += 2; sg = -sg
        return s
    return 4 * (4 * atan_inv(5) - atan_inv(239))
_PI = _pi()

def cos_sin(a):
    """cos and sin of the exact rational a, to ~1e-55."""
    x = Decimal(a.numerator) / Decimal(a.denominator)
    x -= (x / (2 * _PI)).to_integral_value() * 2 * _PI
    c = s = Decimal(0); t = Decimal(1); n = 0
    while abs(t) > Decimal(10) ** -58:
        if n % 4 == 0: c += t
        elif n % 4 == 1: s += t
        elif n % 4 == 2: c -= t
        else: s -= t
        n += 1; t = t * x / n
    return Fr(c), Fr(s)

# ---------------------------------------------------------------- .l files
def parse_entries(text):
    """Parse a .l file into {lowercase name: (angle, axiom, rules)} following
    spec section 2 (only what valid files need)."""
    E = {}
    lines = []
    for raw in text.replace('\r\n', '\n').split('\n'):
        while len(raw) > 255:                          # long lines are split
            lines.append(raw[:255]); raw = raw[255:]
        lines.append(raw)
    code = [l.split(';')[0] for l in lines]
    for i, line in enumerate(code):
        if '{' not in line:
            continue
        head, rest = line.split('{', 1)
        name = re.split(r'[\s(\[]', head.strip().lstrip('}').strip())[0].lower()   # ends at ( or [
        if name in E:
            continue
        # found only if a '}' follows (same line after '{' or a later line)
        # before the next line holding '{'
        if '}' not in rest:
            j = i + 1
            while j < len(code) and '{' not in code[j] and '}' not in code[j]:
                j += 1
            if j >= len(code) or '{' in code[j]:
                continue
        E[name] = _entry([rest] + code[i + 1:])
    return E

def _entry(body):
    angle, axiom, rules = None, None, {}
    for l in body:
        l = l.strip().lower()
        if not l: continue
        if l.startswith('}'): break                     # end of the entry
        l = l.lstrip('=')                               # '=' separates like a space
        if not l.strip(): continue
        l = l.strip()
        m = re.match(r'([^\s=]+)[\s=]*(\S*)', l)
        key, val = m.group(1), m.group(2)
        if key == 'angle': angle = int(re.match(r'[+-]?\d*', val).group(0) or 0)
        elif key == 'axiom': axiom = val
        elif len(key) == 1: rules[key] = rules.get(key, '') + val
    return angle, axiom, rules

# ---------------------------------------------------------------- strings
ARG = re.compile(r'[0-9.]*')
ATARG = re.compile(r'[iq]*[0-9.]*')
def expand(axiom, rules, order, protect):
    """Rewrite `order` times.  Commands in `protect` copy their argument
    unchanged (it is never rewritten)."""
    s = axiom
    for _ in range(max(0, order)):
        out = []; i = 0
        while i < len(s):
            c = s[i]
            if c in protect:
                m = (ATARG if c == '@' else ARG).match(s, i + 1)
                out.append(s[i:m.end()]); i = m.end(); continue
            out.append(rules.get(c, c)); i += 1
        s = ''.join(out)
    return s

def num(txt):
    m = re.match(r'\d*(\.\d*)?', txt).group(0)
    try: return float(m)
    except ValueError: return 0.0

# ---------------------------------------------------------------- turtle
class Turtle:
    def __init__(self, N, asp, draw_pass):
        self.N, self.asp, self.draw_pass = N, asp, draw_pass
        step = X(Fr(2 * math.pi) / N)
        self.tab = []
        for k in range(N):
            c, s = cos_sin(X(step * k))
            self.tab.append((X(X(c) * asp), X(s)))
    def realdir(self, deg):
        r = deg * math.pi / 180                          # double
        c, s = cos_sin(Fr(r))
        return X(Fr(float(c)) * self.asp), Fr(float(s))
    def run(self, s, x, y, size, plot=None):
        N = self.N; k = 0; deg = 0.0; col = 15; rev = False; stack = []
        pts = [(x, y)]; i = 0
        while i < len(s):
            c = s[i]; i += 1
            if c in 'fgdm':
                dx, dy = self.tab[k] if c in 'fg' else self.realdir(deg)
                nx, ny = X(x + X(size * dx)), X(y + X(size * dy))
                if plot and c in 'fd':
                    plot(x, y, nx, ny, col)
                x, y = nx, ny; pts.append((x, y))
            elif c == '+': k = (k + (1 if rev else -1)) % N
            elif c == '-': k = (k + (-1 if rev else 1)) % N
            elif c == '|': k = (k + N // 2) % N
            elif c == '!': rev = not rev
            elif c in '/\\':
                m = ARG.match(s, i); v = num(m.group(0)); i = m.end()
                deg = deg + v if (c == '/') != rev else deg - v
            elif c == '@':
                m = ATARG.match(s, i); a = m.group(0); i = m.end()
                v = num(a.lstrip('iq'))
                if 'q' in a: v = math.sqrt(v)
                if 'i' in a:
                    if v != 0: size = X(size / Fr(v))
                    else: size = Fr(0)
                else:
                    size = X(size * Fr(v))
            elif c in 'c<>' and self.draw_pass:
                m = ARG.match(s, i); n = int(num(m.group(0))); i = m.end()
                if c == 'c': col = n % 256
                elif c == '<':
                    col = (col + n) % 256
                    if col == 0: col = 1
                else:
                    col = (col - n) % 256
                    if col == 0: col = 255
            elif c == '[': stack.append((x, y, k, deg, col, rev, size))
            elif c == ']':
                if not stack: break
                x, y, k, deg, col, rev, size = stack.pop()
        return pts

def line(X0, Y0, X1, Y1):
    """Pixels of a segment: one per step along the major axis, minor
    coordinate = exact value rounded half up (spec 5.6)."""
    dx, dy = X1 - X0, Y1 - Y0; n = max(abs(dx), abs(dy))
    if n == 0: return [(X0, Y0)]
    P = []
    for i in range(n + 1):
        if abs(dx) >= abs(dy):
            px = X0 + (i if dx > 0 else -i)
            P.append((px, math.floor(Fr(Y0) + Fr(dy * (px - X0), dx) + Fr(1, 2))))
        else:
            py = Y0 + (i if dy > 0 else -i)
            P.append((math.floor(Fr(X0) + Fr(dx * (py - Y0), dy) + Fr(1, 2)), py))
    return P

def render(text, name, order, W, H):
    E = parse_entries(text)
    N, axiom, rules = E[name.lower()]
    order = int(order) if order > 0 else 0
    asp = Fr(F32((W / H) * 0.75))
    # pass 1: extents at unit size; colour commands do not take arguments
    s1 = expand(axiom, rules, order, '@/\\')
    pts = Turtle(N, asp, False).run(s1, Fr(0), Fr(0), Fr(1))
    xs = [p[0] for p in pts]; ys = [p[1] for p in pts]
    xmin, xmax, ymin, ymax = min(xs), max(xs), min(ys), max(ys)
    bw, bh = X(xmax - xmin), X(ymax - ymin)
    c = []
    if bw > 0: c.append(X(Fr(W - 10) / bw))
    if bh > 0: c.append(X(Fr(H - 6) / bh))
    sc = Fr(F32(min(c))) if c else Fr(1)
    x0 = X(Fr(W - 1, 2) - X(X(X(xmin + xmax) / 2) * sc))
    y0 = X(Fr(H - 1, 2) - X(X(X(ymin + ymax) / 2) * sc))
    img = bytearray(W * H)
    R = lambda v: math.floor(X(v + Fr(1, 2)))
    def plot(xa, ya, xb, yb, col):
        for px, py in line(R(xa), R(ya), R(xb), R(yb)):
            if 0 <= px < W and 0 <= py < H:
                img[py * W + px] = col
    s2 = expand(axiom, rules, order, 'c<>@/\\')
    Turtle(N, asp, True).run(s2, x0, y0, sc, plot)
    return bytes(img)
